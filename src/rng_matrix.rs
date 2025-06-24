use crate::matrix_utils::{CumsumOrder, dmatrix_cumsum};

use nalgebra::DMatrix;
use num_cpus;
use rand::SeedableRng;
use rand_distr::Distribution;
use rand_distr::StandardNormal;
use rand_xoshiro::Xoshiro256PlusPlus;
use rayon::prelude::*;

#[allow(dead_code)]
pub fn gen_normal_matrix(nrows: usize, ncols: usize, seed: u64) -> DMatrix<f64> {
    let total = nrows.checked_mul(ncols).expect("Matrix too large");
    let mut data = vec![0.0; total];

    // 取得邏輯核心數
    let n_cpus = num_cpus::get_physical();
    // 讓每個核心有多一點工作量，避免太多小 chunk
    let min_chunk = 10_000;
    let chunk_count = (total / min_chunk).max(n_cpus).min(total);
    let chunk_size = (total + chunk_count - 1) / chunk_count;

    data.par_chunks_mut(chunk_size)
        .enumerate()
        .for_each(|(chunk_idx, chunk)| {
            let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed + chunk_idx as u64);
            let normal = StandardNormal;
            for val in chunk.iter_mut() {
                *val = normal.sample(&mut rng);
            }
        });

    DMatrix::from_vec(nrows, ncols, data)
}

pub fn brownian_motion_matrix(
    nrows: usize,
    ncols: usize,
    delta_t: f64,
    order: CumsumOrder,
    start: DMatrix<f64>,
    seed: u64,
) -> DMatrix<f64> {
    fn check_start_shape(order: CumsumOrder, start: &DMatrix<f64>, nrows: usize, ncols: usize) {
        match order {
            CumsumOrder::ColumnMajor | CumsumOrder::RowMajor => {
                debug_assert_eq!(
                    start.nrows(),
                    1,
                    "start for ColumnMajor/RowMajor must be 1x1"
                );
                debug_assert_eq!(
                    start.ncols(),
                    1,
                    "start for ColumnMajor/RowMajor must be 1x1"
                );
            }
            CumsumOrder::ColumnWise => {
                debug_assert_eq!(start.ncols(), 1, "The number of columns in start must be 1");
                debug_assert_eq!(
                    start.nrows(),
                    nrows,
                    "The number of rows in start must match `nrows`"
                );
            }
            CumsumOrder::RowWise => {
                debug_assert_eq!(
                    start.ncols(),
                    ncols,
                    "The number of columns in start must match `ncols`"
                );
                debug_assert_eq!(start.nrows(), 1, "The number of rows in start must be 1");
            }
        }
    }

    fn make_z_matrix(
        order: CumsumOrder,
        nrows: usize,
        ncols: usize,
        start: &DMatrix<f64>,
        seed: u64,
    ) -> DMatrix<f64> {
        match order {
            CumsumOrder::ColumnMajor | CumsumOrder::RowMajor => {
                let mut z = gen_normal_matrix(nrows, ncols, seed);
                z[(0, 0)] = start[(0, 0)];
                z
            }
            CumsumOrder::ColumnWise => {
                let mut cols: Vec<_> = start.column_iter().collect();
                let gen_mat = gen_normal_matrix(nrows, ncols - 1, seed);
                cols.extend(gen_mat.column_iter());
                DMatrix::from_columns(&cols)
            }
            CumsumOrder::RowWise => {
                let mut rows: Vec<_> = start.row_iter().collect();
                let gen_mat = gen_normal_matrix(nrows - 1, ncols, seed);
                rows.extend(gen_mat.row_iter());
                DMatrix::from_rows(&rows)
            }
        }
    }

    #[cfg(debug_assertions)]
    {
        check_start_shape(order, &start, nrows, ncols);
    }

    let z = make_z_matrix(order, nrows, ncols, &start, seed);
    let sqrt_dt = delta_t.sqrt();
    let scaled = z.map(|v| v * sqrt_dt);
    dmatrix_cumsum(&scaled, order)
}
