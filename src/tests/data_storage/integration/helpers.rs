use super::*;
use crate::data_storage::file_format::{
    HEADER_SIZE, STATUS_IN_PROGRESS, STATUS_OFFSET, crc_path, hash_path,
};
use std::io::{Read, Seek, SeekFrom, Write};

/// 將指定 seeds 的槽位清為零，模擬中斷後缺漏的資料（測試用）。
///
/// V7 格式下「移除一筆資料」= 把該 seed 的槽位歸零。同時把檔案狀態
/// 改回進行中，並移除已失效的 CRC sidecar 與整檔雜湊檔，讓後續的
/// `run_simulation()` 走斷點續傳路徑重算這些 seeds。
pub fn remove_seed_from_file(
    simulation: &EigenvalueSimulation,
    model: JohansenModel,
    seeds_to_remove: &[u32],
) -> std::io::Result<usize> {
    let path = simulation.get_file_path(model);
    if !path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "data file not found",
        ));
    }

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)?;

    // 直接解析 V7 header 的必要欄位（同時作為磁碟布局的迴歸驗證）
    let mut header = [0u8; HEADER_SIZE as usize];
    file.read_exact(&mut header)?;
    let num_runs = u64::from_le_bytes(header[18..26].try_into().unwrap());
    let slot_size = header[26] as u64 * 8;

    let zeros = vec![0u8; slot_size as usize];
    let mut removed = 0usize;
    for &seed in seeds_to_remove {
        if seed >= 1 && seed as u64 <= num_runs {
            let offset = HEADER_SIZE + (seed as u64 - 1) * slot_size;
            file.seek(SeekFrom::Start(offset))?;
            file.write_all(&zeros)?;
            removed += 1;
        }
    }

    // 狀態改回進行中；sidecar 與整檔雜湊已失效，一併移除
    file.seek(SeekFrom::Start(STATUS_OFFSET))?;
    file.write_all(&[STATUS_IN_PROGRESS])?;
    file.sync_all()?;
    let _ = std::fs::remove_file(crc_path(&path));
    let _ = std::fs::remove_file(hash_path(&path));

    Ok(removed)
}
