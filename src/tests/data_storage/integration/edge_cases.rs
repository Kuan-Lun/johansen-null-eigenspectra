use super::*;

#[test]
fn test_edge_cases() {
    // 測試非常小的模擬
    let model = JohansenModel::NoInterceptNoTrend;
    let tiny_simulation = test_sim("edge_cases", model, 2, 11, 1);

    for &test_model in &JohansenModel::ALL {
        test_sim("edge_cases", test_model, 2, 11, 1).run().unwrap();
    }
    let data = tiny_simulation.load().unwrap();
    assert_eq!(data.len(), 1); // 只有 seed=1 的一筆記錄
    assert_eq!(data.records()[0].eigenvalues.len(), 2); // 2x2 矩陣有 2 個特徵值

    // 清理測試檔案
    for &test_model in &JohansenModel::ALL {
        test_sim("edge_cases", test_model, 2, 11, 1).delete_data();
    }
}

#[test]
fn test_invalid_config_rejected() {
    // new() 應拒絕超出可儲存範圍的參數
    let model = JohansenModel::NoInterceptNoTrend;
    assert!(EigenvalueSimulation::new(model, 0, 100, 10).is_err()); // dim 太小
    assert!(EigenvalueSimulation::new(model, 255, 100, 10).is_err()); // dim 太大
    assert!(EigenvalueSimulation::new(model, 2, 0, 10).is_err()); // steps 為零
    assert!(EigenvalueSimulation::new(model, 2, 100, 0).is_err()); // num_runs 為零
    assert!(EigenvalueSimulation::new(model, 254, 100, 10).is_ok()); // 邊界值合法
}
