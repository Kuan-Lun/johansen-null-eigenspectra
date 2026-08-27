use super::*;

#[test]
fn test_multiple_models() {
    // 使用唯一的參數組合避免與其他測試檔案衝突
    // 清理所有現有檔案
    for test_model in JohansenModel::all_models() {
        EigenvalueSimulation::new(test_model, 2, 54, 3).remove_data_files();
    }

    // 運行所有模型的計算
    for &test_model in &JohansenModel::all_models() {
        let test_simulation = EigenvalueSimulation::new(test_model, 2, 54, 3);
        test_simulation.run_simulation_quiet();
    }

    // 檢查每個模型都有對應的檔案
    for test_model in JohansenModel::all_models() {
        let test_simulation = EigenvalueSimulation::new(test_model, 2, 54, 3);
        assert!(test_simulation.get_file_path(test_model).exists());

        let data = test_simulation.read_data().unwrap();
        assert_eq!(data.len(), 3);

        // 檢查數據格式：特徵值數量由模型決定（dim 或 dim+1）
        let expected_count = test_model.eigenvalues_per_run(2);
        for eigenvalues in &data {
            assert_eq!(eigenvalues.len(), expected_count);
            assert!(eigenvalues.iter().all(|&x| x.is_finite()));
        }
    }

    // 清理測試檔案
    for test_model in JohansenModel::all_models() {
        EigenvalueSimulation::new(test_model, 2, 54, 3).remove_data_files();
    }
}
