mod matrix_utils;
mod rng_matrix;

use matrix_utils::{CumsumOrder, sum_of_outer_products};
use rng_matrix::brownian_motion_matrix;

use nalgebra::DMatrix;
use nalgebra_lapack::GeneralizedEigen;

fn main() {
    let nrows = 2; // 列數
    let ncols = 300; // 行數
    let seed: u64 = 42; // 你可以改這個 seed

    let delta_t = 1.0; // 時間間隔
    let start = DMatrix::<f64>::zeros(nrows, 1);
    let bm = brownian_motion_matrix(nrows, ncols, delta_t, CumsumOrder::ColumnWise, start, seed);

    let dbm = bm.columns(1, ncols - 1).into_owned() - bm.columns(0, ncols - 1).into_owned();
    let fm = bm.columns(1, ncols - 1).into_owned();

    let sum_fm_dbm_outer_products = sum_of_outer_products(&fm, &dbm);
    let sum_fm_fm_outer_products = sum_of_outer_products(&fm, &fm);
    // let sub = bm.rows(0, 3);
    println!("{}", &sum_fm_dbm_outer_products);
    println!("{}", &sum_fm_fm_outer_products);

    // Ensure both matrices are square and of the same size
    assert_eq!(
        sum_fm_dbm_outer_products.nrows(),
        sum_fm_dbm_outer_products.ncols(),
        "sum_fm_dbm_outer_products is not square"
    );
    assert_eq!(
        sum_fm_fm_outer_products.nrows(),
        sum_fm_fm_outer_products.ncols(),
        "sum_fm_fm_outer_products is not square"
    );
    assert_eq!(
        sum_fm_dbm_outer_products.nrows(),
        sum_fm_fm_outer_products.nrows(),
        "Matrices are not the same size"
    );

    let ge = GeneralizedEigen::new(sum_fm_dbm_outer_products, sum_fm_fm_outer_products);
    println!("eigenvalues: {:?}", ge.raw_eigenvalues());
}
