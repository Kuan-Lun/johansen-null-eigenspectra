use crate::data_storage::EigenvalueSimulation;
use crate::johansen_models::JohansenModel;

#[test]
fn test_read_data_vs_read_all_data() {
    // 創建一個小規模測試
    let simulation = EigenvalueSimulation::new(JohansenModel::NoInterceptNoTrend, 2, 10, 3);
    simulation.remove_data_files();

    // 運行模擬產生數據
    simulation.run_simulation_quiet();

    // read_all_data：帶明確 seed，依 seed 順序
    let all_data = simulation.read_all_data().unwrap();
    assert_eq!(all_data.len(), 3, "read_all_data 應該返回 3 筆記錄");
    for (i, (seed, _)) in all_data.iter().enumerate() {
        assert_eq!(*seed, i as u32 + 1, "seed 應依序從 1 開始");
    }

    // read_data：seed 隱含於索引（第 i 筆即 seed = i+1），內容應與 read_all_data 一致
    let data = simulation.read_data().unwrap();
    assert_eq!(data.len(), 3, "read_data 應該返回 3 筆記錄");
    for (i, eigenvalues) in data.iter().enumerate() {
        assert_eq!(eigenvalues, &all_data[i].1);
    }

    // 清理測試檔案
    simulation.remove_data_files();
}

#[test]
fn test_filename_without_num_runs() {
    // 驗證檔案命名格式不包含 num_runs
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
    // 創建一個測試配置
    let simulation = EigenvalueSimulation::new(JohansenModel::NoInterceptNoTrend, 2, 201, 5);
    simulation.remove_data_files();

    // 情況1：沒有數據檔案
    assert!(
        simulation.read_data().is_err(),
        "read_data 應該在沒有數據時返回錯誤"
    );
    assert!(
        simulation.read_all_data().is_err(),
        "read_all_data 應該在沒有檔案時返回錯誤"
    );

    // 運行部分模擬（只產生3筆數據，但期望5筆）
    let partial_sim = EigenvalueSimulation::new(JohansenModel::NoInterceptNoTrend, 2, 201, 3);
    partial_sim.run_simulation_quiet();

    // 情況2：數據不足
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

    // 完成剩餘的模擬（觸發檔案容量 3 → 5 的擴容路徑）
    simulation.run_simulation_quiet();

    // 情況3：數據完整
    let complete_data = simulation.read_data().unwrap();
    assert_eq!(complete_data.len(), 5, "read_data 應該返回5筆數據");
    let all_complete_data = simulation.read_all_data().unwrap();
    assert_eq!(all_complete_data.len(), 5, "read_all_data 應該返回5筆數據");

    // 擴容不應改變既有 seeds 的結果
    for (i, (seed, eigenvalues)) in all_data.iter().enumerate() {
        assert_eq!(*seed, i as u32 + 1);
        assert_eq!(
            eigenvalues, &complete_data[i],
            "擴容後 seed {seed} 的數據應保持不變"
        );
    }

    // 清理
    simulation.remove_data_files();
}
