//! V7 file format definitions.
//!
//! 主資料檔採定長槽位布局：seed 不寫入檔案，而是由槽位位置隱含
//! （slot i 存放 seed = i+1 的結果）。空槽位為全零（建檔時 `set_len`
//! 預配置，在支援 sparse file 的檔案系統上不佔實際磁碟空間）。
//!
//! 計算期間由 CRC sidecar 檔（`.crc`）提供逐筆完整性驗證（偵測
//! torn write 與未完成槽位）；完成後 sidecar 刪除，改由 SHA-256
//! 雜湊檔（`.sha256`）負責整檔完整性驗證。

use std::path::{Path, PathBuf};

use super::config::{MAX_READ_BUFFER_SIZE, MIN_READ_BUFFER_SIZE};
use super::crc32::Crc32;
use super::error::StorageError;

/// Magic header identifying a V7 data file.
pub const MAGIC_HEADER: &[u8; 12] = b"EIGENVALS_V7";
/// Total size of the data file header in bytes.
pub const HEADER_SIZE: u64 = 32;
/// Byte offset of the status field within the data file header.
pub const STATUS_OFFSET: u64 = 27;

/// Magic header identifying a CRC sidecar file.
pub const CRC_MAGIC: &[u8; 8] = b"EIGCRC_7";
/// Total size of the CRC sidecar header in bytes.
pub const CRC_HEADER_SIZE: u64 = 16;

/// File status: simulation still in progress (slots may be missing).
pub const STATUS_IN_PROGRESS: u8 = 0;
/// File status: every slot is written; the hash file is authoritative.
pub const STATUS_COMPLETE: u8 = 1;

/// Suffix appended to the data file path for the CRC sidecar.
pub const CRC_FILE_SUFFIX: &str = ".crc";
/// Suffix appended to the data file path for the SHA-256 hash file.
pub const HASH_FILE_SUFFIX: &str = ".sha256";

/// Parsed data file header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileHeader {
    pub model: u8,
    pub dim: u8,
    pub steps: u32,
    pub num_runs: u64,
    pub eigenvalues_per_run: u8,
    pub status: u8,
}

impl FileHeader {
    /// Size in bytes of one record slot.
    pub fn slot_size(&self) -> u64 {
        self.eigenvalues_per_run as u64 * 8
    }

    /// Byte offset of the slot holding the record for `seed` (1-based).
    pub fn slot_offset(&self, seed: u32) -> u64 {
        HEADER_SIZE + (seed as u64 - 1) * self.slot_size()
    }

    /// Byte offset of the CRC entry for `seed` within the sidecar file.
    pub fn crc_offset(seed: u32) -> u64 {
        CRC_HEADER_SIZE + (seed as u64 - 1) * 4
    }

    /// Expected size of the data file implied by this header.
    pub fn data_file_size(&self) -> u64 {
        HEADER_SIZE + self.num_runs * self.slot_size()
    }

    /// Expected size of the CRC sidecar file implied by this header.
    pub fn crc_file_size(&self) -> u64 {
        CRC_HEADER_SIZE + self.num_runs * 4
    }

    /// Serialize the header into its 32-byte on-disk form.
    pub fn encode(&self) -> [u8; HEADER_SIZE as usize] {
        let mut buf = [0u8; HEADER_SIZE as usize];
        buf[0..12].copy_from_slice(MAGIC_HEADER);
        buf[12] = self.model;
        buf[13] = self.dim;
        buf[14..18].copy_from_slice(&self.steps.to_le_bytes());
        buf[18..26].copy_from_slice(&self.num_runs.to_le_bytes());
        buf[26] = self.eigenvalues_per_run;
        buf[27] = self.status;
        // 28..32 保留為零
        buf
    }

    /// Parse and structurally validate a 32-byte on-disk header.
    pub fn decode(buf: &[u8; HEADER_SIZE as usize]) -> Result<Self, StorageError> {
        if &buf[0..12] != MAGIC_HEADER {
            return Err(StorageError::MagicMismatch);
        }
        let header = Self {
            model: buf[12],
            dim: buf[13],
            steps: u32::from_le_bytes(buf[14..18].try_into().unwrap()),
            num_runs: u64::from_le_bytes(buf[18..26].try_into().unwrap()),
            eigenvalues_per_run: buf[26],
            status: buf[27],
        };
        if header.eigenvalues_per_run == 0 {
            return Err(StorageError::Corrupted(
                "eigenvalues per record is zero".to_string(),
            ));
        }
        if header.status > STATUS_COMPLETE {
            return Err(StorageError::Corrupted(format!(
                "unknown status byte {}",
                header.status
            )));
        }
        // seed 為 u32，容量不可超出 u32 範圍
        if header.num_runs > u32::MAX as u64 {
            return Err(StorageError::Corrupted(format!(
                "num_runs {} exceeds the u32 seed range",
                header.num_runs
            )));
        }
        Ok(header)
    }

    /// Check that this header matches the requested configuration.
    /// `num_runs` and `status` are intentionally excluded: capacity is
    /// negotiated separately and status describes progress, not identity.
    pub fn check_compatible(&self, expected: &FileHeader) -> Result<(), StorageError> {
        let checks: [(&'static str, u64, u64); 4] = [
            ("model", self.model as u64, expected.model as u64),
            ("dim", self.dim as u64, expected.dim as u64),
            ("steps", self.steps as u64, expected.steps as u64),
            (
                "eigenvalues per record",
                self.eigenvalues_per_run as u64,
                expected.eigenvalues_per_run as u64,
            ),
        ];
        for (field, file_value, expected) in checks {
            if file_value != expected {
                return Err(StorageError::ParamMismatch {
                    field,
                    file_value,
                    expected,
                });
            }
        }
        Ok(())
    }

    /// Serialize the CRC sidecar header (magic + num_runs).
    pub fn encode_crc_header(&self) -> [u8; CRC_HEADER_SIZE as usize] {
        let mut buf = [0u8; CRC_HEADER_SIZE as usize];
        buf[0..8].copy_from_slice(CRC_MAGIC);
        buf[8..16].copy_from_slice(&self.num_runs.to_le_bytes());
        buf
    }

    /// Check that a CRC sidecar header belongs to this data file.
    pub fn crc_header_matches(&self, buf: &[u8; CRC_HEADER_SIZE as usize]) -> bool {
        &buf[0..8] == CRC_MAGIC
            && u64::from_le_bytes(buf[8..16].try_into().unwrap()) == self.num_runs
    }
}

/// Compute the CRC recorded for one slot: CRC32 over `seed_le ‖ payload`.
///
/// 將 seed 納入計算範圍，使 CRC 同時驗證「資料寫在正確的槽位」。
pub fn record_crc(seed: u32, payload: &[u8]) -> u32 {
    let mut hasher = Crc32::new();
    hasher.update(&seed.to_le_bytes());
    hasher.update(payload);
    hasher.finalize()
}

/// Decide whether a slot holds a valid record.
///
/// 有效 ⟺ CRC 相符且 payload 非全零（特徵值幾乎必然為正，全零即空槽位）。
/// `stored_crc` 為 `None` 表示 CRC sidecar 遺失，退回「非全零即有效」的
/// 降級判定（無法偵測 torn write）。
pub fn slot_is_valid(seed: u32, payload: &[u8], stored_crc: Option<u32>) -> bool {
    let nonzero = payload.iter().any(|&b| b != 0);
    match stored_crc {
        Some(crc) => nonzero && crc == record_crc(seed, payload),
        None => nonzero,
    }
}

/// Path of the CRC sidecar file belonging to `data_path`.
pub fn crc_path(data_path: &Path) -> PathBuf {
    let mut s = data_path.as_os_str().to_os_string();
    s.push(CRC_FILE_SUFFIX);
    PathBuf::from(s)
}

/// Path of the SHA-256 hash file belonging to `data_path`.
pub fn hash_path(data_path: &Path) -> PathBuf {
    let mut s = data_path.as_os_str().to_os_string();
    s.push(HASH_FILE_SUFFIX);
    PathBuf::from(s)
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
        let buffer_size = (file_size / 8) as usize;
        buffer_size.clamp(MIN_READ_BUFFER_SIZE, 4 * 1024 * 1024)
    } else {
        MAX_READ_BUFFER_SIZE
    }
}
