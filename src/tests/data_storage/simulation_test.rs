use super::integration::test_sim;
use crate::data_storage::EigenvalueSimulation;
use crate::johansen_models::JohansenModel;

#[test]
fn test_load_vs_load_partial() {
    // 創建一個小規模測試
    let simulation = test_sim(
        "simulation_test",
        JohansenModel::NoInterceptNoTrend,
        2,
        10,
        3,
    );
    simulation.delete_data();

    // 運行模擬產生數據
    simulation.run().unwrap();

    // load_partial：帶明確 seed，依 seed 順序
    let partial = simulation.load_partial().unwrap();
    assert_eq!(partial.len(), 3, "load_partial 應該返回 3 筆記錄");
    assert!(partial.is_complete());
    for (i, record) in partial.records().iter().enumerate() {
        assert_eq!(record.seed, i as u32 + 1, "seed 應依序從 1 開始");
    }

    // load：seed 隱含於索引（第 i 筆即 seed = i+1），內容應與 load_partial 一致
    let data = simulation.load().unwrap();
    assert_eq!(data.len(), 3, "load 應該返回 3 筆記錄");
    for (i, record) in data.records().iter().enumerate() {
        assert_eq!(record, &partial.records()[i]);
    }

    // 清理測試檔案
    simulation.delete_data();
}

#[test]
fn test_filename_without_num_runs() {
    // 驗證檔案命名格式不包含 num_runs
    let simulation1 =
        EigenvalueSimulation::new(JohansenModel::NoInterceptNoTrend, 5, 999, 100).unwrap();
    let simulation2 =
        EigenvalueSimulation::new(JohansenModel::NoInterceptNoTrend, 5, 999, 500).unwrap();

    let path1 = simulation1.data_path();
    let path2 = simulation2.data_path();

    // 兩個不同 num_runs 的配置應該產生相同的檔案路徑
    assert_eq!(path1, path2, "不同 num_runs 應該產生相同的檔案路徑");

    // 檢查完整的目錄結構與檔案名稱
    let expected = std::path::Path::new("data")
        .join("steps999")
        .join("model0")
        .join("eigenvalues_steps999_model0_dim5.dat");
    assert_eq!(path1, expected);

    // 確認檔案名稱不包含 num_runs 值（使用不會與其他參數衝突的值）
    let basename = path1.file_name().unwrap().to_str().unwrap();
    assert!(!basename.contains("_100"), "檔案名稱不應該包含 _100");
    assert!(!basename.contains("_500"), "檔案名稱不應該包含 _500");
}

#[test]
fn test_load_error_handling() {
    // 創建一個測試配置
    let simulation = test_sim("load_errors", JohansenModel::NoInterceptNoTrend, 2, 201, 5);
    simulation.delete_data();

    // 情況1：沒有數據檔案
    assert!(simulation.load().is_err(), "load 應該在沒有數據時返回錯誤");
    assert!(
        simulation.load_partial().is_err(),
        "load_partial 應該在沒有檔案時返回錯誤"
    );

    // 運行部分模擬（只產生3筆數據，但期望5筆）
    let partial_sim = test_sim("load_errors", JohansenModel::NoInterceptNoTrend, 2, 201, 3);
    partial_sim.run().unwrap();

    // 情況2：數據不足
    let partial_result = simulation.load();
    assert!(partial_result.is_err(), "load 應該在數據不足時返回錯誤");
    let error_msg = partial_result.unwrap_err().to_string();
    assert!(error_msg.contains("expected 5 records, found 3 records"));
    assert!(error_msg.contains("Use `load_partial()`"));

    // 但 load_partial 應該成功，並回報資料尚未到齊
    let partial = simulation.load_partial().unwrap();
    assert_eq!(partial.len(), 3, "load_partial 應該返回3筆數據");
    assert!(!partial.is_complete());

    // 完成剩餘的模擬（觸發檔案容量 3 → 5 的擴容路徑）
    simulation.run().unwrap();

    // 情況3：數據完整
    let complete_data = simulation.load().unwrap();
    assert_eq!(complete_data.len(), 5, "load 應該返回5筆數據");
    let complete_partial = simulation.load_partial().unwrap();
    assert_eq!(complete_partial.len(), 5, "load_partial 應該返回5筆數據");
    assert!(complete_partial.is_complete());

    // 擴容不應改變既有 seeds 的結果
    for (i, record) in partial.records().iter().enumerate() {
        assert_eq!(record.seed, i as u32 + 1);
        assert_eq!(
            record,
            &complete_data.records()[i],
            "擴容後 seed {} 的數據應保持不變",
            record.seed
        );
    }

    // 清理
    simulation.delete_data();
}
