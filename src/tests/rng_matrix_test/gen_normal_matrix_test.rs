use super::{STANDARD_NORMAL_EXPECTED_CDF, STANDARD_NORMAL_QUANTILES};
use super::{assert_vec_approx_eq, interpolated_cdf_sorted};

use crate::rng_matrix::gen_normal_matrix;

#[test]
fn test_gen_normal_matrix_cdf() {
    let nrows = 200;
    let ncols = 300;
    let seed: u64 = 42;
    let matrix = gen_normal_matrix(nrows, ncols, seed);
    let mut vec = matrix.as_slice().to_vec();
    vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let cdf_values = interpolated_cdf_sorted(&vec, STANDARD_NORMAL_QUANTILES);
    assert_vec_approx_eq(&cdf_values, STANDARD_NORMAL_EXPECTED_CDF, 1e-2);
}
