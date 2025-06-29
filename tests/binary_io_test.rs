use johansen_null_eigenspectra::data_storage::binary_io::read_binary_file;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// 創建測試用的二進制檔案
fn create_test_binary_file<P: AsRef<Path>>(
    path: P,
    data: &[(u64, Vec<f64>)],
) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // 寫入總數量
    writer.write_all(&(data.len() as u64).to_le_bytes())?;

    if !data.is_empty() {
        // 寫入每次運行的特徵值數量
        let eigenvalues_len = data[0].1.len();
        writer.write_all(&(eigenvalues_len as u64).to_le_bytes())?;

        // 寫入所有數據
        for (seed, eigenvalues) in data {
            writer.write_all(&seed.to_le_bytes())?;
            for &val in eigenvalues {
                writer.write_all(&val.to_le_bytes())?;
            }
        }
    } else {
        // 空檔案情況
        writer.write_all(&0u64.to_le_bytes())?;
    }

    writer.flush()?;
    Ok(())
}

#[test]
fn test_read_nonexistent_file() {
    let result = read_binary_file("nonexistent_file.bin");
    assert!(result.is_err());
}

#[test]
fn test_empty_file() {
    let filename = "test_empty_binary_io.bin";

    // 清理可能存在的檔案
    let _ = std::fs::remove_file(filename);

    // 創建空檔案
    create_test_binary_file(filename, &[]).unwrap();

    // 讀取並驗證
    let result = read_binary_file(filename).unwrap();
    assert!(result.is_empty());

    // 清理
    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_single_entry() {
    let filename = "test_single_binary_io.bin";

    // 清理可能存在的檔案
    let _ = std::fs::remove_file(filename);

    let test_data = vec![(42u64, vec![1.0, 2.0, 3.0])];

    // 寫入測試資料
    create_test_binary_file(filename, &test_data).unwrap();

    // 讀取並驗證
    let result = read_binary_file(filename).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, 42);
    assert_eq!(result[0].1, vec![1.0, 2.0, 3.0]);

    // 清理
    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_multiple_entries() {
    let filename = "test_multiple_binary_io.bin";

    // 清理可能存在的檔案
    let _ = std::fs::remove_file(filename);

    let test_data = vec![
        (1u64, vec![1.1, 1.2]),
        (2u64, vec![2.1, 2.2]),
        (3u64, vec![3.1, 3.2]),
    ];

    // 寫入測試資料
    create_test_binary_file(filename, &test_data).unwrap();

    // 讀取並驗證
    let result = read_binary_file(filename).unwrap();
    assert_eq!(result.len(), 3);

    for (i, (seed, eigenvalues)) in result.iter().enumerate() {
        assert_eq!(*seed, test_data[i].0);
        assert_eq!(*eigenvalues, test_data[i].1);
    }

    // 清理
    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_large_dataset() {
    let filename = "test_large_binary_io.bin";

    // 清理可能存在的檔案
    let _ = std::fs::remove_file(filename);

    // 創建大量測試資料
    let mut test_data = Vec::new();
    for i in 1..=1000 {
        let eigenvalues = vec![i as f64 * 0.1, i as f64 * 0.2, i as f64 * 0.3];
        test_data.push((i as u64, eigenvalues));
    }

    // 寫入測試資料
    create_test_binary_file(filename, &test_data).unwrap();

    // 讀取並驗證
    let result = read_binary_file(filename).unwrap();
    assert_eq!(result.len(), 1000);

    // 驗證前幾個和後幾個條目
    assert_eq!(result[0].0, 1);
    assert_eq!(result[0].1, vec![0.1, 0.2, 0.3]);
    assert_eq!(result[999].0, 1000);
    assert_eq!(result[999].1, vec![100.0, 200.0, 300.0]);

    // 清理
    let _ = std::fs::remove_file(filename);
}

#[test]
fn test_floating_point_precision() {
    let filename = "test_precision_binary_io.bin";

    // 清理可能存在的檔案
    let _ = std::fs::remove_file(filename);

    let test_data = vec![
        (1u64, vec![std::f64::consts::PI, std::f64::consts::E]),
        (2u64, vec![-1.23456789012345, 9.87654321098765]),
    ];

    // 寫入測試資料
    create_test_binary_file(filename, &test_data).unwrap();

    // 讀取並驗證精度
    let result = read_binary_file(filename).unwrap();
    assert_eq!(result.len(), 2);

    // 驗證浮點數精度
    assert!((result[0].1[0] - std::f64::consts::PI).abs() < f64::EPSILON);
    assert!((result[0].1[1] - std::f64::consts::E).abs() < f64::EPSILON);
    assert!((result[1].1[0] - (-1.23456789012345)).abs() < f64::EPSILON);
    assert!((result[1].1[1] - 9.87654321098765).abs() < f64::EPSILON);

    // 清理
    let _ = std::fs::remove_file(filename);
}
