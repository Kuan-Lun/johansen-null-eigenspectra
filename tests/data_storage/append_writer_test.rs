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
        let mut writer = AppendOnlyWriter::new(filename, true).unwrap();
        writer.append_eigenvalues(1, &[1.0, 2.0]).unwrap();
        writer.append_eigenvalues(2, &[3.0, 4.0]).unwrap();
        writer.finish().unwrap();
    }

    // 測試讀取
    let data = read_append_file(filename).unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0], (1, vec![1.0, 2.0]));
    assert_eq!(data[1], (2, vec![3.0, 4.0]));

    // 清理
    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_append_writer_progress() {
    let filename = "test_append_progress.dat";

    // 清理可能存在的檔案
    let _ = std::fs::remove_file(filename);

    // 初始狀態
    let (count, seeds) = check_append_progress(filename).unwrap();
    assert_eq!(count, 0);
    assert_eq!(seeds.len(), 0);

    // 寫入一些數據
    {
        let mut writer = AppendOnlyWriter::new(filename, true).unwrap();
        writer.append_eigenvalues(1, &[1.0, 2.0]).unwrap();
        writer.append_eigenvalues(3, &[5.0, 6.0]).unwrap();
        writer.finish().unwrap();
    }

    // 檢查進度
    let (count, seeds) = check_append_progress(filename).unwrap();
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
        let mut writer = AppendOnlyWriter::new(filename, true).unwrap();
        writer.append_eigenvalues(1, &[1.0, 2.0]).unwrap();
        writer.append_eigenvalues(2, &[3.0, 4.0]).unwrap();
        // 故意不調用 finish()
    }

    // 應該還是能讀取到數據（掃描模式）
    let data = read_append_file(filename).unwrap();
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
        let mut writer = AppendOnlyWriter::new(filename, true).unwrap();
        for i in 1..=1000 {
            let eigenvalues = vec![i as f64 * 0.1, i as f64 * 0.2];
            writer.append_eigenvalues(i, &eigenvalues).unwrap();
        }
        writer.finish().unwrap();
    }

    // 讀取並驗證
    let data = read_append_file(filename).unwrap();
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
        let mut writer = AppendOnlyWriter::new(filename, true).unwrap();
        writer.append_eigenvalues(1, &[1.0, 2.0]).unwrap();

        // 嘗試寫入不一致的特徵值數量，應該失敗
        let result = writer.append_eigenvalues(2, &[3.0, 4.0, 5.0]);
        assert!(result.is_err());
    }

    // 清理
    let _ = std::fs::remove_file(filename);
}
