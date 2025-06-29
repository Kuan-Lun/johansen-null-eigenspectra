use johansen_null_eigenspectra::data_storage::*;
use std::fs;

#[test]
fn test_binary_roundtrip() {
    let test_data = vec![
        vec![1.0, 2.0, 3.0],
        vec![4.0, 5.0, 6.0],
        vec![7.0, 8.0, 9.0],
    ];

    let path = "test_eigenvalues.bin";

    write_eigenvalues_binary(&test_data, path).unwrap();
    let loaded_data = read_eigenvalues_binary(path).unwrap();

    assert_eq!(test_data, loaded_data);

    // 清理測試檔案
    let _ = fs::remove_file(path);
}

#[test]
fn test_csv_roundtrip() {
    let test_data = vec![vec![1.1, 2.2, 3.3], vec![4.4, 5.5, 6.6]];

    let path = "test_eigenvalues.csv";

    write_eigenvalues_csv(&test_data, path).unwrap();
    let loaded_data = read_eigenvalues_csv(path).unwrap();

    // 由於浮點數精度問題，我們檢查近似相等
    assert_eq!(test_data.len(), loaded_data.len());
    for (original, loaded) in test_data.iter().zip(loaded_data.iter()) {
        assert_eq!(original.len(), loaded.len());
        for (&orig, &load) in original.iter().zip(loaded.iter()) {
            assert!((orig - load).abs() < 1e-10);
        }
    }

    // 清理測試檔案
    let _ = fs::remove_file(path);
}

#[test]
fn test_single_eigenvalues_roundtrip() {
    let test_data = vec![3.14159, 2.71828, 1.41421, 0.57721];
    let path = "test_single_eigenvalues.bin";

    write_single_eigenvalues(&test_data, path).unwrap();
    let loaded_data = read_single_eigenvalues(path).unwrap();

    assert_eq!(test_data, loaded_data);

    // 清理測試檔案
    let _ = fs::remove_file(path);
}

#[test]
fn test_empty_data() {
    let empty_data: Vec<Vec<f64>> = vec![];
    let path = "test_empty.bin";

    write_eigenvalues_binary(&empty_data, path).unwrap();
    let loaded_data = read_eigenvalues_binary(path).unwrap();

    assert_eq!(empty_data, loaded_data);

    // 清理測試檔案
    let _ = fs::remove_file(path);
}

#[test]
fn test_csv_with_special_values() {
    let test_data = vec![
        vec![0.0, -1.0, f64::INFINITY],
        vec![f64::NEG_INFINITY, 1e-100, 1e100],
    ];

    let path = "test_special_values.csv";

    write_eigenvalues_csv(&test_data, path).unwrap();
    let loaded_data = read_eigenvalues_csv(path).unwrap();

    // 檢查有限值的近似相等
    assert_eq!(test_data.len(), loaded_data.len());
    for (original, loaded) in test_data.iter().zip(loaded_data.iter()) {
        assert_eq!(original.len(), loaded.len());
        for (&orig, &load) in original.iter().zip(loaded.iter()) {
            if orig.is_finite() && load.is_finite() {
                assert!((orig - load).abs() < 1e-10);
            } else {
                // 對於特殊值（無窮大等），檢查是否相同類型
                assert_eq!(orig.is_infinite(), load.is_infinite());
                assert_eq!(orig.is_sign_positive(), load.is_sign_positive());
            }
        }
    }

    // 清理測試檔案
    let _ = fs::remove_file(path);
}

#[test]
fn test_binary_roundtrip_with_seed() {
    let test_data = vec![
        (42u64, vec![1.0, 2.0, 3.0]),
        (123u64, vec![4.0, 5.0, 6.0]),
        (999u64, vec![7.0, 8.0, 9.0]),
    ];

    let path = "test_eigenvalues_with_seed.bin";

    write_eigenvalues_binary_with_seed(&test_data, path).unwrap();
    let loaded_data = read_eigenvalues_binary_with_seed(path).unwrap();

    assert_eq!(test_data, loaded_data);

    // 清理測試檔案
    let _ = fs::remove_file(path);
}

#[test]
fn test_csv_roundtrip_with_seed() {
    let test_data = vec![(1u64, vec![1.1, 2.2, 3.3]), (2u64, vec![4.4, 5.5, 6.6])];

    let path = "test_eigenvalues_with_seed.csv";

    write_eigenvalues_csv_with_seed(&test_data, path).unwrap();
    let loaded_data = read_eigenvalues_csv_with_seed(path).unwrap();

    // 由於浮點數精度問題，我們檢查近似相等
    assert_eq!(test_data.len(), loaded_data.len());
    for ((orig_seed, original), (load_seed, loaded)) in test_data.iter().zip(loaded_data.iter()) {
        assert_eq!(orig_seed, load_seed);
        assert_eq!(original.len(), loaded.len());
        for (&orig, &load) in original.iter().zip(loaded.iter()) {
            assert!((orig - load).abs() < 1e-10);
        }
    }

    // 清理測試檔案
    let _ = fs::remove_file(path);
}

#[test]
fn test_empty_data_with_seed() {
    let empty_data: Vec<(u64, Vec<f64>)> = vec![];
    let path = "test_empty_with_seed.bin";

    write_eigenvalues_binary_with_seed(&empty_data, path).unwrap();
    let loaded_data = read_eigenvalues_binary_with_seed(path).unwrap();

    assert_eq!(empty_data, loaded_data);

    // 清理測試檔案
    let _ = fs::remove_file(path);
}
