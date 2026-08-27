//! File reading for the V7 fixed-slot format.
//!
//! 提供兩條讀取路徑：
//! - 快速讀取：檔案標記完成（status=1）時直接順序讀取，不做逐筆驗證。
//! - 掃描讀取：進行中的檔案逐槽以 CRC 驗證有效性，用於斷點續傳與部分讀取。

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use super::config::MIN_READ_BUFFER_SIZE;
use super::file_format::{
    CRC_HEADER_SIZE, FileHeader, HEADER_SIZE, calculate_read_buffer_size, slot_is_valid,
};
use crate::error::Error;

/// An opened data file with its parsed header.
pub struct DataFile {
    pub file: File,
    pub header: FileHeader,
}

/// Open a data file read-only and validate its header and size.
pub fn open_data_file<P: AsRef<Path>>(path: P) -> Result<DataFile, Error> {
    let mut file = File::open(&path)?;
    let mut buf = [0u8; HEADER_SIZE as usize];
    file.read_exact(&mut buf)?;
    let header = FileHeader::decode(&buf)?;

    // 容量不變式：檔案至少要有 header 宣告的大小。
    // 允許稍大（擴容中斷可能留下多餘尾端），以 header 為準讀取。
    let actual = file.metadata()?.len();
    let expected = header.data_file_size();
    if actual < expected {
        return Err(Error::Corrupted(format!(
            "data file is smaller than its header implies ({actual} < {expected} bytes)"
        )));
    }
    Ok(DataFile { file, header })
}

/// Open the CRC sidecar belonging to `header`, read-only.
///
/// Returns `Ok(None)` when the sidecar is missing or structurally invalid;
/// the caller then falls back to degraded validity checks.
pub fn open_crc_file<P: AsRef<Path>>(path: P, header: &FileHeader) -> Result<Option<File>, Error> {
    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(Error::Io(e)),
    };
    let mut buf = [0u8; CRC_HEADER_SIZE as usize];
    if file.read_exact(&mut buf).is_err() {
        return Ok(None);
    }
    if !header.crc_header_matches(&buf) {
        return Ok(None);
    }
    if file.metadata()?.len() < header.crc_file_size() {
        return Ok(None);
    }
    Ok(Some(file))
}

/// Decode a slot payload into its eigenvalues.
pub fn decode_payload(payload: &[u8]) -> Vec<f64> {
    payload
        .as_chunks::<8>()
        .0
        .iter()
        .map(|chunk| f64::from_le_bytes(*chunk))
        .collect()
}

/// Sequentially visit the slots for seeds `1..=limit`, reporting validity.
///
/// `crc_file` of `None` selects the degraded "non-zero payload" check.
/// The callback receives `(seed, payload_bytes, valid)`.
pub fn scan_slots(
    data_file: &File,
    header: &FileHeader,
    crc_file: Option<&File>,
    limit: u64,
    mut f: impl FnMut(u32, &[u8], bool),
) -> Result<(), Error> {
    let slot_size = header.slot_size() as usize;
    let buffer_size = calculate_read_buffer_size(header.data_file_size());
    let mut data_reader = BufReader::with_capacity(buffer_size, data_file);
    data_reader.seek(SeekFrom::Start(HEADER_SIZE))?;

    let mut crc_reader = match crc_file {
        Some(file) => {
            let mut reader = BufReader::with_capacity(MIN_READ_BUFFER_SIZE, file);
            reader.seek(SeekFrom::Start(CRC_HEADER_SIZE))?;
            Some(reader)
        }
        None => None,
    };

    let mut payload = vec![0u8; slot_size];
    for seed in 1..=limit {
        data_reader.read_exact(&mut payload)?;
        let stored_crc = match crc_reader.as_mut() {
            Some(reader) => {
                let mut buf = [0u8; 4];
                reader.read_exact(&mut buf)?;
                Some(u32::from_le_bytes(buf))
            }
            None => None,
        };
        let seed = seed as u32;
        let valid = slot_is_valid(seed, &payload, stored_crc);
        f(seed, &payload, valid);
    }
    Ok(())
}

/// Read the slots for seeds `1..=count` without per-record validation.
///
/// 僅適用於已標記完成（status=1）的檔案：完成協議保證所有槽位有效。
pub fn read_slots(
    data_file: &File,
    header: &FileHeader,
    count: u64,
) -> Result<Vec<Vec<f64>>, Error> {
    let slot_size = header.slot_size() as usize;
    let buffer_size = calculate_read_buffer_size(header.data_file_size());
    let mut reader = BufReader::with_capacity(buffer_size, data_file);
    reader.seek(SeekFrom::Start(HEADER_SIZE))?;

    let mut payload = vec![0u8; slot_size];
    let mut records = Vec::with_capacity(count as usize);
    for _ in 0..count {
        reader.read_exact(&mut payload)?;
        records.push(decode_payload(&payload));
    }
    Ok(records)
}
