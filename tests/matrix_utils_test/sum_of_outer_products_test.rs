use johansen_null_eigenspectra::matrix_utils::sum_of_outer_products;
use nalgebra::DMatrix;

#[test]
fn test_sum_of_outer_products_basic() {
    // 兩個 2x2 矩陣，每個 column 都是 [1, 2] 和 [3, 4]
    let a = DMatrix::<f64>::from_columns(&[
        nalgebra::DVector::from_vec(vec![1.0, 2.0]),
        nalgebra::DVector::from_vec(vec![3.0, 4.0]),
    ]);
    let b = DMatrix::<f64>::from_columns(&[
        nalgebra::DVector::from_vec(vec![5.0, 6.0]),
        nalgebra::DVector::from_vec(vec![7.0, 8.0]),
    ]);

    // 手動計算 outer sum
    let expected =
        &a.column(0) * &b.column(0).transpose() + &a.column(1) * &b.column(1).transpose();

    let result = sum_of_outer_products(&a, &b);
    assert!(
        (&result - expected).abs().max() < 1e-10,
        "result = \n{}",
        &result
    );
}

#[test]
fn test_sum_of_outer_products_different_dimensions() {
    // 測試不同維度的矩陣：a 是 3x2, b 是 2x2
    let a = DMatrix::<f64>::from_columns(&[
        nalgebra::DVector::from_vec(vec![1.0, 2.0, 3.0]),
        nalgebra::DVector::from_vec(vec![4.0, 5.0, 6.0]),
    ]);
    let b = DMatrix::<f64>::from_columns(&[
        nalgebra::DVector::from_vec(vec![7.0, 8.0]),
        nalgebra::DVector::from_vec(vec![9.0, 10.0]),
    ]);

    let result = sum_of_outer_products(&a, &b);

    // 驗證結果矩陣的維度應該是 3x2 (a的行數 x b的行數)
    assert_eq!(result.nrows(), 3);
    assert_eq!(result.ncols(), 2);

    // 手動計算驗證
    let expected =
        &a.column(0) * &b.column(0).transpose() + &a.column(1) * &b.column(1).transpose();

    assert!(
        (&result - &expected).abs().max() < 1e-10,
        "Different dimensions test failed\nresult = \n{}\nexpected = \n{}",
        &result,
        &expected
    );
}

#[test]
fn test_sum_of_outer_products_single_column() {
    // 測試只有一列的情況
    let a = DMatrix::<f64>::from_columns(&[nalgebra::DVector::from_vec(vec![2.0, 3.0])]);
    let b = DMatrix::<f64>::from_columns(&[nalgebra::DVector::from_vec(vec![4.0, 5.0])]);

    let result = sum_of_outer_products(&a, &b);
    let expected = &a.column(0) * &b.column(0).transpose();

    assert!(
        (&result - expected).abs().max() < 1e-10,
        "Single column test failed"
    );
}

#[test]
fn test_sum_of_outer_products_zero_matrix() {
    // 測試零矩陣
    let a = DMatrix::<f64>::zeros(3, 2);
    let b = DMatrix::<f64>::zeros(2, 2);

    let result = sum_of_outer_products(&a, &b);
    let expected = DMatrix::<f64>::zeros(3, 2);

    assert!(
        (&result - expected).abs().max() < 1e-10,
        "Zero matrix test failed"
    );
}

#[test]
fn test_sum_of_outer_products_identity_like() {
    // 測試類似單位矩陣的情況
    let a = DMatrix::<f64>::from_columns(&[
        nalgebra::DVector::from_vec(vec![1.0, 0.0]),
        nalgebra::DVector::from_vec(vec![0.0, 1.0]),
    ]);
    let b = DMatrix::<f64>::from_columns(&[
        nalgebra::DVector::from_vec(vec![1.0, 0.0]),
        nalgebra::DVector::from_vec(vec![0.0, 1.0]),
    ]);

    let result = sum_of_outer_products(&a, &b);

    // 手動計算：[1,0] * [1,0]^T + [0,1] * [0,1]^T = [[1,0],[0,0]] + [[0,0],[0,1]] = [[1,0],[0,1]]
    let expected = DMatrix::<f64>::from_row_slice(2, 2, &[1.0, 0.0, 0.0, 1.0]);

    assert!(
        (&result - &expected).abs().max() < 1e-10,
        "Identity-like test failed\nresult = \n{}\nexpected = \n{}",
        &result,
        &expected
    );
}

#[test]
fn test_sum_of_outer_products_large_matrix() {
    // 測試較大的矩陣以確保並行計算正確
    let size = 5;
    let n_samples = 10;

    // 創建隨機但可預測的矩陣
    let a = DMatrix::<f64>::from_fn(size, n_samples, |i, j| (i + j) as f64);
    let b = DMatrix::<f64>::from_fn(size, n_samples, |i, j| (i * j + 1) as f64);

    let result = sum_of_outer_products(&a, &b);

    // 驗證維度
    assert_eq!(result.nrows(), size);
    assert_eq!(result.ncols(), size);

    // 手動計算第一個外積來部分驗證
    let first_outer = &a.column(0) * &b.column(0).transpose();

    // 驗證結果的第一個元素應該至少包含第一個外積的貢獻
    assert!(
        result[(0, 0)] >= first_outer[(0, 0)],
        "Large matrix test failed"
    );
}

#[test]
fn test_sum_of_outer_products_numerical_precision() {
    // 測試數值精度
    let a = DMatrix::<f64>::from_columns(&[
        nalgebra::DVector::from_vec(vec![1e-10, 2e-10]),
        nalgebra::DVector::from_vec(vec![3e-10, 4e-10]),
    ]);
    let b = DMatrix::<f64>::from_columns(&[
        nalgebra::DVector::from_vec(vec![5e-10, 6e-10]),
        nalgebra::DVector::from_vec(vec![7e-10, 8e-10]),
    ]);

    let result = sum_of_outer_products(&a, &b);
    let expected =
        &a.column(0) * &b.column(0).transpose() + &a.column(1) * &b.column(1).transpose();

    assert!(
        (&result - expected).abs().max() < 1e-25,
        "Numerical precision test failed"
    );
}
