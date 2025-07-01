use crate::matrix_utils::{dmatrix_cumsum, CumsumOrder};

use nalgebra::DMatrix;
use num_cpus;
use rand::Rng;
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

    // 建立主 RNG 產生每個 chunk 專用的 seed
    let mut base_rng = Xoshiro256PlusPlus::seed_from_u64(seed);
    let derived_seeds: Vec<u64> = (0..chunk_count).map(|_| base_rng.random()).collect();

    data.par_chunks_mut(chunk_size)
        .zip(derived_seeds.into_par_iter())
        .for_each(|(chunk, chunk_seed)| {
            let mut rng = Xoshiro256PlusPlus::seed_from_u64(chunk_seed);
            let normal = StandardNormal;
            for val in chunk.iter_mut() {
                *val = normal.sample(&mut rng);
            }
        });

    DMatrix::from_vec(nrows, ncols, data)
}

// 布朗運動矩陣的時間軸方向
// 定義時間軸沿著矩陣的哪個方向
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum TimeAxisDirection {
    AlongColumns,
    AlongRows,
}

impl TimeAxisDirection {
    pub fn to_cumsum_order(self) -> CumsumOrder {
        match self {
            TimeAxisDirection::AlongColumns => CumsumOrder::RowWise,
            TimeAxisDirection::AlongRows => CumsumOrder::ColumnWise,
        }
    }
}

pub fn brownian_motion_matrix(
    dim: usize,
    steps: usize,
    delta_t: f64,
    time_axis: TimeAxisDirection,
    start: DMatrix<f64>,
    seed: u64,
) -> DMatrix<f64> {
    #[cfg(debug_assertions)]
    fn check_start_shape(dim: usize, time_axis: TimeAxisDirection, start: &DMatrix<f64>) {
        match time_axis {
            TimeAxisDirection::AlongColumns => {
                debug_assert_eq!(start.ncols(), 1, "The number of columns in start must be 1");
                debug_assert_eq!(
                    start.nrows(),
                    dim,
                    "The number of rows in start must match `dim` (dimensions)"
                );
            }
            TimeAxisDirection::AlongRows => {
                debug_assert_eq!(
                    start.ncols(),
                    dim,
                    "The number of columns in start must match `dim` (dimensions)"
                );
                debug_assert_eq!(start.nrows(), 1, "The number of rows in start must be 1");
            }
        }
    }

    fn make_z_matrix(
        dim: usize,
        steps: usize,
        time_axis: TimeAxisDirection,
        start: &DMatrix<f64>,
        seed: u64,
    ) -> DMatrix<f64> {
        match time_axis {
            TimeAxisDirection::AlongColumns => {
                // 時間軸沿列方向：(dim x (steps+1)) 矩陣
                // start 是初始狀態向量 (dim x 1)

                // 更高效的方法：直接構建最終的數據向量
                let total_cols = steps + 1;
                let mut data = Vec::with_capacity(dim * total_cols);

                // 首先添加 start 列的數據
                data.extend_from_slice(start.as_slice());

                // 然後添加生成的隨機數據
                let gen_mat = gen_normal_matrix(dim, steps, seed);
                data.extend_from_slice(gen_mat.as_slice());

                DMatrix::from_vec(dim, total_cols, data)
            }
            TimeAxisDirection::AlongRows => {
                // 時間軸沿行方向：((steps+1) x dim) 矩陣
                // start 是初始狀態向量 (1 x dim)

                // 更高效的方法：直接構建最終的數據向量
                let total_rows = steps + 1;
                let mut data = Vec::with_capacity(total_rows * dim);

                // 首先添加 start 行的數據
                data.extend_from_slice(start.as_slice());

                // 然後添加生成的隨機數據
                let gen_mat = gen_normal_matrix(steps, dim, seed);
                data.extend_from_slice(gen_mat.as_slice());

                DMatrix::from_vec(total_rows, dim, data)
            }
        }
    }

    #[cfg(debug_assertions)]
    {
        check_start_shape(dim, time_axis, &start);
    }

    let z = make_z_matrix(dim, steps, time_axis, &start, seed);
    let sqrt_dt = delta_t.sqrt();
    let scaled = z.map(|v| v * sqrt_dt);
    dmatrix_cumsum(&scaled, time_axis.to_cumsum_order())
}
