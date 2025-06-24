use crate::matrix_utils::{CumsumOrder, dmatrix_cumsum};

use nalgebra::DMatrix;
use rand_distr::{Distribution, StandardNormal};

pub fn gen_normal_matrix<R: rand::Rng + ?Sized>(
    nrows: usize,
    ncols: usize,
    rng: &mut R,
) -> DMatrix<f64> {
    let data: Vec<f64> = (0..nrows * ncols)
        .map(|_| StandardNormal.sample(rng))
        .collect();
    DMatrix::from_vec(nrows, ncols, data)
}

pub fn brownian_motion_matrix<R: rand::Rng + ?Sized>(
    nrows: usize,
    ncols: usize,
    delta_t: f64,
    order: CumsumOrder,
    start: DMatrix<f64>,
    rng: &mut R,
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

    fn make_z_matrix<R: rand::Rng + ?Sized>(
        order: CumsumOrder,
        nrows: usize,
        ncols: usize,
        start: &DMatrix<f64>,
        rng: &mut R,
    ) -> DMatrix<f64> {
        match order {
            CumsumOrder::ColumnMajor | CumsumOrder::RowMajor => {
                let mut z = gen_normal_matrix(nrows, ncols, rng);
                z[(0, 0)] = start[(0, 0)];
                z
            }
            CumsumOrder::ColumnWise => {
                let mut cols: Vec<_> = start.column_iter().collect();
                let gen_mat = gen_normal_matrix(nrows, ncols - 1, rng);
                cols.extend(gen_mat.column_iter());
                DMatrix::from_columns(&cols)
            }
            CumsumOrder::RowWise => {
                let mut rows: Vec<_> = start.row_iter().collect();
                let gen_mat = gen_normal_matrix(nrows - 1, ncols, rng);
                rows.extend(gen_mat.row_iter());
                DMatrix::from_rows(&rows)
            }
        }
    }

    #[cfg(debug_assertions)]
    {
        check_start_shape(order, &start, nrows, ncols);
    }

    let z = make_z_matrix(order, nrows, ncols, &start, rng);
    let sqrt_dt = delta_t.sqrt();
    let scaled = z.map(|v| v * sqrt_dt);
    dmatrix_cumsum(&scaled, order)
}
