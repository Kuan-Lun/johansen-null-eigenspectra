use crate::matrix_utils::{CumsumOrder, dmatrix_cumsum};
use nalgebra::DMatrix;

#[test]
fn test_cumsum_row_wise() {
    let m = DMatrix::from_row_slice(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let result = dmatrix_cumsum(&m, CumsumOrder::RowWise);
    let expected = DMatrix::from_row_slice(2, 3, &[1.0, 3.0, 6.0, 4.0, 9.0, 15.0]);
    assert_eq!(result, expected);
}

#[test]
fn test_cumsum_column_wise() {
    let m = DMatrix::from_row_slice(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let result = dmatrix_cumsum(&m, CumsumOrder::ColumnWise);
    let expected = DMatrix::from_row_slice(2, 3, &[1.0, 2.0, 3.0, 5.0, 7.0, 9.0]);
    assert_eq!(result, expected);
}

#[test]
fn test_cumsum_column_major() {
    let m = DMatrix::from_row_slice(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let result = dmatrix_cumsum(&m, CumsumOrder::ColumnMajor);
    let expected = DMatrix::from_row_slice(2, 3, &[1.0, 7.0, 15.0, 5.0, 12.0, 21.0]);
    assert_eq!(result, expected);
}

#[test]
fn test_cumsum_row_major() {
    let m = DMatrix::from_row_slice(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let result = dmatrix_cumsum(&m, CumsumOrder::RowMajor);
    let expected = DMatrix::from_row_slice(2, 3, &[1.0, 3.0, 6.0, 10.0, 15.0, 21.0]);
    assert_eq!(result, expected);
}
