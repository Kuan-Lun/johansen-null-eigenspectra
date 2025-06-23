use nalgebra::DMatrix;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Distribution, StandardNormal};

fn gen_normal_matrix<R: rand::Rng + ?Sized>(n: usize, m: usize, rng: &mut R) -> DMatrix<f64> {
    let data: Vec<f64> = (0..n * m).map(|_| StandardNormal.sample(rng)).collect();
    DMatrix::from_vec(n, m, data)
}

#[derive(Debug, Clone, Copy)]
enum CumsumOrder {
    ColumnMajor, // 列主序，依照列的順序累加
    RowMajor,    // 行主序，依照行的順序累加
    ColWise,     // 每一欄各自累加
    RowWise,     // 每一列各自累加
}

fn dmatrix_cumsum(matrix: &DMatrix<f64>, order: CumsumOrder) -> DMatrix<f64> {
    match order {
        CumsumOrder::ColumnMajor => {
            let data = matrix.iter().cloned();
            let mut cumsum_vec = Vec::with_capacity(matrix.len());
            let mut acc = 0.0;
            for v in data {
                acc += v;
                cumsum_vec.push(acc);
            }
            DMatrix::from_vec(matrix.nrows(), matrix.ncols(), cumsum_vec)
        }
        CumsumOrder::RowMajor => {
            let nrows = matrix.nrows();
            let ncols = matrix.ncols();
            let mut cumsum_vec = Vec::with_capacity(matrix.len());
            let mut acc = 0.0;
            for row in 0..nrows {
                for col in 0..ncols {
                    acc += matrix[(row, col)];
                    cumsum_vec.push(acc);
                }
            }
            DMatrix::from_row_slice(nrows, ncols, &cumsum_vec)
        }
        CumsumOrder::ColWise => {
            let nrows = matrix.nrows();
            let ncols = matrix.ncols();
            let mut cumsum_mat = DMatrix::zeros(nrows, ncols);
            for col in 0..ncols {
                let mut acc = 0.0;
                for row in 0..nrows {
                    acc += matrix[(row, col)];
                    cumsum_mat[(row, col)] = acc;
                }
            }
            cumsum_mat
        }
        CumsumOrder::RowWise => {
            let nrows = matrix.nrows();
            let ncols = matrix.ncols();
            let mut cumsum_mat = DMatrix::zeros(nrows, ncols);
            for row in 0..nrows {
                let mut acc = 0.0;
                for col in 0..ncols {
                    acc += matrix[(row, col)];
                    cumsum_mat[(row, col)] = acc;
                }
            }
            cumsum_mat
        }
    }
}

fn main() {
    let n = 2; // 列數
    let m = 3; // 行數
    let seed: u64 = 42; // 你可以改這個 seed
    let mut rng = StdRng::seed_from_u64(seed);
    let matrix = gen_normal_matrix(n, m, &mut rng);
    println!("隨機標準常態分佈矩陣:");
    println!("{}", &matrix);
    let cumsum_col = dmatrix_cumsum(&matrix, CumsumOrder::ColumnMajor);
    println!("\nColumn-major cumsum:");
    println!("{}", cumsum_col);
    let cumsum_row = dmatrix_cumsum(&matrix, CumsumOrder::RowMajor);
    println!("\nRow-major cumsum:");
    println!("{}", cumsum_row);
    let cumsum_col = dmatrix_cumsum(&matrix, CumsumOrder::ColWise);
    println!("\nColumn-wise cumsum:");
    println!("{}", cumsum_col);
    let cumsum_row = dmatrix_cumsum(&matrix, CumsumOrder::RowWise);
    println!("\nRow-wise cumsum:");
    println!("{}", cumsum_row);
    println!("隨機標準常態分佈矩陣:");
    println!("{}", &matrix);
}
