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

#[cfg(test)]
mod test_read_methods {
    use super::*;
    use crate::johansen_models::JohansenModel;

    #[test]
    fn test_read_data_vs_read_all_data() {
        // 創建一個小規模測試
        let simulation = EigenvalueSimulation::new(JohansenModel::NoInterceptNoTrend, 2, 10, 3);

        // 清理現有檔案
        let filename = simulation.get_filename(JohansenModel::NoInterceptNoTrend);
        let _ = std::fs::remove_file(&filename);

        // 運行模擬產生數據
        simulation.run_simulation_quiet();

        // 測試 read_all_data
        let all_data = simulation.read_all_data().unwrap();
        assert_eq!(all_data.len(), 3, "read_all_data 應該返回 3 筆記錄");

        // 測試 read_data（應該過濾 seed > num_runs 的記錄）
        let filtered_data = simulation.read_data().unwrap();
        assert_eq!(
            filtered_data.len(),
            3,
            "read_data 應該返回 3 筆記錄（因為 seeds 1-3 都 <= num_runs=3）"
        );

        // 驗證所有返回的 seed 都 <= num_runs
        for (seed, _) in &filtered_data {
            assert!(
                *seed <= simulation.num_runs as u32,
                "seed {} 不應該大於 num_runs {}",
                seed,
                simulation.num_runs
            );
        }

        // 清理測試檔案
        let _ = std::fs::remove_file(&filename);
    }

    #[test]
    fn test_filename_without_num_runs() {
        // 驗證新的檔案命名格式不包含 num_runs
        let simulation1 = EigenvalueSimulation::new(JohansenModel::NoInterceptNoTrend, 5, 999, 100);
        let simulation2 = EigenvalueSimulation::new(JohansenModel::NoInterceptNoTrend, 5, 999, 500);

        let filename1 = simulation1.get_filename(JohansenModel::NoInterceptNoTrend);
        let filename2 = simulation2.get_filename(JohansenModel::NoInterceptNoTrend);

        // 兩個不同 num_runs 的配置應該產生相同的檔案名稱
        assert_eq!(filename1, filename2, "不同 num_runs 應該產生相同的檔案名稱");

        // 檢查檔案名稱格式
        assert!(filename1.contains("model0"));
        assert!(filename1.contains("dim5"));
        assert!(filename1.contains("steps999"));
        assert!(filename1.ends_with(".dat"));

        // 確認檔案名稱不包含 num_runs 值（使用不會與其他參數衝突的值）
        let basename = std::path::Path::new(&filename1)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        assert!(!basename.contains("_100"), "檔案名稱不應該包含 _100");
        assert!(!basename.contains("_500"), "檔案名稱不應該包含 _500");
        assert_eq!(basename, "eigenvalues_model0_dim5_steps999.dat");
    }

    #[test]
    fn test_read_data_error_handling() {
        use std::fs;

        // 創建一個測試配置
        let simulation = EigenvalueSimulation::new(JohansenModel::NoInterceptNoTrend, 2, 201, 5);
        let filename = simulation.get_filename(JohansenModel::NoInterceptNoTrend);
        let _ = fs::remove_file(&filename);

        // 情況1：沒有數據檔案
        let no_data_result = simulation.read_data();
        assert!(
            no_data_result.is_err(),
            "read_data 應該在沒有數據時返回錯誤"
        );

        // 情況2：使用 read_all_data 也應該失敗（因為沒有檔案）
        let no_data_all_result = simulation.read_all_data();
        assert!(
            no_data_all_result.is_err(),
            "read_all_data 應該在沒有檔案時返回錯誤"
        );

        // 運行部分模擬（只產生3筆數據，但期望5筆）
        let partial_sim = EigenvalueSimulation::new(JohansenModel::NoInterceptNoTrend, 2, 201, 3);
        partial_sim.run_simulation_quiet();

        // 情況3：數據不足
        let partial_result = simulation.read_data();
        assert!(
            partial_result.is_err(),
            "read_data 應該在數據不足時返回錯誤"
        );

        let error_msg = partial_result.unwrap_err().to_string();
        assert!(error_msg.contains("expected 5 records, found 3 records"));
        assert!(error_msg.contains("Use `read_all_data()`"));

        // 但 read_all_data 應該成功
        let all_data = simulation.read_all_data().unwrap();
        assert_eq!(all_data.len(), 3, "read_all_data 應該返回3筆數據");

        // 完成剩餘的模擬
        simulation.run_simulation_quiet();

        // 情況4：數據完整
        let complete_data = simulation.read_data().unwrap();
        assert_eq!(complete_data.len(), 5, "read_data 應該返回5筆數據");

        // read_all_data 也應該返回相同的數據
        let all_complete_data = simulation.read_all_data().unwrap();
        assert_eq!(all_complete_data.len(), 5, "read_all_data 應該返回5筆數據");

        // 清理
        let _ = fs::remove_file(&filename);
    }
}
