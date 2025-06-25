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
