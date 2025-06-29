//! Data storage module for eigenvalue simulations.
//!
//! This module provides functionality for running large-scale simulations
//! and storing eigenvalue data efficiently.

pub mod binary_io; // 設為公開，允許測試
mod parallel_compute;
mod resumable_writer;
mod simulation;
mod stream_writer;

// Re-export the main API
pub use simulation::EigenvalueSimulation;
