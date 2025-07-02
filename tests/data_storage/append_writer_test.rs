use johansen_null_eigenspectra::data_storage::append_writer::{
    AppendOnlyWriter, check_append_progress, read_append_file,
};

#[test]
fn test_append_writer_basic() {
    let filename = "test_append_basic.dat";

    // 清理可能存在的檔案
    let _ = std::fs::remove_file(filename);

    // 測試寫入
    {
        let mut writer =
            AppendOnlyWriter::with_expected_size(filename, None, 0, 2, 100, true).unwrap();
        writer.append_eigenvalues(1, &[1.0, 2.0]).unwrap();
        writer.append_eigenvalues(2, &[3.0, 4.0]).unwrap();
        writer.finish().unwrap();
    }

    // 測試讀取
    let (data, model, dim, steps) = read_append_file(filename).unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0], (1, vec![1.0, 2.0]));
    assert_eq!(data[1], (2, vec![3.0, 4.0]));
    assert_eq!(model, 0);
    assert_eq!(dim, 2);
    assert_eq!(steps, 100);

    // 清理
    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_append_writer_progress() {
    let filename = "test_append_progress.dat";

    // 清理可能存在的檔案
    let _ = std::fs::remove_file(filename);

    // 初始狀態
    let (count, seeds) = check_append_progress(filename, 1, 2, 100).unwrap();
    assert_eq!(count, 0);
    assert_eq!(seeds.len(), 0);

    // 寫入一些數據
    {
        let mut writer =
            AppendOnlyWriter::with_expected_size(filename, None, 1, 2, 100, true).unwrap();
        writer.append_eigenvalues(1, &[1.0, 2.0]).unwrap();
        writer.append_eigenvalues(3, &[5.0, 6.0]).unwrap();
        writer.finish().unwrap();
    }

    // 檢查進度
    let (count, seeds) = check_append_progress(filename, 1, 2, 100).unwrap();
    assert_eq!(count, 2);
    assert!(seeds.contains(&1));
    assert!(seeds.contains(&3));

    // 清理
    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_append_writer_incomplete_file() {
    let filename = "test_append_incomplete.dat";

    // 清理可能存在的檔案
    let _ = std::fs::remove_file(filename);

    // 創建不完整的檔案（沒有結束標記）
    {
        let mut writer =
            AppendOnlyWriter::with_expected_size(filename, None, 2, 2, 150, true).unwrap();
        writer.append_eigenvalues(1, &[1.0, 2.0]).unwrap();
        writer.append_eigenvalues(2, &[3.0, 4.0]).unwrap();
        // 故意不調用 finish()
    }

    // 應該還是能讀取到數據（掃描模式）
    let (data, _model, _dim, _steps) = read_append_file(filename).unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0], (1, vec![1.0, 2.0]));
    assert_eq!(data[1], (2, vec![3.0, 4.0]));

    // 清理
    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_append_writer_large_dataset() {
    let filename = "test_append_large.dat";

    // 清理可能存在的檔案
    let _ = std::fs::remove_file(filename);

    // 寫入大量數據
    {
        let mut writer =
            AppendOnlyWriter::with_expected_size(filename, None, 3, 2, 500, true).unwrap();
        for i in 1..=1000 {
            let eigenvalues = vec![i as f64 * 0.1, i as f64 * 0.2];
            writer.append_eigenvalues(i, &eigenvalues).unwrap();
        }
        writer.finish().unwrap();
    }

    // 讀取並驗證
    let (data, _model, _dim, _steps) = read_append_file(filename).unwrap();
    assert_eq!(data.len(), 1000);

    // 檢查前幾個和後幾個
    assert_eq!(data[0], (1, vec![0.1, 0.2]));
    assert_eq!(data[999], (1000, vec![100.0, 200.0]));

    // 清理
    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_append_writer_eigenvalue_consistency() {
    let filename = "test_append_consistency.dat";

    // 清理可能存在的檔案
    let _ = std::fs::remove_file(filename);

    {
        let mut writer =
            AppendOnlyWriter::with_expected_size(filename, None, 0, 2, 100, true).unwrap();
        writer.append_eigenvalues(1, &[1.0, 2.0]).unwrap();

        // 嘗試寫入不一致的特徵值數量，應該失敗
        let result = writer.append_eigenvalues(2, &[3.0, 4.0, 5.0]);
        assert!(result.is_err());
    }

    // 清理
    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_eigenvalue_count_overflow_protection() {
    let filename = "test_overflow.dat";
    let _ = std::fs::remove_file(filename);

    // 嘗試寫入超過 u8::MAX 的特徵值數量
    let large_eigenvalues: Vec<f64> = (0..300).map(|i| i as f64).collect(); // 300 > 255

    // 這應該失敗
    let mut writer =
        AppendOnlyWriter::with_expected_size(filename, None, 0, 255, 1000, true).unwrap();
    let result = writer.append_eigenvalues(1, &large_eigenvalues);

    match result {
        Err(e) => {
            assert!(e.to_string().contains("Too many eigenvalues"));
            assert!(e.to_string().contains("exceeds maximum of 255"));
        }
        Ok(_) => panic!("Should have failed with too many eigenvalues"),
    }

    // 清理
    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_valid_eigenvalue_count_boundary() {
    let filename = "test_boundary.dat";
    let _ = std::fs::remove_file(filename);

    // 創建 exactly 255 個特徵值
    let boundary_eigenvalues: Vec<f64> = (0..255).map(|i| i as f64).collect();

    // 這應該成功
    {
        let mut writer =
            AppendOnlyWriter::with_expected_size(filename, None, 4, 255, 1000, true).unwrap();
        let result = writer.append_eigenvalues(1, &boundary_eigenvalues);
        assert!(
            result.is_ok(),
            "Should succeed with exactly 255 eigenvalues"
        );
        writer.finish().unwrap();
    }

    // 驗證可以讀取
    let (data, _model, _dim, _steps) = read_append_file(filename).unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0].1.len(), 255);

    // 清理
    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_parameter_validation() {
    let filename = "test_param_validation.dat";

    // 清理可能存在的檔案
    let _ = std::fs::remove_file(filename);

    // 先創建一個檔案
    {
        let mut writer =
            AppendOnlyWriter::with_expected_size(filename, None, 1, 3, 100, true).unwrap();
        writer.append_eigenvalues(1, &[1.0, 2.0, 3.0]).unwrap();
        writer.finish().unwrap();
    }

    // 嘗試用不同的參數打開，應該失敗
    let result = AppendOnlyWriter::with_expected_size(filename, None, 2, 3, 100, true); // 不同的 model
    assert!(result.is_err());

    let result = AppendOnlyWriter::with_expected_size(filename, None, 1, 4, 100, true); // 不同的 dim  
    assert!(result.is_err());

    let result = AppendOnlyWriter::with_expected_size(filename, None, 1, 3, 200, true); // 不同的 steps
    assert!(result.is_err());

    // 用正確的參數應該成功
    let result = AppendOnlyWriter::with_expected_size(filename, None, 1, 3, 100, true);
    assert!(result.is_ok());

    // 清理
    let _ = std::fs::remove_file(filename);
}
