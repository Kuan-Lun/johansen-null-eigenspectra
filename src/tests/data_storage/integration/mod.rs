pub use crate::data_storage::EigenvalueSimulation;
pub use crate::johansen_models::JohansenModel;

mod basic_api;
mod data_integrity;
mod edge_cases;
mod filename_consistency;
mod helpers;
mod incompatible_file;
mod load_partial;
mod multiple_models;
mod resumable;

pub use helpers::*;

/// 建立寫入系統暫存目錄的測試模擬配置，不觸碰專案的 data/ 資料夾。
/// 每個測試以自己的 `subdir` 隔離，避免並行測試互相干擾。
pub fn test_sim(
    subdir: &str,
    model: JohansenModel,
    dim: usize,
    steps: usize,
    num_runs: usize,
) -> EigenvalueSimulation {
    EigenvalueSimulation::new(model, dim, steps, num_runs)
        .unwrap()
        .with_data_dir(
            std::env::temp_dir()
                .join("johansen_integration_tests")
                .join(subdir),
        )
}
