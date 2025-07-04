pub(crate) mod data_storage;
pub(crate) mod display_utils;
pub(crate) mod johansen_models;
pub(crate) mod johansen_statistics;
pub(crate) mod matrix_utils;
pub(crate) mod rng_matrix;
mod simulation_analyzers;

// Re-export the main API
pub use data_storage::EigenvalueSimulation;
pub use johansen_models::JohansenModel;
