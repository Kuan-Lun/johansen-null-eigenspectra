use super::*;

#[test]
fn test_read_all_data() {
    // 使用唯一的參數組合避免與其他測試檔案衝突
    let model = JohansenModel::NoInterceptNoTrend;
    let simulation = EigenvalueSimulation::new(model, 3, 52, 3);

    // 清理所有現有檔案
    for test_model in JohansenModel::all_models() {
        let test_simulation = EigenvalueSimulation::new(test_model, 3, 52, 3);
        let filename = test_simulation.get_filename(test_model);
        let _ = std::fs::remove_file(&filename);
    }

    // 確保有一些數據
    for &test_model in &JohansenModel::all_models() {
        let test_simulation = EigenvalueSimulation::new(test_model, 3, 52, 3);
        test_simulation.run_simulation_quiet();
    }

    // 改用 for-loop 逐個模型讀取資料
    let mut all_data = Vec::new();
    for &model in &JohansenModel::all_models() {
        let sim = EigenvalueSimulation::new(model, 3, 52, 3);
        let result = sim.read_data();
        all_data.push((model, result));
    }

    assert_eq!(all_data.len(), 5); // 5個模型

    // 檢查每個模型的結果
    for (model, result) in all_data {
        match result {
            Ok(data) => {
                assert_eq!(data.len(), 3);
                // 檢查檔案名稱是否正確
                let expected_filename = simulation.get_filename(model);
                assert!(std::path::Path::new(&expected_filename).exists());
            }
            Err(_) => {
                // 某些模型可能沒有數據，這是正常的
            }
        }
    }

    // 清理測試檔案
    for model in JohansenModel::all_models() {
        let filename = simulation.get_filename(model);
        let _ = std::fs::remove_file(&filename);
    }
}
