use super::*;

#[test]
fn test_resumable_functionality() {
    // 使用唯一的參數組合避免與其他測試檔案衝突
    let model = JohansenModel::NoInterceptNoTrend;
    let simulation = EigenvalueSimulation::new(model, 2, 103, 5);
    let filename = simulation.get_filename(model);

    // 清理現有檔案
    let _ = std::fs::remove_file(&filename);

    // 首次運行完整計算 - 只運行指定模型
    simulation.run_simulation_quiet();

    let mut data = simulation.read_data().unwrap();
    data.sort_by_key(|(seed, _)| *seed);
    assert_eq!(data.len(), 5);

    // 記錄原始數據
    let original_sums: Vec<f64> = data
        .iter()
        .map(|(_, eigenvalues)| eigenvalues.iter().sum())
        .collect();

    // 移除部分數據來模擬中斷
    let seeds_to_remove = &[2, 4];
    let removed_count = remove_seed_from_file(&simulation, model, seeds_to_remove).unwrap();
    assert_eq!(removed_count, 2);

    // 檢查移除後的數據 - 由於數據不完整，read_data() 會報錯
    // 改用 read_all_data() 來檢查剩餘數據
    let mut data_after_removal = simulation.read_all_data().unwrap();
    data_after_removal.sort_by_key(|(seed, _)| *seed);
    assert_eq!(data_after_removal.len(), 3);

    // 驗證剩餘的數據正確
    for (seed, _) in &data_after_removal {
        assert!(!seeds_to_remove.contains(seed));
    }

    // 運行斷點續傳 - 只運行指定模型
    simulation.run_simulation_quiet();

    // 檢查最終結果
    let mut final_data = simulation.read_data().unwrap();
    final_data.sort_by_key(|(seed, _)| *seed);
    assert_eq!(final_data.len(), 5);

    // 檢查所有 seeds 都存在
    let final_seeds: Vec<u32> = final_data.iter().map(|(seed, _)| *seed).collect();
    for expected_seed in 1..=5 {
        assert!(final_seeds.contains(&expected_seed));
    }

    // 驗證補齊的數據與原始數據一致（相同 seed 應該產生相同結果）
    let final_sums: Vec<f64> = final_data
        .iter()
        .map(|(_, eigenvalues)| eigenvalues.iter().sum())
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
    let _ = std::fs::remove_file(&filename);
    let _ = std::fs::remove_file(&format!("{}.test_backup", filename));
}
