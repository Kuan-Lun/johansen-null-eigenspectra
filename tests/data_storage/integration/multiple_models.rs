use super::*;

#[test]
fn test_multiple_models() {
    // 使用唯一的參數組合避免與其他測試檔案衝突
    let model = JohansenModel::NoInterceptNoTrend;
    let simulation = EigenvalueSimulation::new(model, 2, 54, 3);

    // 清理所有現有檔案
    for test_model in JohansenModel::all_models() {
        let test_simulation = EigenvalueSimulation::new(test_model, 2, 54, 3);
        let filename = test_simulation.get_filename(test_model);
        let _ = std::fs::remove_file(&filename);
    }

    // 運行所有模型的計算
    for &test_model in &JohansenModel::all_models() {
        let test_simulation = EigenvalueSimulation::new(test_model, 2, 54, 3);
        test_simulation.run_simulation_quiet();
    }

    // 檢查每個模型都有對應的檔案
    for test_model in JohansenModel::all_models() {
        let test_simulation = EigenvalueSimulation::new(test_model, 2, 54, 3);
        let filename = test_simulation.get_filename(test_model);
        assert!(std::path::Path::new(&filename).exists());

        let data = test_simulation.read_data().unwrap();
        assert_eq!(data.len(), 3);

        // 檢查數據格式
        for (seed, eigenvalues) in &data {
            assert!(*seed >= 1 && *seed <= 3);
            // 不固定特徵值數量，因為不同模型可能返回不同數量
            assert!(eigenvalues.len() >= 1, "至少應該有1個特徵值");
            assert!(eigenvalues.iter().all(|&x| x.is_finite()));
        }
    }

    // 清理測試檔案
    for model in JohansenModel::all_models() {
        let filename = simulation.get_filename(model);
        let _ = std::fs::remove_file(&filename);
    }
}
