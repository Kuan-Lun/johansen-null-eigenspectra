//! Simulation configuration and the main public API.
//!
//! 提供 `EigenvalueSimulation` 結構體，這是整個模組的主要入口點。
//! V7 格式下 seed 由槽位位置隱含：`read_data()` 回傳的第 i 筆
//! 即為 seed = i+1 的結果。

use std::path::Path;

use super::error::StorageError;
use super::file_format::{FileHeader, STATUS_COMPLETE, STATUS_IN_PROGRESS, crc_path, hash_path};
use super::parallel_compute::run_model_simulation;
use super::reader::{decode_payload, open_crc_file, open_data_file, read_slots, scan_slots};
use super::writer::sha256_file;
use crate::johansen_models::JohansenModel;

/// 特徵值模擬配置結構體
/// 封裝所有模擬參數，提供統一的運算和讀取接口
#[derive(Debug, Clone)]
pub struct EigenvalueSimulation {
    /// 約翰森模型
    pub model: JohansenModel,
    /// 矩陣維度
    pub dim: usize,
    /// 時間步驟數
    pub steps: usize,
    /// 模擬運行次數
    pub num_runs: usize,
}

impl EigenvalueSimulation {
    /// Create a new simulation configuration.
    ///
    /// # Panics
    /// Panics when a parameter falls outside the storable range:
    /// `dim` in `1..=254` (so `dim + 1` eigenvalues still fit in a byte),
    /// `steps` in `1..=u32::MAX`, `num_runs` in `1..=u32::MAX`.
    pub fn new(model: JohansenModel, dim: usize, steps: usize, num_runs: usize) -> Self {
        assert!(
            (1..=254).contains(&dim),
            "dim must be in 1..=254, got {dim}"
        );
        assert!(
            steps >= 1 && steps <= u32::MAX as usize,
            "steps must be in 1..=u32::MAX, got {steps}"
        );
        assert!(
            num_runs >= 1 && num_runs <= u32::MAX as usize,
            "num_runs must be in 1..=u32::MAX, got {num_runs}"
        );
        Self {
            model,
            dim,
            steps,
            num_runs,
        }
    }

    /// 構建此模擬配置對應的 V7 檔案標頭（進行中狀態）
    pub(crate) fn file_header(&self) -> FileHeader {
        FileHeader {
            model: self.model.to_number(),
            dim: self.dim as u8,
            steps: self.steps as u32,
            num_runs: self.num_runs as u64,
            eigenvalues_per_run: self.model.eigenvalues_per_run(self.dim) as u8,
            status: STATUS_IN_PROGRESS,
        }
    }

    /// 運行支援斷點續傳的大規模特徵值計算並保存結果
    /// 這是主要的模擬運算接口，針對單一模型進行計算
    pub fn run_simulation(&self) {
        run_model_simulation(self, false);
    }

    /// 運行模擬（安靜模式）
    /// 不輸出進度信息，適合在批量處理或測試環境中使用
    pub fn run_simulation_quiet(&self) {
        run_model_simulation(self, true);
    }

    /// Read the eigenvalue records for seeds `1..=num_runs`, in seed order.
    ///
    /// Seeds are implicit: index `i` holds the record for seed `i + 1`.
    /// Returns an error when any requested record is missing or invalid;
    /// use `read_all_data()` to read whatever is available instead.
    pub fn read_data(&self) -> std::io::Result<Vec<Vec<f64>>> {
        self.read_data_impl().map_err(Into::into)
    }

    fn read_data_impl(&self) -> Result<Vec<Vec<f64>>, StorageError> {
        let filename = self.get_filename(self.model);
        let data = open_data_file(&filename)?;
        data.header.check_compatible(&self.file_header())?;

        let requested = self.num_runs as u64;
        if data.header.status == STATUS_COMPLETE && data.header.num_runs >= requested {
            // 完成協議保證所有槽位有效：直接快速讀取
            return read_slots(&data.file, &data.header, requested);
        }

        // 進行中（或容量不足）的檔案：逐槽驗證
        let limit = data.header.num_runs.min(requested);
        let crc_file = open_crc_file(crc_path(Path::new(&filename)), &data.header)?;
        let mut records: Vec<Vec<f64>> = Vec::new();
        let mut found = 0usize;
        scan_slots(
            &data.file,
            &data.header,
            crc_file.as_ref(),
            limit,
            |_seed, payload, valid| {
                if valid {
                    found += 1;
                    records.push(decode_payload(payload));
                }
            },
        )?;

        if found as u64 != requested {
            return Err(StorageError::Incomplete {
                found,
                expected: self.num_runs,
            });
        }
        Ok(records)
    }

    /// Read every valid record currently stored, with explicit seeds.
    ///
    /// Intended for inspecting incomplete files; records are returned in
    /// seed order and gaps (unfinished seeds) are skipped.
    #[allow(dead_code)] // library API：binary target 未使用
    pub fn read_all_data(&self) -> std::io::Result<Vec<(u32, Vec<f64>)>> {
        self.read_all_data_impl().map_err(Into::into)
    }

    #[allow(dead_code)]
    fn read_all_data_impl(&self) -> Result<Vec<(u32, Vec<f64>)>, StorageError> {
        let filename = self.get_filename(self.model);
        let data = open_data_file(&filename)?;
        data.header.check_compatible(&self.file_header())?;

        if data.header.status == STATUS_COMPLETE {
            let records = read_slots(&data.file, &data.header, data.header.num_runs)?;
            return Ok(records
                .into_iter()
                .enumerate()
                .map(|(i, eigenvalues)| (i as u32 + 1, eigenvalues))
                .collect());
        }

        let crc_file = open_crc_file(crc_path(Path::new(&filename)), &data.header)?;
        let mut records: Vec<(u32, Vec<f64>)> = Vec::new();
        scan_slots(
            &data.file,
            &data.header,
            crc_file.as_ref(),
            data.header.num_runs,
            |seed, payload, valid| {
                if valid {
                    records.push((seed, decode_payload(payload)));
                }
            },
        )?;
        Ok(records)
    }

    /// Verify the data file against its SHA-256 hash file.
    ///
    /// 供拿到別人的檔案、或長期存放後做完整性驗證用；
    /// 一般讀取不會自動驗證（避免每次開檔都付出整檔雜湊的成本）。
    #[allow(dead_code)] // library API：binary target 未使用
    pub fn verify_file(&self) -> std::io::Result<()> {
        self.verify_file_impl().map_err(Into::into)
    }

    #[allow(dead_code)]
    fn verify_file_impl(&self) -> Result<(), StorageError> {
        let filename = self.get_filename(self.model);
        let data_path = Path::new(&filename);
        let hash_file = hash_path(data_path);

        let content = std::fs::read_to_string(&hash_file).map_err(|e| {
            StorageError::HashMismatch(format!(
                "cannot read hash file {}: {e}",
                hash_file.display()
            ))
        })?;
        let expected = content.split_whitespace().next().ok_or_else(|| {
            StorageError::HashMismatch(format!("hash file {} is empty", hash_file.display()))
        })?;

        let actual = sha256_file(data_path)?;
        if actual == expected {
            Ok(())
        } else {
            Err(StorageError::HashMismatch(format!(
                "expected {expected}, computed {actual}"
            )))
        }
    }

    /// Remove the data file and its sidecar files (CRC and hash) for this model.
    pub fn remove_data_files(&self) {
        let filename = self.get_filename(self.model);
        let data_path = Path::new(&filename);
        let _ = std::fs::remove_file(data_path);
        let _ = std::fs::remove_file(crc_path(data_path));
        let _ = std::fs::remove_file(hash_path(data_path));
    }

    /// 獲取當前模型的檔案名稱
    ///
    /// 這是唯一的檔案命名入口點。所有內部檔案操作都通過此方法獲取檔案名稱，
    /// 確保檔案命名邏輯的一致性。如果需要自定義檔案命名規則，
    /// 可以繼承此 struct 並重寫此方法。
    ///
    /// 檔案會自動存放在 data/ 資料夾中，如果資料夾不存在會自動創建。
    /// 如果創建資料夾失敗，程式會 panic，因為沒有資料夾就無法儲存檔案。
    /// 使用 PathBuf 確保跨平台路徑分隔符的正確性。
    pub fn get_filename(&self, model: JohansenModel) -> String {
        use std::path::PathBuf;

        // 確保 data 資料夾存在，失敗時應該 panic 而不是繼續
        let data_dir = PathBuf::from("data");
        std::fs::create_dir_all(&data_dir).unwrap_or_else(|e| {
            panic!(
                "Failed to create data directory '{}': {}. \
                 This is required for storing simulation results. \
                 Please check file system permissions.",
                data_dir.display(),
                e
            );
        });

        // 使用 PathBuf 構建跨平台的檔案路徑
        let filename = format!(
            "eigenvalues_model{}_dim{}_steps{}.dat",
            model.to_number(),
            self.dim,
            self.steps
        );

        data_dir.join(filename).to_string_lossy().to_string()
    }
}
