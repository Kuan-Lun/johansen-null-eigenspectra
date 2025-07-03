//! 檔案讀取器 - 高效的特徵值檔案讀取
//!
//! 實現了帶有元數據的快速讀取和掃描式讀取

use crate::display_utils::format_number_with_commas;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use super::file_format::{EOF_MARKER, MAGIC_HEADER, calculate_read_buffer_size};
use super::uleb128;

/// 檔案讀取結果類型別名
pub type FileReadResult = std::io::Result<(Vec<(u32, Vec<f64>)>, u8, u8, u32)>;

/// 從 BufReader 讀取 ULEB128 編碼的 u32 值
fn read_uleb128(reader: &mut BufReader<File>) -> std::io::Result<u32> {
    uleb128::read_from_reader(reader)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
}

/// 讀取追加格式的檔案
pub fn read_append_file<P: AsRef<Path>>(path: P) -> FileReadResult {
    let file = File::open(&path)?;
    let file_size = file.metadata()?.len();

    // 根據檔案大小計算最佳緩衝區大小
    let buffer_size = calculate_read_buffer_size(file_size);
    let mut reader = BufReader::with_capacity(buffer_size, file);

    // 檢查魔術標頭
    let mut magic_buf = [0u8; 12];
    reader.read_exact(&mut magic_buf)?;
    if magic_buf != MAGIC_HEADER {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "File format error: magic header mismatch",
        ));
    }

    // 讀取檔案參數
    let mut model_buf = [0u8; 1];
    let mut dim_buf = [0u8; 1];
    let mut steps_buf = [0u8; 4];

    reader.read_exact(&mut model_buf)?;
    reader.read_exact(&mut dim_buf)?;
    reader.read_exact(&mut steps_buf)?;

    let model = u8::from_le_bytes(model_buf);
    let dim = u8::from_le_bytes(dim_buf);
    let steps = u32::from_le_bytes(steps_buf);

    // 嘗試從檔案末尾讀取元數據
    let file_len = reader.get_ref().metadata()?.len();
    if file_len < 18 + 8 + 8 + 1 {
        // magic(12) + model(1) + dim(1) + steps(4) + eof_marker(8) + count(8) + eigenvalues_per_run(1)
        return Ok((Vec::new(), model, dim, steps)); // 檔案太小，可能是空檔案
    }

    // 檢查是否有完整的結束標記
    let metadata = read_file_metadata(&mut reader, file_len)?;

    let data = if let Some((total_count, eigenvalues_per_run)) = metadata {
        // 有完整的結束標記，使用快速讀取
        read_with_metadata(&mut reader, total_count, eigenvalues_per_run)?
    } else {
        // 沒有結束標記，掃描式讀取（用於未完成的檔案）
        scan_read_data(&mut reader)?
    };

    Ok((data, model, dim, steps))
}

/// 嘗試從檔案末尾讀取元數據
fn read_file_metadata(
    reader: &mut BufReader<File>,
    file_len: u64,
) -> std::io::Result<Option<(usize, usize)>> {
    // 定位到檔案末尾的元數據位置
    let metadata_offset = file_len - 8 - 1; // count + eigenvalues_per_run(u8)
    reader.seek(SeekFrom::Start(metadata_offset - 8))?; // 包括 EOF_MARKER

    // 檢查 EOF 標記
    let mut eof_buf = [0u8; 8];
    reader.read_exact(&mut eof_buf)?;
    if eof_buf != EOF_MARKER {
        return Ok(None); // 沒有有效的結束標記
    }

    // 讀取總數和特徵值數量
    let mut count_buf = [0u8; 8];
    let mut eigenvalues_buf = [0u8; 1]; // 改為 1 byte

    reader.read_exact(&mut count_buf)?;
    reader.read_exact(&mut eigenvalues_buf)?;

    let total_count = u64::from_le_bytes(count_buf) as usize;
    let eigenvalues_per_run = u8::from_le_bytes(eigenvalues_buf) as usize;

    Ok(Some((total_count, eigenvalues_per_run)))
}

/// 使用元數據快速讀取
fn read_with_metadata(
    reader: &mut BufReader<File>,
    total_count: usize,
    eigenvalues_per_run: usize,
) -> std::io::Result<Vec<(u32, Vec<f64>)>> {
    // 回到數據開始位置
    reader.seek(SeekFrom::Start(18))?; // 跳過魔術標頭(12) + model(1) + dim(1) + steps(4)

    let mut data = Vec::with_capacity(total_count);

    for _ in 0..total_count {
        // 讀取 ULEB128 編碼的 seed
        let seed = read_uleb128(reader)?;

        let mut count_buf = [0u8; 1]; // 1 byte (u8)
        reader.read_exact(&mut count_buf)?;
        let eigenvalue_count_u8 = u8::from_le_bytes(count_buf);
        let eigenvalue_count = eigenvalue_count_u8 as usize;

        // 驗證 eigenvalue_count 在合理範圍內（雖然 u8 已經限制了範圍）
        if eigenvalue_count == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid eigenvalue count: cannot be zero",
            ));
        }

        if eigenvalue_count != eigenvalues_per_run {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Eigenvalue count mismatch: expected {}, actual {}",
                    format_number_with_commas(eigenvalues_per_run),
                    format_number_with_commas(eigenvalue_count)
                ),
            ));
        }

        let mut eigenvalues = Vec::with_capacity(eigenvalue_count);
        for _ in 0..eigenvalue_count {
            let mut val_buf = [0u8; 8];
            reader.read_exact(&mut val_buf)?;
            eigenvalues.push(f64::from_le_bytes(val_buf));
        }

        data.push((seed, eigenvalues));
    }

    Ok(data)
}

/// 掃描式讀取（用於沒有結束標記的檔案）
fn scan_read_data(reader: &mut BufReader<File>) -> std::io::Result<Vec<(u32, Vec<f64>)>> {
    // 回到數據開始位置
    reader.seek(SeekFrom::Start(18))?; // 跳過魔術標頭(12) + model(1) + dim(1) + steps(4)

    let mut data = Vec::new();

    #[allow(clippy::while_let_loop)]
    loop {
        // 嘗試讀取 ULEB128 編碼的 seed
        let seed = match read_uleb128(reader) {
            Ok(s) => s,
            Err(_) => break, // 到達檔案末尾或遇到錯誤
        };

        // 檢查是否遇到 EOF 標記
        // 由於使用 ULEB128，需要在讀取 eigenvalue count 後檢查 EOF
        let mut count_buf = [0u8; 1]; // 1 byte (u8)
        if reader.read_exact(&mut count_buf).is_err() {
            break; // 不完整的數據塊
        }

        let eigenvalue_count_u8 = u8::from_le_bytes(count_buf);
        let eigenvalue_count = eigenvalue_count_u8 as usize;

        // 檢查特徵值數量是否合理（u8 已經限制在 0-255 範圍內）
        if eigenvalue_count == 0 {
            // 零計數可能表示到達了預分配的空白區域
            // 檢查接下來是否是 EOF_MARKER
            let current_pos = reader.stream_position()?;
            let mut potential_eof = vec![0u8; EOF_MARKER.len()];
            if reader.read_exact(&mut potential_eof).is_ok() && potential_eof == EOF_MARKER {
                break; // 確認遇到 EOF 標記
            } else {
                // 回退並繼續，因為可能只是一個零計數的有效記錄
                reader.seek(SeekFrom::Start(current_pos))?;
            }
        }

        // 讀取特徵值
        let mut eigenvalues = Vec::with_capacity(eigenvalue_count);
        let mut read_complete = true;

        for _ in 0..eigenvalue_count {
            let mut val_buf = [0u8; 8];
            if reader.read_exact(&mut val_buf).is_err() {
                read_complete = false;
                break;
            }
            eigenvalues.push(f64::from_le_bytes(val_buf));
        }

        if !read_complete {
            break; // 不完整的特徵值數據
        }

        data.push((seed, eigenvalues));
    }

    Ok(data)
}
