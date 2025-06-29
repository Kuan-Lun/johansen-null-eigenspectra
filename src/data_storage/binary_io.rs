//! 二進制文件 I/O 操作
//!
//! 提供讀取和寫入特徵值數據的二進制格式處理功能。

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// 從指定路徑讀取二進制格式的特徵值數據
///
/// # 文件格式
/// - 8 bytes: 總運行次數 (u64, little-endian)
/// - 8 bytes: 每次運行的特徵值數量 (u64, little-endian)
/// - 對於每次運行：
///   - 8 bytes: seed (u64, little-endian)
///   - N * 8 bytes: 特徵值 (f64, little-endian)
///
/// # 參數
/// - `path`: 二進制文件路徑
///
/// # 返回值
/// 返回 `Vec<(seed, eigenvalues)>` 其中：
/// - `seed`: 隨機數種子
/// - `eigenvalues`: 對應的特徵值向量
///
/// # 注意
/// 文件中的數據可能無序（因為並行計算的結果按完成順序寫入）
/// 如果需要有序數據，調用者應該自行排序
pub fn read_binary_file<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<(u64, Vec<f64>)>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut buffer = [0u8; 8];

    // 讀取維度信息
    reader.read_exact(&mut buffer)?;
    let num_runs = u64::from_le_bytes(buffer) as usize;

    if num_runs == 0 {
        return Ok(Vec::new());
    }

    reader.read_exact(&mut buffer)?;
    let eigenvalues_per_run = u64::from_le_bytes(buffer) as usize;

    let mut data = Vec::with_capacity(num_runs);

    // 讀取所有數據
    for _ in 0..num_runs {
        // 讀取seed
        reader.read_exact(&mut buffer)?;
        let seed = u64::from_le_bytes(buffer);

        // 讀取特徵值
        let mut eigenvalues = Vec::with_capacity(eigenvalues_per_run);
        for _ in 0..eigenvalues_per_run {
            reader.read_exact(&mut buffer)?;
            eigenvalues.push(f64::from_le_bytes(buffer));
        }
        data.push((seed, eigenvalues));
    }

    Ok(data)
}
