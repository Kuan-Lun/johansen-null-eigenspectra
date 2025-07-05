//! 特徵值模擬配置和主要 API
//!
//! 提供 `EigenvalueSimulation` 結構體，這是整個模組的主要入口點。

use super::parallel_compute::run_model_simulation;
use super::reader::read_append_file;
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
    /// 創建新的特徵值模擬配置
    pub fn new(model: JohansenModel, dim: usize, steps: usize, num_runs: usize) -> Self {
        Self {
            model,
            dim,
            steps,
            num_runs,
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

    /// 從追加格式讀取指定模型的所有特徵值數據（包含seed）
    /// 注意：返回的數據可能無序，如需有序請自行排序
    pub fn read_all_data(&self) -> std::io::Result<Vec<(u32, Vec<f64>)>> {
        let filename = self.get_filename(self.model);
        read_append_file(&filename).map(|(data, _model, _dim, _steps)| data)
    }

    /// 從追加格式讀取指定模型的特徵值數據（包含seed）
    /// 只返回 seed <= num_runs 的記錄，符合當前模擬配置的預期範圍
    /// 注意：返回的數據可能無序，如需有序請自行排序
    ///
    /// # 錯誤處理
    /// 當可用數據數量不等於 num_runs 時返回錯誤，建議：
    /// - 使用 `read_all_data()` 讀取所有可用數據，或
    /// - 運行 `run_simulation()` 生成完整數據
    pub fn read_data(&self) -> std::io::Result<Vec<(u32, Vec<f64>)>> {
        let all_data = self.read_all_data()?;
        let filtered_data: Vec<_> = all_data
            .into_iter()
            .filter(|(seed, _)| *seed <= self.num_runs as u32)
            .collect();

        if filtered_data.len() != self.num_runs {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Data count mismatch: expected {} records, found {} records. \
                     This indicates incomplete simulation data.\n\
                     Solutions:\n\
                     1. Use `read_all_data()` to read available data ({} records)\n\
                     2. Run `run_simulation()` to complete the simulation\n\
                     3. Create a new EigenvalueSimulation with num_runs={} to match existing data",
                    self.num_runs,
                    filtered_data.len(),
                    filtered_data.len(),
                    filtered_data.len()
                ),
            ));
        }

        Ok(filtered_data)
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

        // 使用 PathBuf 構建跨平台的檔案路徑，使用新的檔案擴展名
        let filename = format!(
            "eigenvalues_model{}_dim{}_steps{}.dat",
            &model.to_number(),
            self.dim,
            self.steps
        );

        data_dir.join(filename).to_string_lossy().to_string()
    }
}
