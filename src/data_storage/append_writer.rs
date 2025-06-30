//! 追加寫入器 - 高性能的數據寫入方案
//!
//! 這個模組實現了真正的追加寫入，避免每次都重寫整個檔案，
//! 大幅提升大規模模擬的寫入性能。

use crate::display_utils::{format_number_with_commas, format_remaining_time};
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::mpsc;
use std::thread;

use super::config::PROGRESS_REPORT_INTERVAL;

/// 格式化剩餘時間顯示
/// 檔案格式常數
const MAGIC_HEADER: &[u8] = b"EIGENVALS_V2"; // 12 bytes
const EOF_MARKER: &[u8] = b"EOF_MARK"; // 8 bytes

/// 追加寫入器 - 支援高效的數據追加和斷點續傳
pub struct AppendOnlyWriter {
    writer: BufWriter<File>,
    written_count: usize,
    eigenvalues_per_run: Option<usize>,
    quiet: bool,
}

impl AppendOnlyWriter {
    /// 創建新的追加寫入器
    pub fn new<P: AsRef<Path>>(path: P, quiet: bool) -> std::io::Result<Self> {
        let path_ref = path.as_ref();
        let is_new_file = !path_ref.exists();

        let mut written_count = 0;
        let mut eigenvalues_per_run = None;

        if is_new_file {
            // 新檔案：直接創建並寫入魔術標頭
            let file = OpenOptions::new().create(true).write(true).open(path_ref)?;

            let mut writer = BufWriter::new(file);
            writer.write_all(MAGIC_HEADER)?;
            writer.flush()?;

            Ok(Self {
                writer,
                written_count: 0,
                eigenvalues_per_run: None,
                quiet,
            })
        } else {
            // 既有檔案：檢查數據並移除 EOF 標記
            if let Ok(existing_data) = read_append_file(&path) {
                written_count = existing_data.len();
                if let Some((_, eigenvalues)) = existing_data.first() {
                    eigenvalues_per_run = Some(eigenvalues.len());
                }
                if !quiet {
                    println!(
                        "檢測到既有檔案，已有 {} 筆數據",
                        format_number_with_commas(written_count)
                    );
                }
            }

            // 移除 EOF 標記：打開檔案並截斷到數據結束位置
            let mut file = OpenOptions::new().read(true).write(true).open(path_ref)?;

            let file_len = file.metadata()?.len();

            // 檢查檔案結尾是否真的包含 EOF 標記
            if file_len >= 12 + 20 {
                file.seek(SeekFrom::End(-20))?; // EOF_MARK + count + eigenvalues_per_run
                let mut eof_buf = [0u8; 8];
                if let Ok(()) = file.read_exact(&mut eof_buf) {
                    if &eof_buf == EOF_MARKER {
                        // 移除結束標記和元數據
                        let new_len = file_len - 20;
                        file.set_len(new_len)?;
                    }
                }
            }

            // 設置為追加模式
            let file = OpenOptions::new().append(true).open(path_ref)?;

            let writer = BufWriter::new(file);

            Ok(Self {
                writer,
                written_count,
                eigenvalues_per_run,
                quiet,
            })
        }
    }

    /// 追加特徵值數據
    pub fn append_eigenvalues(&mut self, seed: u64, eigenvalues: &[f64]) -> std::io::Result<()> {
        // 如果是第一次寫入，記錄特徵值的數量
        if self.eigenvalues_per_run.is_none() {
            self.eigenvalues_per_run = Some(eigenvalues.len());
        }

        // 檢查特徵值數量是否一致
        if let Some(expected_len) = self.eigenvalues_per_run {
            if eigenvalues.len() != expected_len {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "特徵值數量不一致: 期待 {}, 實際 {}",
                        format_number_with_commas(expected_len),
                        format_number_with_commas(eigenvalues.len())
                    ),
                ));
            }
        }

        // 寫入數據塊：[seed: 8 bytes] [eigenvalue_count: 4 bytes] [eigenvalues: count * 8 bytes]
        self.writer.write_all(&seed.to_le_bytes())?;
        self.writer
            .write_all(&(eigenvalues.len() as u32).to_le_bytes())?;

        for &val in eigenvalues {
            self.writer.write_all(&val.to_le_bytes())?;
        }

        self.written_count += 1;

        // 定期刷新緩衝區
        if self.written_count % 1000 == 0 {
            self.writer.flush()?;
        }

        Ok(())
    }

    /// 完成寫入，添加結束標記
    pub fn finish(mut self) -> std::io::Result<()> {
        // 刷新所有緩衝的數據
        self.writer.flush()?;

        // 寫入結束標記和總數
        self.writer.write_all(EOF_MARKER)?;
        self.writer
            .write_all(&(self.written_count as u64).to_le_bytes())?;

        if let Some(eigenvalues_per_run) = self.eigenvalues_per_run {
            self.writer
                .write_all(&(eigenvalues_per_run as u32).to_le_bytes())?;
        } else {
            self.writer.write_all(&0u32.to_le_bytes())?;
        }

        self.writer.flush()?;

        if !self.quiet {
            println!(
                "SUCCESS: 追加寫入完成，共寫入 {} 筆數據",
                format_number_with_commas(self.written_count)
            );
        }

        Ok(())
    }

    /// 獲取已寫入的數據筆數
    #[allow(dead_code)]
    pub fn written_count(&self) -> usize {
        self.written_count
    }
}

/// 讀取追加格式的檔案
pub fn read_append_file<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<(u64, Vec<f64>)>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // 檢查魔術標頭
    let mut magic_buf = [0u8; 12];
    reader.read_exact(&mut magic_buf)?;
    if &magic_buf != MAGIC_HEADER {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "檔案格式錯誤：魔術標頭不匹配",
        ));
    }

    // 嘗試從檔案末尾讀取元數據
    let file_len = reader.get_ref().metadata()?.len();
    if file_len < 12 + 8 + 8 + 4 {
        // magic + eof_marker + count + eigenvalues_per_run
        return Ok(Vec::new()); // 檔案太小，可能是空檔案
    }

    // 檢查是否有完整的結束標記
    let metadata = read_file_metadata(&mut reader, file_len)?;

    if let Some((total_count, eigenvalues_per_run)) = metadata {
        // 有完整的結束標記，使用快速讀取
        read_with_metadata(&mut reader, total_count, eigenvalues_per_run)
    } else {
        // 沒有結束標記，掃描式讀取（用於未完成的檔案）
        scan_read_data(&mut reader)
    }
}

/// 嘗試從檔案末尾讀取元數據
fn read_file_metadata(
    reader: &mut BufReader<File>,
    file_len: u64,
) -> std::io::Result<Option<(usize, usize)>> {
    // 定位到檔案末尾的元數據位置
    let metadata_offset = file_len - 8 - 4; // count + eigenvalues_per_run
    reader.seek(SeekFrom::Start(metadata_offset - 8))?; // 包括 EOF_MARKER

    // 檢查 EOF 標記
    let mut eof_buf = [0u8; 8];
    reader.read_exact(&mut eof_buf)?;
    if &eof_buf != EOF_MARKER {
        return Ok(None); // 沒有有效的結束標記
    }

    // 讀取總數和特徵值數量
    let mut count_buf = [0u8; 8];
    let mut eigenvalues_buf = [0u8; 4];

    reader.read_exact(&mut count_buf)?;
    reader.read_exact(&mut eigenvalues_buf)?;

    let total_count = u64::from_le_bytes(count_buf) as usize;
    let eigenvalues_per_run = u32::from_le_bytes(eigenvalues_buf) as usize;

    Ok(Some((total_count, eigenvalues_per_run)))
}

/// 使用元數據快速讀取
fn read_with_metadata(
    reader: &mut BufReader<File>,
    total_count: usize,
    eigenvalues_per_run: usize,
) -> std::io::Result<Vec<(u64, Vec<f64>)>> {
    // 回到數據開始位置
    reader.seek(SeekFrom::Start(12))?; // 跳過魔術標頭

    let mut data = Vec::with_capacity(total_count);

    for _ in 0..total_count {
        let mut seed_buf = [0u8; 8];
        let mut count_buf = [0u8; 4];

        reader.read_exact(&mut seed_buf)?;
        reader.read_exact(&mut count_buf)?;

        let seed = u64::from_le_bytes(seed_buf);
        let eigenvalue_count = u32::from_le_bytes(count_buf) as usize;

        if eigenvalue_count != eigenvalues_per_run {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "特徵值數量不一致: 期待 {}, 實際 {}",
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
fn scan_read_data(reader: &mut BufReader<File>) -> std::io::Result<Vec<(u64, Vec<f64>)>> {
    // 回到數據開始位置
    reader.seek(SeekFrom::Start(12))?; // 跳過魔術標頭

    let mut data = Vec::new();

    loop {
        let mut seed_buf = [0u8; 8];
        let mut count_buf = [0u8; 4];

        // 嘗試讀取 seed
        if reader.read_exact(&mut seed_buf).is_err() {
            break; // 到達檔案末尾
        }

        // 檢查是否是 EOF 標記
        if &seed_buf == EOF_MARKER {
            break; // 遇到結束標記
        }

        // 讀取特徵值數量
        if reader.read_exact(&mut count_buf).is_err() {
            break; // 不完整的數據塊
        }

        let seed = u64::from_le_bytes(seed_buf);
        let eigenvalue_count = u32::from_le_bytes(count_buf) as usize;

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

/// 檢查檔案進度（追加格式）
pub fn check_append_progress<P: AsRef<Path>>(path: P) -> std::io::Result<(usize, Vec<u64>)> {
    if !path.as_ref().exists() {
        return Ok((0, Vec::new()));
    }

    match read_append_file(&path) {
        Ok(data) => {
            let completed_runs = data.len();
            let completed_seeds: Vec<u64> = data.iter().map(|(seed, _)| *seed).collect();
            Ok((completed_runs, completed_seeds))
        }
        Err(_) => Ok((0, Vec::new())), // 檔案損壞或無法讀取，重新開始
    }
}

/// 獲取尚未完成的seed列表
pub fn get_remaining_seeds(total_runs: usize, completed_seeds: &[u64]) -> Vec<u64> {
    let completed_set: HashSet<u64> = completed_seeds.iter().copied().collect();
    (1..=total_runs as u64)
        .filter(|seed| !completed_set.contains(seed))
        .collect()
}

/// 啟動追加寫入執行緒
pub fn spawn_append_writer_thread(
    filename: String,
    receiver: mpsc::Receiver<(u64, Vec<f64>)>,
    total_runs: usize,
    quiet: bool,
) -> thread::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>> {
    thread::spawn(move || {
        let mut writer = AppendOnlyWriter::new(&filename, quiet)?;
        let mut count = 0;
        let start_time = std::time::Instant::now();

        while let Ok((seed, eigenvalues)) = receiver.recv() {
            writer.append_eigenvalues(seed, &eigenvalues)?;
            count += 1;

            if count % PROGRESS_REPORT_INTERVAL == 0 && !quiet {
                let progress_ratio = count as f64 / total_runs as f64;
                let elapsed = start_time.elapsed();

                if progress_ratio > 0.0 {
                    println!(
                        "模擬進度: {}/{} ({:.2}%) - {}",
                        format_number_with_commas(count),
                        format_number_with_commas(total_runs),
                        progress_ratio * 100.0,
                        format_remaining_time(elapsed, count, total_runs)
                    );
                } else {
                    println!(
                        "模擬進度: {}/{} ({:.2}%)",
                        format_number_with_commas(count),
                        format_number_with_commas(total_runs),
                        progress_ratio * 100.0
                    );
                }
            }
        }

        writer.finish()?;
        Ok(())
    })
}
