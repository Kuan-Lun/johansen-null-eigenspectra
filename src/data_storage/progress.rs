//! 進度管理和斷點續傳功能
//!
//! 提供 seed 管理、進度檢查和斷點續傳相關功能

use std::collections::HashSet;
use std::path::Path;

use super::reader::read_append_file;

/// 檢查檔案進度（追加格式）並驗證參數匹配
pub fn check_append_progress<P: AsRef<Path>>(
    path: P,
    expected_model: u8,
    expected_dim: u8,
    expected_steps: u32,
) -> std::io::Result<(usize, Vec<u32>)> {
    if !path.as_ref().exists() {
        return Ok((0, Vec::new()));
    }

    match read_append_file(&path) {
        Ok((data, file_model, file_dim, file_steps)) => {
            // 驗證參數是否匹配
            if file_model != expected_model {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Model mismatch: file has model {file_model}, expected {expected_model}"
                    ),
                ));
            }
            if file_dim != expected_dim {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Dimension mismatch: file has dim {file_dim}, expected {expected_dim}"),
                ));
            }
            if file_steps != expected_steps {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Steps mismatch: file has steps {file_steps}, expected {expected_steps}"
                    ),
                ));
            }

            let completed_runs = data.len();
            let completed_seeds: Vec<u32> = data.iter().map(|(seed, _)| *seed).collect();
            Ok((completed_runs, completed_seeds))
        }
        Err(_) => Ok((0, Vec::new())), // 檔案損壞或無法讀取，重新開始
    }
}

/// 獲取尚未完成的seed列表
pub fn get_remaining_seeds(total_runs: usize, completed_seeds: &[u32]) -> Vec<u32> {
    let completed_set: HashSet<u32> = completed_seeds.iter().copied().collect();
    (1..=total_runs as u32)
        .filter(|seed| !completed_set.contains(seed))
        .collect()
}
