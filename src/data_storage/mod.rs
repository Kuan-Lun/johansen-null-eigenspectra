//! Data storage module for eigenvalue simulations.
//!
//! This module provides functionality for running large-scale simulations
//! and storing eigenvalue data efficiently with resumable append-only writing.

mod config;
pub mod file_format;
pub mod parallel_compute; // 並行計算引擎
pub mod progress;
pub mod reader;
mod simulation;
pub mod thread_manager;
pub mod uleb128; // ULEB128 編碼/解碼
pub mod writer;

// Re-export the main API
pub use simulation::EigenvalueSimulation;
