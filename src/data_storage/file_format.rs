//! 檔案格式定義和相關工具函數
//!
//! 定義了特徵值檔案的二進制格式常數和計算函數

use super::config::{MAX_READ_BUFFER_SIZE, MIN_READ_BUFFER_SIZE};

/// 檔案格式常數
pub const MAGIC_HEADER: &[u8] = b"EIGENVALS_V6"; // 12 bytes
pub const EOF_MARKER: &[u8] = b"EOF_MARK"; // 8 bytes

/// 計算預期檔案大小以便預先配置磁碟空間
///
/// 注意：由於 seed 現在使用 ULEB128 編碼，檔案大小會因 seed 值而異
/// 這個函數使用公式精確計算 1 到 num_runs 範圍內所有 seed 的編碼總大小
pub fn calculate_expected_file_size(num_runs: usize, eigenvalues_per_run: usize) -> u64 {
    let header = MAGIC_HEADER.len() as u64 + 1 + 1 + 4; // magic + model(1) + dim(1) + steps(4)

    // 直接計算所有 seed 的 ULEB128 編碼總大小
    let total_seed_bytes = calculate_total_uleb128_size(num_runs as u32);

    let eigenvalues_total_bytes = eigenvalues_per_run as u64 * 8 * num_runs as u64; // 每個 eigenvalue 8 bytes
    let eigenvalue_counts_bytes = num_runs as u64; // 每個記錄的 eigenvalue count (1 byte)
    let metadata = EOF_MARKER.len() as u64 + 8 + 1; // eof_marker + total_count + eigenvalues_per_run(u8)

    header + total_seed_bytes + eigenvalue_counts_bytes + eigenvalues_total_bytes + metadata
}

/// 計算 1 到 max_value 範圍內所有 ULEB128 編碼的總大小
///
/// ULEB128 編碼規律：
/// - 1-127: 1 byte
/// - 128-16383: 2 bytes  
/// - 16384-2097151: 3 bytes
/// - 2097152-268435455: 4 bytes
/// - 268435456-4294967295: 5 bytes
fn calculate_total_uleb128_size(max_value: u32) -> u64 {
    if max_value == 0 {
        return 1; // 至少需要編碼 0
    }

    let mut total_bytes = 0u64;
    let mut remaining = max_value;

    // 1 byte: 1-127
    if remaining >= 1 {
        let count = remaining.min(127);
        total_bytes += count as u64;
        remaining = remaining.saturating_sub(127);
    }

    // 2 bytes: 128-16383
    if remaining > 0 {
        let count = remaining.min(16383 - 127);
        total_bytes += count as u64 * 2;
        remaining = remaining.saturating_sub(16383 - 127);
    }

    // 3 bytes: 16384-2097151
    if remaining > 0 {
        let count = remaining.min(2097151 - 16383);
        total_bytes += count as u64 * 3;
        remaining = remaining.saturating_sub(2097151 - 16383);
    }

    // 4 bytes: 2097152-268435455
    if remaining > 0 {
        let count = remaining.min(268435455 - 2097151);
        total_bytes += count as u64 * 4;
        remaining = remaining.saturating_sub(268435455 - 2097151);
    }

    // 5 bytes: 268435456-4294967295
    if remaining > 0 {
        total_bytes += remaining as u64 * 5;
    }

    total_bytes
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
