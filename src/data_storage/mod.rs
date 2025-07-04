//! Data storage module for eigenvalue simulations.
//!
//! This module provides functionality for running large-scale simulations
//! and storing eigenvalue data efficiently with resumable append-only writing.

mod config;
pub(crate) mod file_format;
pub(crate) mod parallel_compute; // 並行計算引擎
pub(crate) mod progress;
pub(crate) mod reader;
pub(crate) mod simulation;
pub(crate) mod thread_manager;
pub(crate) mod uleb128; // ULEB128 編碼/解碼
pub(crate) mod writer;

// Re-export the main API
pub use simulation::EigenvalueSimulation;
