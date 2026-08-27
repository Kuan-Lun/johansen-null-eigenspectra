use super::*;

#[test]
fn test_resumable_functionality() {
    let model = JohansenModel::NoInterceptNoTrend;
    let simulation = test_sim("resumable", model, 2, 103, 5);

    // 清理現有檔案
    simulation.delete_data();

    // 首次運行完整計算
    simulation.run().unwrap();

    let data = simulation.load().unwrap();
    assert_eq!(data.len(), 5);

    // 記錄原始數據（load 依 seed 順序回傳：第 i 筆即 seed = i+1）
    let original_sums: Vec<f64> = data
        .records()
        .iter()
        .map(|record| record.eigenvalues.iter().sum())
        .collect();

    // 移除部分數據來模擬中斷
    let seeds_to_remove = &[2u32, 4];
    let removed_count = remove_seed_from_file(&simulation, seeds_to_remove).unwrap();
    assert_eq!(removed_count, 2);

    // 數據不完整：load 應報錯，改用 load_partial 檢查剩餘數據
    assert!(simulation.load().is_err());
    let partial = simulation.load_partial().unwrap();
    assert!(!partial.is_complete());
    assert_eq!(partial.len(), 3);

    // 驗證剩餘的數據正確（被移除的 seeds 不應出現）
    for record in partial.records() {
        assert!(!seeds_to_remove.contains(&record.seed));
    }

    // 運行斷點續傳
    simulation.run().unwrap();

    // 檢查最終結果
    let final_data = simulation.load().unwrap();
    assert_eq!(final_data.len(), 5);

    // 驗證補齊的數據與原始數據一致（相同 seed 應該產生相同結果）
    let final_sums: Vec<f64> = final_data
        .records()
        .iter()
        .map(|record| record.eigenvalues.iter().sum())
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
    simulation.delete_data();
}
