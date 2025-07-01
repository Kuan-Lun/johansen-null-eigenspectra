use nalgebra::DMatrix;

#[derive(Debug, Clone, Copy)]
pub enum CumsumOrder {
    #[allow(dead_code)]
    ColumnMajor, // 全矩陣，依照 column-major 順序累加
    #[allow(dead_code)]
    RowMajor, // 全矩陣，依照 row-major 順序累加
    ColumnWise, // 每一欄各自累加
    RowWise,    // 每一列各自累加
}

pub fn dmatrix_cumsum(matrix: &DMatrix<f64>, order: CumsumOrder) -> DMatrix<f64> {
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
        CumsumOrder::ColumnWise => {
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

pub fn sum_of_outer_products(a: &DMatrix<f64>, b: &DMatrix<f64>) -> DMatrix<f64> {
    use rayon::prelude::*;

    let (a_nrows, n_samples) = a.shape();
    let b_nrows = b.nrows();
    debug_assert_eq!(b.ncols(), n_samples);

    (0..n_samples)
        .into_par_iter()
        .map(|i| {
            let col1 = a.column(i);
            let col2 = b.column(i);
            &col1 * &col2.transpose()
        })
        .reduce(
            || DMatrix::<f64>::zeros(a_nrows, b_nrows), // 初始值：a的行數 × b的行數
            |acc, outer_product| acc + outer_product,   // 累加操作
        )
}
