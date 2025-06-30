//! Data storage module for eigenvalue simulations.
//!
//! This module provides functionality for running large-scale simulations
//! and storing eigenvalue data efficiently with resumable append-only writing.

pub mod append_writer; // 高效的追加寫入器，支援斷點續傳
mod config;
pub mod parallel_compute; // 並行計算引擎
mod simulation;

// Re-export the main API
pub use simulation::EigenvalueSimulation;
