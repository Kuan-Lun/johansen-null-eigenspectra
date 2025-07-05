use crate::data_storage::EigenvalueSimulation;
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
