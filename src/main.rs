mod matrix_utils;

use nalgebra::DMatrix;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Distribution, StandardNormal};

fn gen_normal_matrix<R: rand::Rng + ?Sized>(n: usize, m: usize, rng: &mut R) -> DMatrix<f64> {
    let data: Vec<f64> = (0..n * m).map(|_| StandardNormal.sample(rng)).collect();
    DMatrix::from_vec(n, m, data)
}

fn main() {
    let n = 2; // 列數
    let m = 3; // 行數
    let seed: u64 = 42; // 你可以改這個 seed
    let mut rng = StdRng::seed_from_u64(seed);
    let matrix = gen_normal_matrix(n, m, &mut rng);
    println!("隨機標準常態分佈矩陣:");
    println!("{}", &matrix);
}
