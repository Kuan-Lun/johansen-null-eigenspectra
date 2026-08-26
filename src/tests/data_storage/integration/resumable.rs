use super::*;

#[test]
fn test_resumable_functionality() {
    // 使用唯一的參數組合避免與其他測試檔案衝突
    let model = JohansenModel::NoInterceptNoTrend;
    let simulation = EigenvalueSimulation::new(model, 2, 103, 5);

    // 清理現有檔案
    simulation.remove_data_files();

    // 首次運行完整計算
    simulation.run_simulation_quiet();

    let data = simulation.read_data().unwrap();
    assert_eq!(data.len(), 5);

    // 記錄原始數據（read_data 依 seed 順序回傳：第 i 筆即 seed = i+1）
    let original_sums: Vec<f64> = data
        .iter()
        .map(|eigenvalues| eigenvalues.iter().sum())
        .collect();

    // 移除部分數據來模擬中斷
    let seeds_to_remove = &[2u32, 4];
    let removed_count = remove_seed_from_file(&simulation, model, seeds_to_remove).unwrap();
    assert_eq!(removed_count, 2);

    // 數據不完整：read_data 應報錯，改用 read_all_data 檢查剩餘數據
    assert!(simulation.read_data().is_err());
    let data_after_removal = simulation.read_all_data().unwrap();
    assert_eq!(data_after_removal.len(), 3);

    // 驗證剩餘的數據正確（被移除的 seeds 不應出現）
    for (seed, _) in &data_after_removal {
        assert!(!seeds_to_remove.contains(seed));
    }

    // 運行斷點續傳
    simulation.run_simulation_quiet();

    // 檢查最終結果
    let final_data = simulation.read_data().unwrap();
    assert_eq!(final_data.len(), 5);

    // 驗證補齊的數據與原始數據一致（相同 seed 應該產生相同結果）
    let final_sums: Vec<f64> = final_data
        .iter()
        .map(|eigenvalues| eigenvalues.iter().sum())
        .collect();
    for (i, (&original_sum, &final_sum)) in original_sums.iter().zip(final_sums.iter()).enumerate()
    {
        assert!(
            (original_sum - final_sum).abs() < 1e-10,
            "Seed {} 的特徵值總和不一致: 原始={:.6}, 最終={:.6}",
            i + 1,
            original_sum,
            final_sum
        );
    }

    // 清理測試檔案
    simulation.remove_data_files();
}
