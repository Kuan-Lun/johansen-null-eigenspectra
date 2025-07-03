//! Johansen 協整檢驗的統計計算模組
//!
//! 本模組包含用於計算 Johansen 協整檢驗在不同模型下的漸進虛無統計分布的函數。
//! 主要功能包括：
//! - 計算特徵值分布
//! - 構造不同模型下的 F 矩陣
//! - 廣義特徵值問題求解

use crate::johansen_models::JohansenModel;
use crate::matrix_utils::sum_of_outer_products;
use nalgebra::DMatrix;
use nalgebra_lapack::GeneralizedEigen;

/// 計算 Johansen 測試在指定模型下的特徵值
///
/// # 參數
/// * `bm_previous` - 前一時間步的布朗運動矩陣
/// * `dbm` - 布朗運動的差分矩陣
/// * `delta_t` - 時間間隔
/// * `model` - Johansen 模型類型
///
/// # 返回值
/// 按降序排列的特徵值向量
fn calculate_eigenvalues_from_matrices(
    bm_previous: &DMatrix<f64>,
    dbm: &DMatrix<f64>,
    delta_t: f64,
    model: JohansenModel,
) -> Vec<f64> {
    let fm = construct_f_matrix(bm_previous, model);

    let sum_dbm_fm_outer_products = sum_of_outer_products(dbm, &fm);
    let sum_fm_fm_outer_products = sum_of_outer_products(&fm, &fm) * delta_t;

    let ge = GeneralizedEigen::new(
        sum_dbm_fm_outer_products.transpose() * sum_dbm_fm_outer_products,
        sum_fm_fm_outer_products,
    );

    let mut eigenvalues_real: Vec<f64> = ge
        .raw_eigenvalues()
        .iter()
        .map(|val| val.0.norm() / val.1)
        .collect();
    eigenvalues_real.sort_by(|a, b| b.partial_cmp(a).unwrap());
    eigenvalues_real
}

/// 計算 Johansen 測試在指定模型下的特徵值（從完整布朗運動矩陣）
///
/// # 參數
/// * `dim` - 維度
/// * `steps` - 時間步數
/// * `seed` - 隨機種子
/// * `model` - Johansen 模型類型
///
/// # 返回值
/// 按降序排列的特徵值向量
pub fn calculate_eigenvalues(
    dim: usize,
    steps: usize,
    seed: u32,
    model: JohansenModel,
) -> Vec<f64> {
    use crate::rng_matrix::{TimeAxisDirection, brownian_motion_matrix};

    // 將 u32 seed 轉換為 u64 以兼容底層 RNG
    let seed_u64 = seed as u64;

    let delta_t = 1.0 / (steps as f64);
    let bm = brownian_motion_matrix(
        dim,
        steps,
        delta_t,
        TimeAxisDirection::AlongColumns,
        DMatrix::<f64>::zeros(dim, 1),
        seed_u64,
    );

    let bm_current = bm.columns(1, steps);
    let bm_previous = bm.columns(0, steps);
    let dbm = &bm_current - &bm_previous;

    calculate_eigenvalues_from_matrices(&bm_previous.into_owned(), &dbm, delta_t, model)
}

/// 根據指定的 Johansen 模型構造 F 矩陣
///
/// # 參數
/// * `bm_previous` - 前一時間步的布朗運動矩陣
/// * `model` - Johansen 模型類型
///
/// # 返回值
/// 構造的 F 矩陣
///
/// # 模型說明
/// - `NoInterceptNoTrend`: 無常數項無趨勢項模型
/// - `InterceptNoTrendWithInterceptInCoint`: 有常數項無趨勢項且常數項在協整關係中
/// - `InterceptNoTrendNoInterceptInCoint`: 有常數項無趨勢項但常數項不在協整關係中
/// - `InterceptTrendWithTrendInCoint`: 有常數項有趨勢項且趨勢項在協整關係中
/// - `InterceptTrendNoTrendInCoint`: 有常數項有趨勢項但趨勢項不在協整關係中
fn construct_f_matrix(bm_previous: &DMatrix<f64>, model: JohansenModel) -> DMatrix<f64> {
    let (rows, cols) = bm_previous.shape();

    match model {
        JohansenModel::NoInterceptNoTrend => bm_previous.clone(),

        JohansenModel::InterceptNoTrendWithInterceptInCoint => {
            let mut fm = DMatrix::<f64>::zeros(rows + 1, cols);
            fm.rows_mut(0, rows).copy_from(bm_previous);
            fm.rows_mut(rows, 1).fill(1.0);
            fm
        }

        JohansenModel::InterceptNoTrendNoInterceptInCoint => {
            let t = cols as f64;
            let mut x_demean = bm_previous.rows(0, rows - 1).clone_owned();

            // 對每一行進行去均值處理
            for mut row in x_demean.row_iter_mut() {
                let mean = row.mean();
                for val in row.iter_mut() {
                    *val -= mean;
                }
            }

            // 構造時間趨勢項
            let mut y = DMatrix::<f64>::zeros(1, cols);
            for (i, val) in y.iter_mut().enumerate() {
                *val = (i + 1) as f64 / t - 0.5;
            }

            let mut combined = DMatrix::<f64>::zeros(rows, cols);
            combined.rows_mut(0, rows - 1).copy_from(&x_demean);
            combined.rows_mut(rows - 1, 1).copy_from(&y);

            combined
        }

        JohansenModel::InterceptTrendWithTrendInCoint => {
            let mut x = bm_previous.clone_owned();
            let t = cols as f64;

            // 對所有行進行去均值處理
            for mut row in x.row_iter_mut() {
                let mean = row.mean();
                for val in row.iter_mut() {
                    *val -= mean;
                }
            }

            // 構造時間趨勢項
            let mut y = DMatrix::<f64>::zeros(1, cols);
            for (i, val) in y.iter_mut().enumerate() {
                *val = (i + 1) as f64 / t - 0.5;
            }

            let mut fm = DMatrix::<f64>::zeros(rows + 1, cols);
            fm.rows_mut(0, rows).copy_from(&x);
            fm.rows_mut(rows, 1).copy_from(&y);
            fm
        }

        JohansenModel::InterceptTrendNoTrendInCoint => {
            let t = cols as f64;

            // 構造時間趨勢項
            let mut y = DMatrix::<f64>::zeros(1, cols);
            for (i, val) in y.iter_mut().enumerate() {
                *val = (i + 1) as f64 / t;
            }

            let x_part = bm_previous.rows(0, rows - 1);
            let mut y_squared = y.clone();
            for val in y_squared.iter_mut() {
                *val = val.powi(2);
            }

            // 構造包含二次項的矩陣
            let mut x_with_y2 = DMatrix::<f64>::zeros(rows, cols);
            x_with_y2.rows_mut(0, rows - 1).copy_from(&x_part);
            x_with_y2.rows_mut(rows - 1, 1).copy_from(&y_squared);

            // 構造投影矩陣的基
            let mut z = DMatrix::<f64>::zeros(2, cols);
            z.rows_mut(0, 1).fill(1.0);
            z.rows_mut(1, 1).copy_from(&y);

            // 計算投影並得到殘差
            let zt = z.transpose();
            let zzt = &z * &zt;
            let zzt_inv = zzt.try_inverse().unwrap();
            let projection = &x_with_y2 * &zt * &zzt_inv * &z;
            let fm = &x_with_y2 - &projection;
            fm
        }
    }
}
