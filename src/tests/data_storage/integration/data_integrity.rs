use super::*;

#[test]
fn test_data_integrity() {
    // 使用唯一的參數組合避免與其他測試檔案衝突
    let model = JohansenModel::NoInterceptNoTrend;
    let simulation = EigenvalueSimulation::new(model, 2, 105, 5);

    // 清理並運行
    let filename = simulation.get_filename(model);
    let _ = std::fs::remove_file(&filename);
    for &test_model in &JohansenModel::all_models() {
        let test_simulation = EigenvalueSimulation::new(test_model, 2, 105, 5);
        test_simulation.run_simulation_quiet();
    }

    // 檢查檔案是否被創建
    assert!(
        std::path::Path::new(&filename).exists(),
        "檔案應該存在: {}",
        filename
    );

    // 多次讀取，確保結果一致
    let data1 = simulation.read_data().expect("第一次讀取應該成功");
    let data2 = simulation.read_data().expect("第二次讀取應該成功");

    // 測試排序一致性
    let mut sorted_data1 = simulation.read_data().expect("第一次排序讀取應該成功");
    sorted_data1.sort_by_key(|(seed, _)| *seed);
    let mut sorted_data2 = simulation.read_data().expect("第二次排序讀取應該成功");
    sorted_data2.sort_by_key(|(seed, _)| *seed);

    assert_eq!(data1, data2);
    assert_eq!(sorted_data1, sorted_data2);

    // 檢查排序版本確實是排序的
    for i in 1..sorted_data1.len() {
        assert!(sorted_data1[i - 1].0 <= sorted_data1[i].0);
    }

    // 檢查兩個版本包含相同的數據（只是順序可能不同）
    assert_eq!(data1.len(), sorted_data1.len());
    for (seed, eigenvalues) in &data1 {
        let found = sorted_data1
            .iter()
            .find(|(s, _)| s == seed)
            .expect("應該找到對應的 seed");
        assert_eq!(eigenvalues, &found.1);
    }

    // 清理測試檔案
    let _ = std::fs::remove_file(&filename);
}
