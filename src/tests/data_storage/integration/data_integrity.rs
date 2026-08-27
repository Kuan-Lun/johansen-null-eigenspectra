use super::*;

#[test]
fn test_data_integrity() {
    // 使用唯一的參數組合避免與其他測試檔案衝突
    let model = JohansenModel::NoInterceptNoTrend;
    let simulation = EigenvalueSimulation::new(model, 2, 105, 5);

    // 清理並運行
    simulation.remove_data_files();
    for &test_model in &JohansenModel::all_models() {
        let test_simulation = EigenvalueSimulation::new(test_model, 2, 105, 5);
        test_simulation.run_simulation_quiet();
    }

    // 檢查檔案是否被創建
    let data_path = simulation.get_file_path(model);
    assert!(data_path.exists(), "檔案應該存在: {}", data_path.display());

    // 多次讀取，確保結果一致
    let data1 = simulation.read_data().expect("第一次讀取應該成功");
    let data2 = simulation.read_data().expect("第二次讀取應該成功");
    assert_eq!(data1, data2);

    // 所有數值應為有限值
    for eigenvalues in &data1 {
        assert!(eigenvalues.iter().all(|x| x.is_finite()));
    }

    // read_all_data 的 seed 順序與 read_data 的索引順序一致
    let all_data = simulation.read_all_data().expect("read_all_data 應該成功");
    assert_eq!(all_data.len(), data1.len());
    for (i, (seed, eigenvalues)) in all_data.iter().enumerate() {
        assert_eq!(*seed, i as u32 + 1);
        assert_eq!(eigenvalues, &data1[i]);
    }

    // 清理測試檔案
    for &test_model in &JohansenModel::all_models() {
        EigenvalueSimulation::new(test_model, 2, 105, 5).remove_data_files();
    }
}
