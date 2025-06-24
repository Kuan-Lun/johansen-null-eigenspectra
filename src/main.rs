mod matrix_utils;
mod rng_matrix;

use matrix_utils::CumsumOrder;
use rng_matrix::{brownian_motion_matrix, gen_normal_matrix};

use nalgebra::DMatrix;
use rand::SeedableRng;
use rand::rngs::StdRng;

fn main() {
    let nrows = 2; // 列數
    let ncols = 3; // 行數
    let seed: u64 = 42; // 你可以改這個 seed
    let mut rng = StdRng::seed_from_u64(seed);
    let matrix = gen_normal_matrix(nrows, ncols, &mut rng);
    println!("隨機標準常態分佈矩陣:");
    println!("{}", &matrix);

    let delta_t = 1.0; // 時間間隔
    // 1x1 起始點
    let start = DMatrix::<f64>::zeros(nrows, 1);
    let mut rng = StdRng::seed_from_u64(seed);
    let bm = brownian_motion_matrix(
        nrows,
        ncols,
        delta_t,
        CumsumOrder::ColumnWise,
        start,
        &mut rng,
    );
    println!("布朗運動矩陣:");
    println!("{}", &bm);
}
