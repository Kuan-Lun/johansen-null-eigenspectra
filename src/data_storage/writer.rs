//! Slot writer for the V7 fixed-slot format.
//!
//! 每個 seed 對應主檔中固定位置的槽位，worker 執行緒以定位寫入
//! （pwrite）並行寫入互不重疊的槽位，無需鎖或寫入執行緒。
//! 寫入期間同步維護 CRC sidecar；完成時執行完成協議：
//! fsync → status=1 → fsync → 計算整檔 SHA-256 → 寫入雜湊檔 → 刪除 sidecar。

use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::error::StorageError;
use super::file_format::{
    CRC_HEADER_SIZE, FileHeader, HEADER_SIZE, STATUS_COMPLETE, STATUS_IN_PROGRESS, STATUS_OFFSET,
    calculate_read_buffer_size, crc_path, hash_path, record_crc,
};
use super::reader::{open_data_file, scan_slots};
use crate::display_utils::format_number_with_commas;

/// 跨平台的定位寫入（不移動檔案游標，可多執行緒併行使用）
#[cfg(unix)]
fn write_all_at(file: &File, buf: &[u8], offset: u64) -> std::io::Result<()> {
    use std::os::unix::fs::FileExt;
    file.write_all_at(buf, offset)
}

#[cfg(windows)]
fn write_all_at(file: &File, mut buf: &[u8], mut offset: u64) -> std::io::Result<()> {
    use std::os::windows::fs::FileExt;
    while !buf.is_empty() {
        let written = file.seek_write(buf, offset)?;
        if written == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "failed to write whole buffer",
            ));
        }
        buf = &buf[written..];
        offset += written as u64;
    }
    Ok(())
}

/// Result of opening a data file for writing.
pub enum OpenResult {
    /// Every requested slot already holds a valid record.
    AlreadyComplete,
    /// The file accepts writes for the listed remaining seeds.
    InProgress {
        writer: SlotWriter,
        remaining_seeds: Vec<u32>,
        completed_runs: usize,
    },
}

/// Positioned writer over a data file and its CRC sidecar.
pub struct SlotWriter {
    data_file: File,
    crc_file: File,
    header: FileHeader,
    data_path: PathBuf,
    /// 檔案容量內、但在本次請求範圍之外的未完成槽位數。
    /// 大於零時 finish() 不得標記完成。
    pending_outside_run: usize,
    quiet: bool,
}

impl SlotWriter {
    /// Open (or create) the data file for `expected` and determine which
    /// seeds within `1..=expected.num_runs` still need to be computed.
    ///
    /// 既有檔案會經過參數驗證、必要時擴容，並掃描全部槽位的有效性。
    pub fn open<P: AsRef<Path>>(
        path: P,
        expected: FileHeader,
        quiet: bool,
    ) -> Result<OpenResult, StorageError> {
        let data_path = path.as_ref().to_path_buf();
        if !data_path.exists() {
            return Self::create_new(data_path, expected, quiet);
        }

        let existing = open_data_file(&data_path)?;
        let mut header = existing.header;
        drop(existing);
        header.check_compatible(&expected)?;

        // 已完成且容量足夠：無事可做
        if header.status == STATUS_COMPLETE && header.num_runs >= expected.num_runs {
            return Ok(OpenResult::AlreadyComplete);
        }

        let data_file = OpenOptions::new().read(true).write(true).open(&data_path)?;

        let crc_file = if header.num_runs < expected.num_runs {
            // 容量不足：擴容（會將 status 歸零並使舊雜湊檔失效）
            expand_file(
                &data_file,
                &mut header,
                expected.num_runs,
                &data_path,
                quiet,
            )?
        } else {
            // 修剪擴容中斷可能殘留的多餘尾端，恢復大小不變式
            if data_file.metadata()?.len() > header.data_file_size() {
                data_file.set_len(header.data_file_size())?;
            }
            ensure_crc_file(&data_file, &header, &data_path, quiet)?
        };

        // 掃描整個檔案容量的槽位有效性（涵蓋請求範圍之外的槽位，
        // 供 finish() 判斷能否將整個檔案標記為完成）
        let mut remaining_all: Vec<u32> = Vec::new();
        scan_slots(
            &data_file,
            &header,
            Some(&crc_file),
            header.num_runs,
            |seed, _payload, valid| {
                if !valid {
                    remaining_all.push(seed);
                }
            },
        )?;

        let requested = expected.num_runs;
        let mut remaining_seeds: Vec<u32> = Vec::new();
        let mut pending_outside_run = 0usize;
        for &seed in &remaining_all {
            if seed as u64 <= requested {
                remaining_seeds.push(seed);
            } else {
                pending_outside_run += 1;
            }
        }
        let completed_runs = requested as usize - remaining_seeds.len();

        if !quiet && completed_runs > 0 {
            println!(
                "Detected existing file with {} completed records",
                format_number_with_commas(completed_runs)
            );
        }

        let writer = SlotWriter {
            data_file,
            crc_file,
            header,
            data_path,
            pending_outside_run,
            quiet,
        };

        if remaining_seeds.is_empty() {
            // 請求範圍已全部完成（例如前次執行在收尾前中斷）：直接收尾
            writer.finish()?;
            return Ok(OpenResult::AlreadyComplete);
        }

        Ok(OpenResult::InProgress {
            writer,
            remaining_seeds,
            completed_runs,
        })
    }

    /// 建立全新的主檔與 CRC sidecar，並以 `set_len` 預配置空間
    fn create_new(
        data_path: PathBuf,
        expected: FileHeader,
        quiet: bool,
    ) -> Result<OpenResult, StorageError> {
        let mut header = expected;
        header.status = STATUS_IN_PROGRESS;

        // 路徑計算不建立目錄，改由此處在建立檔案前一併建立；
        // parent 可能是 ""（純檔名），此時無目錄可建
        if let Some(parent) = data_path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }

        let data_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&data_path)?;
        write_all_at(&data_file, &header.encode(), 0)?;
        data_file.set_len(header.data_file_size())?;
        data_file.sync_all()?;

        let crc_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(crc_path(&data_path))?;
        write_all_at(&crc_file, &header.encode_crc_header(), 0)?;
        crc_file.set_len(header.crc_file_size())?;
        crc_file.sync_all()?;

        let remaining_seeds: Vec<u32> = (1..=header.num_runs as u32).collect();
        Ok(OpenResult::InProgress {
            writer: SlotWriter {
                data_file,
                crc_file,
                header,
                data_path,
                pending_outside_run: 0,
                quiet,
            },
            remaining_seeds,
            completed_runs: 0,
        })
    }

    /// Write the eigenvalues for `seed` into its slot.
    ///
    /// Slots do not overlap, so this is safe to call concurrently from
    /// multiple threads for distinct seeds.
    pub fn write_slot(&self, seed: u32, eigenvalues: &[f64]) -> Result<(), StorageError> {
        let expected_len = self.header.eigenvalues_per_run as usize;
        if eigenvalues.len() != expected_len {
            return Err(StorageError::InvalidInput(format!(
                "eigenvalue count mismatch: expected {expected_len}, actual {} (model {}, dim {}, steps {})",
                eigenvalues.len(),
                self.header.model,
                self.header.dim,
                self.header.steps
            )));
        }
        if seed == 0 || seed as u64 > self.header.num_runs {
            return Err(StorageError::InvalidInput(format!(
                "seed {seed} is outside the file capacity 1..={}",
                self.header.num_runs
            )));
        }

        let payload: Vec<u8> = eigenvalues
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect();
        write_all_at(&self.data_file, &payload, self.header.slot_offset(seed))?;

        let crc = record_crc(seed, &payload);
        write_all_at(
            &self.crc_file,
            &crc.to_le_bytes(),
            FileHeader::crc_offset(seed),
        )?;
        Ok(())
    }

    /// Flush both files' data to disk.
    pub fn sync(&self) -> Result<(), StorageError> {
        self.data_file.sync_data()?;
        self.crc_file.sync_data()?;
        Ok(())
    }

    /// Finalize the file.
    ///
    /// When every slot in the file is valid, this marks the file complete,
    /// writes the SHA-256 hash file, and removes the CRC sidecar. Otherwise
    /// the file stays in progress and keeps its sidecar for future resumes.
    pub fn finish(mut self) -> Result<(), StorageError> {
        self.data_file.sync_data()?;
        self.crc_file.sync_data()?;

        if self.pending_outside_run > 0 {
            // 檔案容量大於本次請求範圍，仍有槽位未完成：維持進行中狀態
            if !self.quiet {
                println!(
                    "Note: {} slots beyond the requested range remain incomplete; file stays resumable",
                    format_number_with_commas(self.pending_outside_run)
                );
            }
            return Ok(());
        }

        // 完成協議：資料已落盤 → 標記完成 → 落盤 → 整檔雜湊 → 刪除 sidecar
        self.header.status = STATUS_COMPLETE;
        write_all_at(&self.data_file, &[STATUS_COMPLETE], STATUS_OFFSET)?;
        self.data_file.sync_all()?;

        let hash_hex = sha256_file(&self.data_path)?;
        let basename = self
            .data_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        std::fs::write(
            hash_path(&self.data_path),
            format!("{hash_hex}  {basename}\n"),
        )?;

        let _ = std::fs::remove_file(crc_path(&self.data_path));

        if !self.quiet {
            println!(
                "SUCCESS: simulation data complete, {} records for model {}, dim {}, steps {}",
                format_number_with_commas(self.header.num_runs as usize),
                self.header.model,
                self.header.dim,
                self.header.steps
            );
        }
        Ok(())
    }
}

/// 嘗試以讀寫模式打開結構有效的 CRC sidecar；遺失或無效時回傳 `None`
fn try_open_valid_crc(header: &FileHeader, crc_path: &Path) -> Result<Option<File>, StorageError> {
    let mut file = match OpenOptions::new().read(true).write(true).open(crc_path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(StorageError::Io(e)),
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

/// 確保 CRC sidecar 存在且結構有效；遺失時以降級模式重建
fn ensure_crc_file(
    data_file: &File,
    header: &FileHeader,
    data_path: &Path,
    quiet: bool,
) -> Result<File, StorageError> {
    let path = crc_path(data_path);
    if let Some(file) = try_open_valid_crc(header, &path)? {
        return Ok(file);
    }
    if !quiet {
        println!(
            "WARNING: CRC sidecar missing or invalid; rebuilding from existing data (torn-write detection unavailable for existing records)"
        );
    }
    rebuild_crc_file(data_file, header, &path)
}

/// 重建 CRC sidecar：非零槽位以現存內容計算 CRC（視為有效），零槽位保持 0
fn rebuild_crc_file(
    data_file: &File,
    header: &FileHeader,
    crc_path: &Path,
) -> Result<File, StorageError> {
    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(crc_path)?;
    {
        let mut writer = BufWriter::new(&file);
        writer.write_all(&header.encode_crc_header())?;

        let buffer_size = calculate_read_buffer_size(header.data_file_size());
        let mut reader = BufReader::with_capacity(buffer_size, data_file);
        reader.seek(SeekFrom::Start(HEADER_SIZE))?;

        let mut payload = vec![0u8; header.slot_size() as usize];
        for seed in 1..=header.num_runs {
            reader.read_exact(&mut payload)?;
            let crc = if payload.iter().any(|&b| b != 0) {
                record_crc(seed as u32, &payload)
            } else {
                0
            };
            writer.write_all(&crc.to_le_bytes())?;
        }
        writer.flush()?;
    }
    file.sync_all()?;
    Ok(file)
}

/// 擴容：延長主檔與 sidecar 至新容量並更新 header。
/// 舊資料原封保留；舊 sidecar 有效時延用其 CRC，否則重建
/// （包含「已完成檔案再擴容」的情境：sidecar 已刪除，據現存資料重建）。
fn expand_file(
    data_file: &File,
    header: &mut FileHeader,
    new_num_runs: u64,
    data_path: &Path,
    quiet: bool,
) -> Result<File, StorageError> {
    if !quiet {
        println!(
            "Expanding data file capacity from {} to {} runs",
            format_number_with_commas(header.num_runs as usize),
            format_number_with_commas(new_num_runs as usize)
        );
    }
    let old_header = *header;
    header.num_runs = new_num_runs;
    header.status = STATUS_IN_PROGRESS;

    // 先延長主檔（新區域為零 → 未完成槽位），再處理 sidecar，最後才改寫
    // header：中途斷電時 header 仍描述舊的一致狀態
    data_file.set_len(header.data_file_size())?;

    let path = crc_path(data_path);
    let crc_file = match try_open_valid_crc(&old_header, &path)? {
        Some(file) => {
            // 延長既有 sidecar，保留舊 CRC（維持 torn-write 偵測能力）
            file.set_len(header.crc_file_size())?;
            write_all_at(&file, &header.encode_crc_header(), 0)?;
            file.sync_all()?;
            file
        }
        None => rebuild_crc_file(data_file, header, &path)?,
    };

    write_all_at(data_file, &header.encode(), 0)?;
    data_file.sync_all()?;

    // 舊的整檔雜湊已失效
    let _ = std::fs::remove_file(hash_path(data_path));
    Ok(crc_file)
}

/// Stream a file through SHA-256 and return the lowercase hex digest.
pub fn sha256_file(path: &Path) -> Result<String, StorageError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let digest = hasher.finalize();
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}
