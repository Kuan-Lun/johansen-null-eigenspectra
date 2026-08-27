use super::*;

#[test]
fn test_data_integrity() {
    let model = JohansenModel::NoInterceptNoTrend;
    let simulation = test_sim("data_integrity", model, 2, 105, 5);

    // 清理並運行
    simulation.delete_data();
    for &test_model in &JohansenModel::ALL {
        test_sim("data_integrity", test_model, 2, 105, 5)
            .run()
            .unwrap();
    }

    // 檢查檔案是否被創建
    let data_path = simulation.data_path();
    assert!(data_path.exists(), "檔案應該存在: {}", data_path.display());

    // 多次讀取，確保結果一致
    let data1 = simulation.load().expect("第一次讀取應該成功");
    let data2 = simulation.load().expect("第二次讀取應該成功");
    assert_eq!(data1, data2);

    // 所有數值應為有限值
    for record in data1.records() {
        assert!(record.eigenvalues.iter().all(|x| x.is_finite()));
    }

    // load_partial 的 seed 順序與 load 的索引順序一致
    let partial = simulation.load_partial().expect("load_partial 應該成功");
    assert_eq!(partial.len(), data1.len());
    for (i, record) in partial.records().iter().enumerate() {
        assert_eq!(record.seed, i as u32 + 1);
        assert_eq!(record, &data1.records()[i]);
    }

    // 清理測試檔案
    for &test_model in &JohansenModel::ALL {
        test_sim("data_integrity", test_model, 2, 105, 5).delete_data();
    }
}
