//! The crate-wide error type.
//!
//! 以結構化的錯誤變體取代以字串比對辨識錯誤的做法，
//! 讓呼叫端能依錯誤種類決定「刪檔重建」或「直接失敗」。

use std::fmt;

/// Errors produced by this crate.
///
/// 標記為 `#[non_exhaustive]`：未來新增變體不構成破壞性變更，
/// 呼叫端 match 時需保留萬用分支。
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
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
    /// A configuration parameter falls outside the supported range.
    InvalidConfig(String),
}

impl Error {
    /// 判斷此錯誤是否代表「既有檔案無法沿用，應刪除後重建」。
    /// 資料皆可由 seed 重算，因此重建只是損失計算時間而非資料。
    /// 刪檔由呼叫端透過 `EigenvalueSimulation::delete_data()` 明確執行。
    pub fn is_recreatable(&self) -> bool {
        matches!(
            self,
            Error::MagicMismatch | Error::ParamMismatch { .. } | Error::Corrupted(_)
        )
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::MagicMismatch => {
                write!(f, "File format error: magic header mismatch")
            }
            Error::ParamMismatch {
                field,
                file_value,
                expected,
            } => write!(
                f,
                "{field} mismatch: file has {field} {file_value}, expected {expected}"
            ),
            Error::Corrupted(msg) => write!(f, "Corrupted data file: {msg}"),
            Error::Incomplete { found, expected } => write!(
                f,
                "Data count mismatch: expected {expected} records, found {found} records. \
                 This indicates incomplete simulation data.\n\
                 Solutions:\n\
                 1. Use `load_partial()` to read available data ({found} records)\n\
                 2. Run `run()` to complete the simulation"
            ),
            Error::HashMismatch(msg) => {
                write!(f, "SHA-256 verification failed: {msg}")
            }
            Error::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            Error::InvalidConfig(msg) => write!(f, "Invalid configuration: {msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}
