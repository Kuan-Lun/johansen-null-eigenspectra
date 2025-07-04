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

    /// 從追加格式讀取指定模型的特徵值數據（包含seed）
    /// 注意：返回的數據可能無序，如需有序請自行排序
    pub fn read_data(&self) -> std::io::Result<Vec<(u32, Vec<f64>)>> {
        let filename = self.get_filename(self.model);
        read_append_file(&filename).map(|(data, _model, _dim, _steps)| data)
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
            "eigenvalues_model{}_dim{}_steps{}_{}.dat",
            &model.to_number(),
            self.dim,
            self.steps,
            self.num_runs
        );

        data_dir.join(filename).to_string_lossy().to_string()
    }
}
