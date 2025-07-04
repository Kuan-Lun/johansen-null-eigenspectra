use super::*;

#[test]
fn test_edge_cases() {
    // 測試非常小的模擬，使用唯一的參數組合
    let model = JohansenModel::NoInterceptNoTrend;
    let tiny_simulation = EigenvalueSimulation::new(model, 2, 11, 1);

    for &test_model in &JohansenModel::all_models() {
        let test_simulation = EigenvalueSimulation::new(test_model, 2, 11, 1);
        test_simulation.run_simulation_quiet();
    }
    let data = tiny_simulation.read_data().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0].0, 1); // seed 應該是 1

    // 清理測試檔案
    let filename = tiny_simulation.get_filename(model);
    let _ = std::fs::remove_file(&filename);
}
