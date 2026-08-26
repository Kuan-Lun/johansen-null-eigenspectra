//! Error types for the data storage module.
//!
//! 以結構化的錯誤變體取代舊版以字串比對辨識錯誤的做法，
//! 讓呼叫端能依錯誤種類決定「刪檔重建」或「直接失敗」。

use std::fmt;

/// Errors produced by the eigenvalue data storage layer.
#[derive(Debug)]
pub enum StorageError {
    /// Underlying I/O failure.
    Io(std::io::Error),
    /// The magic header does not identify a V7 data file.
    MagicMismatch,
    /// A file header field conflicts with the requested configuration.
    ParamMismatch {
        field: &'static str,
        file_value: u64,
        expected: u64,
    },
    /// The file structure is inconsistent with its header.
    Corrupted(String),
    /// Fewer valid records than the requested number of runs.
    Incomplete { found: usize, expected: usize },
    /// SHA-256 verification failed or the hash file is unusable.
    HashMismatch(String),
    /// The caller passed arguments that violate the file's configuration.
    InvalidInput(String),
}

impl StorageError {
    /// 判斷此錯誤是否代表「既有檔案無法沿用，應刪除後重建」。
    /// 資料皆可由 seed 重算，因此重建只是損失計算時間而非資料。
    pub fn is_recreatable(&self) -> bool {
        matches!(
            self,
            StorageError::MagicMismatch
                | StorageError::ParamMismatch { .. }
                | StorageError::Corrupted(_)
        )
    }
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::Io(e) => write!(f, "I/O error: {e}"),
            StorageError::MagicMismatch => {
                write!(f, "File format error: magic header mismatch")
            }
            StorageError::ParamMismatch {
                field,
                file_value,
                expected,
            } => write!(
                f,
                "{field} mismatch: file has {field} {file_value}, expected {expected}"
            ),
            StorageError::Corrupted(msg) => write!(f, "Corrupted data file: {msg}"),
            StorageError::Incomplete { found, expected } => write!(
                f,
                "Data count mismatch: expected {expected} records, found {found} records. \
                 This indicates incomplete simulation data.\n\
                 Solutions:\n\
                 1. Use `read_all_data()` to read available data ({found} records)\n\
                 2. Run `run_simulation()` to complete the simulation\n\
                 3. Create a new EigenvalueSimulation with num_runs={found} to match existing data"
            ),
            StorageError::HashMismatch(msg) => {
                write!(f, "SHA-256 verification failed: {msg}")
            }
            StorageError::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::Io(err)
    }
}

impl From<StorageError> for std::io::Error {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::Io(e) => e,
            other => std::io::Error::new(std::io::ErrorKind::InvalidData, other.to_string()),
        }
    }
}
