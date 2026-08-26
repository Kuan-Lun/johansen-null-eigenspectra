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
    assert_eq!(data.len(), 1); // 只有 seed=1 的一筆記錄
    assert_eq!(data[0].len(), 2); // 2x2 矩陣有 2 個特徵值

    // 清理測試檔案
    for &test_model in &JohansenModel::all_models() {
        EigenvalueSimulation::new(test_model, 2, 11, 1).remove_data_files();
    }
}
