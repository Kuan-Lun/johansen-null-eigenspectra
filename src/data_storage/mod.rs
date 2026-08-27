//! Data storage module for eigenvalue simulations.
//!
//! V7 定長槽位格式：seed 由槽位位置隱含，不寫入檔案。
//! 計算期間以 CRC sidecar 逐筆驗證（支援斷點續傳與 torn-write 偵測），
//! 完成後轉為 SHA-256 整檔雜湊驗證。

mod config;
pub(crate) mod crc32;
pub(crate) mod file_format;
pub(crate) mod parallel_compute; // 並行計算引擎
pub(crate) mod reader;
pub(crate) mod simulation;
pub(crate) mod writer;

// Re-export the main API
pub use parallel_compute::Progress;
pub use simulation::EigenvalueSimulation;
