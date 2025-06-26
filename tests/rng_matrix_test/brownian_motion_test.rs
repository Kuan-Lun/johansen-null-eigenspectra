use crate::{STANDARD_NORMAL_EXPECTED_CDF, STANDARD_NORMAL_QUANTILES};
use crate::{assert_vec_approx_eq, interpolated_cdf_sorted};

use johansen_null_eigenspectra::rng_matrix::{TimeAxisDirection, brownian_motion_matrix};

use nalgebra::DMatrix;

#[test]
fn test_brownian_motion_matrix_shapes() {
    let dim = 3;
    let steps = 100;
    let delta_t = 0.01;
    let seed = 42;

    let test_cases = [
        (
            TimeAxisDirection::AlongColumns,
            DMatrix::zeros(dim, 1),
            dim,
            steps + 1,
        ),
        (
            TimeAxisDirection::AlongRows,
            DMatrix::zeros(1, dim),
            steps + 1,
            dim,
        ),
    ];
    for (direction, start, expected_rows, expected_cols) in test_cases {
        let result = brownian_motion_matrix(dim, steps, delta_t, direction, start, seed);
        assert_eq!(
            result.nrows(),
            expected_rows,
            "Wrong number of rows for {:?}",
            direction
        );
        assert_eq!(
            result.ncols(),
            expected_cols,
            "Wrong number of cols for {:?}",
            direction
        );
    }
}

#[test]
fn test_brownian_motion_increments_normality() {
    let dim = 3;
    let steps = 1000;
    let delta_t = 0.01;
    let seed = 42;

    let test_cases = [
        (TimeAxisDirection::AlongColumns, DMatrix::zeros(dim, 1)),
        (TimeAxisDirection::AlongRows, DMatrix::zeros(1, dim)),
    ];

    for (direction, start) in test_cases {
        let result = brownian_motion_matrix(dim, steps, delta_t, direction, start, seed);

        // 對每個維度檢查增量是否常態
        for d in 0..dim {
            // 計算增量 (差分)
            let mut increments = Vec::new();

            match direction {
                TimeAxisDirection::AlongColumns => {
                    // 時間沿列方向：result[(d, t)]
                    for t in 1..result.ncols() {
                        let increment = (result[(d, t)] - result[(d, t - 1)]) / delta_t.sqrt();
                        increments.push(increment);
                    }
                }
                TimeAxisDirection::AlongRows => {
                    // 時間沿行方向：result[(t, d)]
                    for t in 1..result.nrows() {
                        let increment = (result[(t, d)] - result[(t - 1, d)]) / delta_t.sqrt();
                        increments.push(increment);
                    }
                }
            }

            // 排序以計算經驗 CDF
            increments.sort_by(|a, b| a.partial_cmp(b).unwrap());

            // 計算經驗 CDF 並與標準常態分布比較
            let empirical_cdf = interpolated_cdf_sorted(&increments, STANDARD_NORMAL_QUANTILES);
            assert_vec_approx_eq(&empirical_cdf, STANDARD_NORMAL_EXPECTED_CDF, 0.05);

            println!(
                "Direction {:?}, Dimension {} increments pass normality test",
                direction, d
            );
        }
    }
}

#[test]
fn test_brownian_motion_reproducibility() {
    let dim = 2;
    let steps = 50;
    let delta_t = 0.01;
    let start = DMatrix::zeros(dim, 1);
    let seed = 123;

    let result1 = brownian_motion_matrix(
        dim,
        steps,
        delta_t,
        TimeAxisDirection::AlongColumns,
        start.clone(),
        seed,
    );

    let result2 = brownian_motion_matrix(
        dim,
        steps,
        delta_t,
        TimeAxisDirection::AlongColumns,
        start,
        seed,
    );

    assert_eq!(result1, result2);
}

#[test]
fn test_brownian_motion_different_seeds() {
    let dim = 2;
    let steps = 50;
    let delta_t = 0.01;
    let start = DMatrix::zeros(dim, 1);

    let result1 = brownian_motion_matrix(
        dim,
        steps,
        delta_t,
        TimeAxisDirection::AlongColumns,
        start.clone(),
        123,
    );

    let result2 = brownian_motion_matrix(
        dim,
        steps,
        delta_t,
        TimeAxisDirection::AlongColumns,
        start,
        456,
    );

    assert_ne!(result1, result2);
}

#[test]
#[should_panic(expected = "The number of columns in start must be 1")]
fn test_brownian_motion_wrong_start_shape_along_columns() {
    let dim = 2;
    let steps = 10;
    let delta_t = 0.01;
    let start = DMatrix::zeros(2, 2); // 錯誤的矩陣尺寸
    let seed = 42;

    brownian_motion_matrix(
        dim,
        steps,
        delta_t,
        TimeAxisDirection::AlongColumns,
        start,
        seed,
    );
}

#[test]
#[should_panic(expected = "The number of rows in start must be 1")]
fn test_brownian_motion_wrong_start_shape_along_rows() {
    let dim = 2;
    let steps = 10;
    let delta_t = 0.01;
    let start = DMatrix::zeros(2, 2); // 錯誤的矩陣尺寸
    let seed = 42;

    brownian_motion_matrix(
        dim,
        steps,
        delta_t,
        TimeAxisDirection::AlongRows,
        start,
        seed,
    );
}
