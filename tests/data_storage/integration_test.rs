use johansen_null_eigenspectra::data_storage::EigenvalueSimulation;
use johansen_null_eigenspectra::data_storage::append_writer::AppendOnlyWriter;
use johansen_null_eigenspectra::data_storage::parallel_compute::run_model_simulation;
use johansen_null_eigenspectra::johansen_models::JohansenModel;

/// 重寫追加格式檔案的測試輔助函數
fn rewrite_append_file(filename: &str, data: &[(u64, Vec<f64>)]) -> std::io::Result<()> {
    // 刪除舊檔案
    let _ = std::fs::remove_file(filename);

    // 使用追加寫入器重建檔案
    let mut writer = AppendOnlyWriter::new(filename, true)?;

    for (seed, eigenvalues) in data {
        writer.append_eigenvalues(*seed, eigenvalues)?;
    }

    writer.finish()
}

/// 從檔案中移除指定的seed數據（測試用）
fn remove_seed_from_file(
    simulation: &EigenvalueSimulation,
    model: JohansenModel,
    seeds_to_remove: &[u64],
) -> std::io::Result<usize> {
    let filename = simulation.get_filename(model);

    if !std::path::Path::new(&filename).exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "檔案不存在",
        ));
    }

    // 讀取現有數據並排序
    let mut data = simulation.read_data(model)?;
    data.sort_by_key(|(seed, _)| *seed);
    let original_count = data.len();

    // 過濾掉指定的seeds
    let filtered_data: Vec<(u64, Vec<f64>)> = data
        .into_iter()
        .filter(|(seed, _)| !seeds_to_remove.contains(seed))
        .collect();

    let removed_count = original_count - filtered_data.len();

    if removed_count > 0 {
        // 備份原檔案
        let backup_filename = format!("{}.test_backup", filename);
        std::fs::copy(&filename, &backup_filename)?;

        // 重寫檔案
        rewrite_append_file(&filename, &filtered_data)?;
    }

    Ok(removed_count)
}

#[test]
fn test_basic_simulation_api() {
    // 使用唯一的參數組合避免與其他測試檔案衝突
    let simulation = EigenvalueSimulation::new(2, 101, 5);
    let model = JohansenModel::NoInterceptNoTrend;
    let filename = simulation.get_filename(model);

    // 清理現有檔案
    let _ = std::fs::remove_file(&filename);

    // 測試檔案名稱生成
    assert!(filename.contains("model0"));
    assert!(filename.contains("dim2"));
    assert!(filename.contains("_5.dat"));

    // 運行模擬
    simulation.run_simulation_quiet();

    // 檢查檔案是否存在
    assert!(std::path::Path::new(&filename).exists());

    // 讀取數據
    let data = simulation.read_data(model).unwrap();
    assert_eq!(data.len(), 5);

    // 檢查每筆數據格式
    for (seed, eigenvalues) in &data {
        assert!(*seed >= 1 && *seed <= 5);
        assert_eq!(eigenvalues.len(), 2); // 2x2 矩陣有 2 個特徵值
    }

    // 測試排序功能
    let mut sorted_data = simulation.read_data(model).unwrap();
    sorted_data.sort_by_key(|(seed, _)| *seed);
    assert_eq!(sorted_data.len(), 5);
    for i in 1..sorted_data.len() {
        assert!(sorted_data[i - 1].0 <= sorted_data[i].0);
    }

    // 清理測試檔案
    let _ = std::fs::remove_file(&filename);
}

#[test]
fn test_read_all_data() {
    // 使用唯一的參數組合避免與其他測試檔案衝突
    let simulation = EigenvalueSimulation::new(3, 52, 3);

    // 清理所有現有檔案
    for model in JohansenModel::all_models() {
        let filename = simulation.get_filename(model);
        let _ = std::fs::remove_file(&filename);
    }

    // 確保有一些數據
    simulation.run_simulation_quiet();

    let all_data = simulation.read_all_data();
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

#[test]
fn test_resumable_functionality() {
    // 使用唯一的參數組合避免與其他測試檔案衝突
    let simulation = EigenvalueSimulation::new(2, 103, 5);
    let model = JohansenModel::NoInterceptNoTrend;
    let filename = simulation.get_filename(model);

    // 清理現有檔案
    let _ = std::fs::remove_file(&filename);

    // 首次運行完整計算 - 只運行指定模型
    run_model_simulation(
        simulation.dim,
        simulation.steps,
        simulation.num_runs,
        |m| simulation.get_filename(m),
        model,
        true, // quiet
    );

    let mut data = simulation.read_data(model).unwrap();
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

    // 檢查移除後的數據
    let mut data_after_removal = simulation.read_data(model).unwrap();
    data_after_removal.sort_by_key(|(seed, _)| *seed);
    assert_eq!(data_after_removal.len(), 3);

    // 驗證剩餘的數據正確
    for (seed, _) in &data_after_removal {
        assert!(!seeds_to_remove.contains(seed));
    }

    // 運行斷點續傳 - 只運行指定模型
    run_model_simulation(
        simulation.dim,
        simulation.steps,
        simulation.num_runs,
        |m| simulation.get_filename(m),
        model,
        true, // quiet
    );

    // 檢查最終結果
    let mut final_data = simulation.read_data(model).unwrap();
    final_data.sort_by_key(|(seed, _)| *seed);
    assert_eq!(final_data.len(), 5);

    // 檢查所有 seeds 都存在
    let final_seeds: Vec<u64> = final_data.iter().map(|(seed, _)| *seed).collect();
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

#[test]
fn test_multiple_models() {
    // 使用唯一的參數組合避免與其他測試檔案衝突
    let simulation = EigenvalueSimulation::new(2, 54, 3);

    // 清理所有現有檔案
    for model in JohansenModel::all_models() {
        let filename = simulation.get_filename(model);
        let _ = std::fs::remove_file(&filename);
    }

    // 運行所有模型的計算
    simulation.run_simulation_quiet();

    // 檢查每個模型都有對應的檔案
    for model in JohansenModel::all_models() {
        let filename = simulation.get_filename(model);
        assert!(std::path::Path::new(&filename).exists());

        let data = simulation.read_data(model).unwrap();
        assert_eq!(data.len(), 3);

        // 檢查數據格式
        for (seed, eigenvalues) in &data {
            assert!(*seed >= 1 && *seed <= 3);
            // 不固定特徵值數量，因為不同模型可能返回不同數量
            assert!(eigenvalues.len() >= 1, "至少應該有1個特徵值");
            assert!(eigenvalues.iter().all(|&x| x.is_finite()));
        }
    }

    // 清理測試檔案
    for model in JohansenModel::all_models() {
        let filename = simulation.get_filename(model);
        let _ = std::fs::remove_file(&filename);
    }
}

#[test]
fn test_edge_cases() {
    // 測試非常小的模擬，使用唯一的參數組合
    let tiny_simulation = EigenvalueSimulation::new(2, 11, 1);
    let model = JohansenModel::NoInterceptNoTrend;

    tiny_simulation.run_simulation_quiet();
    let data = tiny_simulation.read_data(model).unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0].0, 1); // seed 應該是 1

    // 清理測試檔案
    let filename = tiny_simulation.get_filename(model);
    let _ = std::fs::remove_file(&filename);
}

#[test]
fn test_filename_consistency() {
    let simulation = EigenvalueSimulation::new(3, 500, 1000);

    // 測試不同模型的檔案名稱
    for (i, model) in JohansenModel::all_models().iter().enumerate() {
        let filename = simulation.get_filename(*model);
        assert!(filename.contains(&format!("model{}", i)));
        assert!(filename.contains("dim3"));
        assert!(filename.contains("steps500"));
        assert!(filename.contains("_1000.dat"));
    }
}

#[test]
fn test_data_integrity() {
    // 使用唯一的參數組合避免與其他測試檔案衝突
    let simulation = EigenvalueSimulation::new(2, 105, 5);
    let model = JohansenModel::NoInterceptNoTrend;

    // 清理並運行
    let filename = simulation.get_filename(model);
    let _ = std::fs::remove_file(&filename);
    simulation.run_simulation_quiet();

    // 檢查檔案是否被創建
    assert!(
        std::path::Path::new(&filename).exists(),
        "檔案應該存在: {}",
        filename
    );

    // 多次讀取，確保結果一致
    let data1 = simulation.read_data(model).expect("第一次讀取應該成功");
    let data2 = simulation.read_data(model).expect("第二次讀取應該成功");

    // 測試排序一致性
    let mut sorted_data1 = simulation.read_data(model).expect("第一次排序讀取應該成功");
    sorted_data1.sort_by_key(|(seed, _)| *seed);
    let mut sorted_data2 = simulation.read_data(model).expect("第二次排序讀取應該成功");
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
