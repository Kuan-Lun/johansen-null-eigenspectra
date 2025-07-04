use super::*;

#[test]
fn test_basic_simulation_api() {
    // 使用唯一的參數組合避免與其他測試檔案衝突
    let model = JohansenModel::NoInterceptNoTrend;
    let simulation = EigenvalueSimulation::new(model, 2, 101, 5);
    let filename = simulation.get_filename(model);

    // 清理現有檔案
    let _ = std::fs::remove_file(&filename);

    // 測試檔案名稱生成
    assert!(filename.contains("model0"));
    assert!(filename.contains("dim2"));
    assert!(filename.contains("_5.dat"));

    // 運行模擬
    for &model in &JohansenModel::all_models() {
        let model_simulation = EigenvalueSimulation::new(model, 2, 101, 5);
        model_simulation.run_simulation_quiet();
    }

    // 檢查檔案是否存在
    assert!(std::path::Path::new(&filename).exists());

    // 讀取數據
    let data = simulation.read_data().unwrap();
    assert_eq!(data.len(), 5);

    // 檢查每筆數據格式
    for (seed, eigenvalues) in &data {
        assert!(*seed >= 1 && *seed <= 5);
        assert_eq!(eigenvalues.len(), 2); // 2x2 矩陣有 2 個特徵值
    }

    // 測試排序功能
    let mut sorted_data = simulation.read_data().unwrap();
    sorted_data.sort_by_key(|(seed, _)| *seed);
    assert_eq!(sorted_data.len(), 5);
    for i in 1..sorted_data.len() {
        assert!(sorted_data[i - 1].0 <= sorted_data[i].0);
    }

    // 清理測試檔案
    let _ = std::fs::remove_file(&filename);
}
