//! Simulation configuration and the main public API.
//!
//! 提供 `EigenvalueSimulation` 結構體，這是整個 crate 的主要入口點。
//! V7 格式下 seed 由槽位位置隱含：slot i 存放 seed = i+1 的結果。

use std::path::PathBuf;

use super::file_format::{FileHeader, STATUS_COMPLETE, STATUS_IN_PROGRESS, crc_path, hash_path};
use super::parallel_compute::{Progress, run_model_simulation};
use super::reader::{decode_payload, open_crc_file, open_data_file, read_slots, scan_slots};
use super::writer::sha256_file;
use crate::error::Error;
use crate::johansen_models::JohansenModel;
use crate::simulation_data::{SimulationData, SimulationRecord};

/// 特徵值模擬配置結構體
/// 封裝所有模擬參數，提供統一的運算和讀取接口
///
/// 欄位為私有以維持 `new()` 檢查的不變量；以同名 getter 讀取。
#[derive(Debug, Clone)]
pub struct EigenvalueSimulation {
    /// 約翰森模型
    model: JohansenModel,
    /// 矩陣維度
    dim: usize,
    /// 時間步驟數
    steps: usize,
    /// 模擬運行次數
    num_runs: usize,
    /// 資料根目錄（預設為工作目錄下的 `data/`）
    data_dir: PathBuf,
}

impl EigenvalueSimulation {
    /// Create a new simulation configuration.
    ///
    /// 參數超出可儲存範圍時回傳 [`Error::InvalidConfig`]：
    /// `dim` 須在 `1..=254`（`dim + 1` 個特徵值仍須放得進一個 byte）、
    /// `steps` 與 `num_runs` 須在 `1..=u32::MAX`。
    pub fn new(
        model: JohansenModel,
        dim: usize,
        steps: usize,
        num_runs: usize,
    ) -> Result<Self, Error> {
        if !(1..=254).contains(&dim) {
            return Err(Error::InvalidConfig(format!(
                "dim must be in 1..=254, got {dim}"
            )));
        }
        if steps < 1 || steps > u32::MAX as usize {
            return Err(Error::InvalidConfig(format!(
                "steps must be in 1..=u32::MAX, got {steps}"
            )));
        }
        if num_runs < 1 || num_runs > u32::MAX as usize {
            return Err(Error::InvalidConfig(format!(
                "num_runs must be in 1..=u32::MAX, got {num_runs}"
            )));
        }
        Ok(Self {
            model,
            dim,
            steps,
            num_runs,
            data_dir: PathBuf::from("data"),
        })
    }

    /// 更換資料根目錄（預設為工作目錄下的 `data/`）
    pub fn with_data_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.data_dir = dir.into();
        self
    }

    /// 約翰森模型
    pub fn model(&self) -> JohansenModel {
        self.model
    }

    /// 矩陣維度
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// 時間步驟數
    pub fn steps(&self) -> usize {
        self.steps
    }

    /// 模擬運行次數
    pub fn num_runs(&self) -> usize {
        self.num_runs
    }

    /// 構建此模擬配置對應的 V7 檔案標頭（進行中狀態）
    pub(crate) fn file_header(&self) -> FileHeader {
        FileHeader {
            model: u8::from(self.model),
            dim: self.dim as u8,
            steps: self.steps as u32,
            num_runs: self.num_runs as u64,
            eigenvalues_per_run: self.model.eigenvalues_per_run(self.dim) as u8,
            status: STATUS_IN_PROGRESS,
        }
    }

    /// Run the simulation to completion, resuming from any existing data.
    ///
    /// 安靜執行，不輸出任何內容；需要進度回報時用 [`run_with_progress`]。
    /// 既有檔案與配置不相容時回傳錯誤而**不會**自行刪檔：呼叫端以
    /// [`Error::is_recreatable`] 判斷可否刪檔重建，刪檔用 [`delete_data`]。
    ///
    /// [`run_with_progress`]: Self::run_with_progress
    /// [`delete_data`]: Self::delete_data
    pub fn run(&self) -> Result<(), Error> {
        run_model_simulation(self, |_| {})
    }

    /// Run the simulation, reporting [`Progress`] through a callback.
    ///
    /// 開檔完成後立即回報一次（表示斷點續傳的起始點；`completed == total`
    /// 表示先前已全部完成），之後每批落盤後回報一次。
    /// 其餘行為與 [`run`](Self::run) 相同。
    pub fn run_with_progress(&self, on_progress: impl FnMut(Progress)) -> Result<(), Error> {
        run_model_simulation(self, on_progress)
    }

    /// Load the records for seeds `1..=num_runs` into memory, in seed order.
    ///
    /// 任何一筆缺漏或無效即回傳 [`Error::Incomplete`]；
    /// 想讀取未完成檔案的現有資料改用 [`load_partial`](Self::load_partial)。
    pub fn load(&self) -> Result<SimulationData, Error> {
        let data_path = self.data_path();
        let data = open_data_file(&data_path)?;
        data.header.check_compatible(&self.file_header())?;

        let requested = self.num_runs as u64;
        if data.header.status == STATUS_COMPLETE && data.header.num_runs >= requested {
            // 完成協議保證所有槽位有效：直接快速讀取
            let records = read_slots(&data.file, &data.header, requested)?;
            return Ok(SimulationData::new(seeded_records(records), true));
        }

        // 進行中（或容量不足）的檔案：逐槽驗證
        let limit = data.header.num_runs.min(requested);
        let crc_file = open_crc_file(crc_path(&data_path), &data.header)?;
        let mut records: Vec<SimulationRecord> = Vec::new();
        scan_slots(
            &data.file,
            &data.header,
            crc_file.as_ref(),
            limit,
            |seed, payload, valid| {
                if valid {
                    records.push(SimulationRecord {
                        seed,
                        eigenvalues: decode_payload(payload),
                    });
                }
            },
        )?;

        if records.len() as u64 != requested {
            return Err(Error::Incomplete {
                found: records.len(),
                expected: self.num_runs,
            });
        }
        Ok(SimulationData::new(records, true))
    }

    /// Load every valid record currently stored, skipping gaps.
    ///
    /// 供檢視未完成的檔案；記錄依 seed 順序回傳，缺漏的 seed 直接跳過。
    /// 回傳資料的 [`is_complete`](SimulationData::is_complete) 表示
    /// 配置要求的 `1..=num_runs` 是否全部到齊。
    pub fn load_partial(&self) -> Result<SimulationData, Error> {
        let data_path = self.data_path();
        let data = open_data_file(&data_path)?;
        data.header.check_compatible(&self.file_header())?;

        if data.header.status == STATUS_COMPLETE {
            let records = read_slots(&data.file, &data.header, data.header.num_runs)?;
            let complete = data.header.num_runs >= self.num_runs as u64;
            return Ok(SimulationData::new(seeded_records(records), complete));
        }

        let crc_file = open_crc_file(crc_path(&data_path), &data.header)?;
        let mut records: Vec<SimulationRecord> = Vec::new();
        scan_slots(
            &data.file,
            &data.header,
            crc_file.as_ref(),
            data.header.num_runs,
            |seed, payload, valid| {
                if valid {
                    records.push(SimulationRecord {
                        seed,
                        eigenvalues: decode_payload(payload),
                    });
                }
            },
        )?;
        let complete = records
            .iter()
            .filter(|record| record.seed as usize <= self.num_runs)
            .count()
            == self.num_runs;
        Ok(SimulationData::new(records, complete))
    }

    /// Verify the data file against its SHA-256 hash file.
    ///
    /// 供拿到別人的檔案、或長期存放後做完整性驗證用；
    /// 一般讀取不會自動驗證（避免每次開檔都付出整檔雜湊的成本）。
    /// 雜湊檔為標準 `sha256sum` 格式，也可用 `shasum -a 256 -c` 外部驗證。
    pub fn verify(&self) -> Result<(), Error> {
        let data_path = self.data_path();
        let hash_file = hash_path(&data_path);

        let content = std::fs::read_to_string(&hash_file).map_err(|e| {
            Error::HashMismatch(format!(
                "cannot read hash file {}: {e}",
                hash_file.display()
            ))
        })?;
        let expected = content.split_whitespace().next().ok_or_else(|| {
            Error::HashMismatch(format!("hash file {} is empty", hash_file.display()))
        })?;

        let actual = sha256_file(&data_path)?;
        if actual == expected {
            Ok(())
        } else {
            Err(Error::HashMismatch(format!(
                "expected {expected}, computed {actual}"
            )))
        }
    }

    /// Remove the data file and its sidecar files (CRC and hash) for this
    /// configuration, then clean up the `model{}/` and `steps{}/` directories
    /// if they became empty.
    ///
    /// **破壞性操作**：資料皆可由 seed 重算，但重算需要付出計算時間。
    /// 主要用途是處理 [`Error::is_recreatable`] 為真的不相容檔案。
    pub fn delete_data(&self) {
        let data_path = self.data_path();
        let _ = std::fs::remove_file(&data_path);
        let _ = std::fs::remove_file(crc_path(&data_path));
        let _ = std::fs::remove_file(hash_path(&data_path));

        // 由內往外移除已空的目錄；remove_dir 對非空目錄會失敗，
        // 因此不會動到其他 model/steps 的資料
        if let Some(model_dir) = data_path.parent() {
            let _ = std::fs::remove_dir(model_dir);
            if let Some(steps_dir) = model_dir.parent() {
                let _ = std::fs::remove_dir(steps_dir);
            }
        }
    }

    /// 此模擬配置的資料檔案路徑
    ///
    /// 這是唯一的檔案命名入口點，所有檔案操作都通過此方法獲取路徑。
    /// 格式為 `{data_dir}/steps{steps}/model{model}/eigenvalues_steps{steps}_model{model}_dim{dim}.dat`，
    /// 其中 `data_dir` 預設為工作目錄下的 `data/`（見 [`with_data_dir`](Self::with_data_dir)）。
    /// 此方法為純函數，不會建立目錄；目錄由寫入端在建立檔案前建立。
    pub fn data_path(&self) -> PathBuf {
        let model = u8::from(self.model);
        self.data_dir
            .join(format!("steps{}", self.steps))
            .join(format!("model{model}"))
            .join(format!(
                "eigenvalues_steps{}_model{model}_dim{}.dat",
                self.steps, self.dim
            ))
    }
}

/// 將依 seed 順序的特徵值向量標上隱含的 seed（第 i 筆即 seed = i+1）
fn seeded_records(records: Vec<Vec<f64>>) -> Vec<SimulationRecord> {
    records
        .into_iter()
        .enumerate()
        .map(|(i, eigenvalues)| SimulationRecord {
            seed: i as u32 + 1,
            eigenvalues,
        })
        .collect()
}
