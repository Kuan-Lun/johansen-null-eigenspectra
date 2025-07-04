//! ULEB128 (Unsigned Little Endian Base 128) 編碼/解碼實現
//!
//! ULEB128 是一種變長整數編碼方式，每個位元組的最高位用作延續位，低7位存儲數據。
//! 這種編碼對於小數值非常高效，因為它們只需要較少的位元組。
//!
//! # 編碼規則
//! - 每個位元組使用 7 位存儲數據，1 位作為延續標誌
//! - 延續位 (MSB) 為 1 表示還有更多位元組，為 0 表示這是最後一個位元組
//! - 數據按小端序存儲（最低有效位元組在前）
//!
//! # 範例
//! 使用 encode 和 decode 函數進行 ULEB128 編碼和解碼：
//! - encode(300) 將返回 [0xAC, 0x02]
//! - decode([0xAC, 0x02]) 將返回 (300, 2)

// 這是一個內部模塊，僅供 crate 內部使用
#![allow(dead_code)]

use std::fs::File;
use std::io::{BufReader, Read};

/// ULEB128 編碼錯誤類型
#[derive(Debug, Clone, PartialEq)]
pub enum Uleb128Error {
    /// 數值對於 u32 來說太大
    ValueTooLarge,
    /// 編碼不完整（意外結束）
    IncompleteEncoding,
    /// 編碼太長（超過 u32 的最大可能長度）
    EncodingTooLong,
    /// IO 錯誤
    IoError(String),
}

impl std::fmt::Display for Uleb128Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Uleb128Error::ValueTooLarge => write!(f, "ULEB128 value too large for u32"),
            Uleb128Error::IncompleteEncoding => write!(f, "Incomplete ULEB128 encoding"),
            Uleb128Error::EncodingTooLong => write!(f, "ULEB128 encoding too long"),
            Uleb128Error::IoError(msg) => write!(f, "IO error: {msg}"),
        }
    }
}

impl std::error::Error for Uleb128Error {}

impl From<std::io::Error> for Uleb128Error {
    fn from(err: std::io::Error) -> Self {
        Uleb128Error::IoError(err.to_string())
    }
}

/// ULEB128 編碼一個 u32 值
///
/// # 參數
/// * `value` - 要編碼的 u32 值
///
/// # 返回值
/// 編碼後的位元組向量
///
/// # 範例
/// 編碼各種數值：
/// - encode(0) -> [0x00]
/// - encode(127) -> [0x7F]
/// - encode(128) -> [0x80, 0x01]
/// - encode(300) -> [0xAC, 0x02]
pub fn encode(mut value: u32) -> Vec<u8> {
    let mut result = Vec::new();

    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;

        if value != 0 {
            byte |= 0x80; // 設置延續位
        }

        result.push(byte);

        if value == 0 {
            break;
        }
    }

    result
}

/// ULEB128 解碼，從位元組切片讀取一個 u32 值
///
/// # 參數
/// * `bytes` - 包含 ULEB128 編碼數據的位元組切片
///
/// # 返回值
/// `Ok((解碼的值, 使用的位元組數))` 或 `Err(Uleb128Error)`
///
/// # 範例
/// 解碼各種數值：
/// - decode([0x00]) -> (0, 1)
/// - decode([0x7F]) -> (127, 1)
/// - decode([0x80, 0x01]) -> (128, 2)
/// - decode([0xAC, 0x02]) -> (300, 2)
pub fn decode(bytes: &[u8]) -> Result<(u32, usize), Uleb128Error> {
    let mut result = 0u32;
    let mut shift = 0;
    let mut bytes_read = 0;

    for &byte in bytes {
        bytes_read += 1;

        // 防止過長的編碼 (先檢查這個)
        if bytes_read > 5 {
            return Err(Uleb128Error::EncodingTooLong);
        }

        // 檢查是否會溢出 u32 (u32 最多需要 5 個 7-bit 組)
        if shift >= 32 {
            return Err(Uleb128Error::ValueTooLarge);
        }

        let value_bits = (byte & 0x7F) as u32;

        // 檢查是否會導致溢出
        if shift == 28 && value_bits > 0x0F {
            return Err(Uleb128Error::ValueTooLarge);
        }

        result |= value_bits << shift;
        shift += 7;

        // 如果沒有延續位，解碼完成
        if (byte & 0x80) == 0 {
            return Ok((result, bytes_read));
        }
    }

    Err(Uleb128Error::IncompleteEncoding)
}

/// 計算 ULEB128 編碼後的大小（位元組數）
///
/// # 參數
/// * `value` - 要計算編碼大小的 u32 值
///
/// # 返回值
/// 編碼後需要的位元組數
///
/// # 範例
/// 各種數值的編碼大小：
/// - encoded_size(0) -> 1
/// - encoded_size(127) -> 1
/// - encoded_size(128) -> 2
/// - encoded_size(16383) -> 2
/// - encoded_size(16384) -> 3
pub fn encoded_size(value: u32) -> usize {
    if value == 0 {
        return 1;
    }

    let mut size = 0;
    let mut v = value;
    while v > 0 {
        size += 1;
        v >>= 7;
    }
    size
}

/// 從讀取器中讀取 ULEB128 編碼的值
///
/// # 參數
/// * `reader` - 包含 ULEB128 編碼數據的檔案讀取器
///
/// # 返回值
/// `Ok(解碼的值)` 或 `Err(Uleb128Error)`
///
/// # 範例
/// 從檔案讀取 ULEB128 編碼的值：
/// ```
/// // 開啟檔案並創建讀取器
/// // let file = File::open("data.bin")?;
/// // let mut reader = BufReader::new(file);
/// // let value = read_from_reader(&mut reader)?;
/// ```
pub fn read_from_reader(reader: &mut BufReader<File>) -> Result<u32, Uleb128Error> {
    let mut result = 0u32;
    let mut shift = 0;
    let mut bytes_read = 0;

    loop {
        let mut byte_buf = [0u8; 1];
        reader.read_exact(&mut byte_buf)?;
        let byte = byte_buf[0];
        bytes_read += 1;

        // 檢查是否會溢出 u32
        if shift >= 32 {
            return Err(Uleb128Error::ValueTooLarge);
        }

        let value_bits = (byte & 0x7F) as u32;

        // 檢查是否會導致溢出
        if shift == 28 && value_bits > 0x0F {
            return Err(Uleb128Error::ValueTooLarge);
        }

        result |= value_bits << shift;
        shift += 7;

        // 如果沒有延續位，解碼完成
        if (byte & 0x80) == 0 {
            return Ok(result);
        }

        // 防止無限迴圈
        if bytes_read > 5 {
            return Err(Uleb128Error::EncodingTooLong);
        }
    }
}
