//! Monte Carlo simulation of the null eigenvalue spectra of the Johansen
//! cointegration test.
//!
//! 公開 API 以 [`EigenvalueSimulation`] 為入口：`run()` 計算（支援斷點
//! 續傳）、`load()` / `load_partial()` 載入為 [`SimulationData`]、再以
//! [`SimulationData::percentiles`] 計算檢定統計量的百分位數。
//! 所有可失敗操作回傳 [`Error`]。使用範例見 `LIBRARY_USAGE.md`。

pub(crate) mod data_storage;
pub(crate) mod error;
pub(crate) mod johansen_models;
pub(crate) mod johansen_statistics;
pub(crate) mod matrix_utils;
pub(crate) mod rng_matrix;
pub(crate) mod simulation_data;

#[cfg(test)]
mod tests;

// Re-export the public API
pub use data_storage::{EigenvalueSimulation, Progress};
pub use error::Error;
pub use johansen_models::JohansenModel;
pub use simulation_data::{SimulationData, SimulationRecord, Statistic};
