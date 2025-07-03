//! 檔案格式定義和相關工具函數
//!
//! 定義了特徵值檔案的二進制格式常數和計算函數

use super::config::{MAX_READ_BUFFER_SIZE, MIN_READ_BUFFER_SIZE};

/// 檔案格式常數
pub const MAGIC_HEADER: &[u8] = b"EIGENVALS_V5"; // 12 bytes
pub const EOF_MARKER: &[u8] = b"EOF_MARK"; // 8 bytes

/// 計算預期檔案大小以便預先配置磁碟空間
pub fn calculate_expected_file_size(num_runs: usize, eigenvalues_per_run: usize) -> u64 {
    let header = MAGIC_HEADER.len() as u64 + 1 + 1 + 4; // magic + model(1) + dim(1) + steps(4)
    let record_size = 4 + 1 + eigenvalues_per_run as u64 * 8; // seed: 4 bytes (u32), count: 1 byte (u8)
    let metadata = EOF_MARKER.len() as u64 + 8 + 1; // eof_marker + total_count + eigenvalues_per_run(u8)
    header + record_size * num_runs as u64 + metadata
}

/// 根據檔案大小計算最佳讀取緩衝區大小
pub fn calculate_read_buffer_size(file_size: u64) -> usize {
    // 根據檔案大小調整緩衝區：
    // - 小檔案 (< 1MB): 64KB
    // - 中檔案 (1MB - 100MB): 檔案大小的 1/8，最多 4MB
    // - 大檔案 (> 100MB): 16MB

    const ONE_MB: u64 = 1024 * 1024;
    const HUNDRED_MB: u64 = 100 * ONE_MB;

    if file_size < ONE_MB {
        MIN_READ_BUFFER_SIZE
    } else if file_size < HUNDRED_MB {
        // 使用檔案大小的 1/8 作為緩衝區，但不超過 4MB
        let buffer_size = (file_size / 8) as usize;
        buffer_size.clamp(MIN_READ_BUFFER_SIZE, 4 * 1024 * 1024)
    } else {
        MAX_READ_BUFFER_SIZE
    }
}
