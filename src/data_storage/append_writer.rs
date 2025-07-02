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

use super::config::{
    FLUSH_INTERVAL, MAX_READ_BUFFER_SIZE, MIN_READ_BUFFER_SIZE, PROGRESS_REPORT_INTERVAL,
    WRITE_BUFFER_CAPACITY,
};

/// 檔案格式常數
const MAGIC_HEADER: &[u8] = b"EIGENVALS_V5"; // 12 bytes
const EOF_MARKER: &[u8] = b"EOF_MARK"; // 8 bytes

/// 計算預期檔案大小以便預先配置磁碟空間
pub fn calculate_expected_file_size(num_runs: usize, eigenvalues_per_run: usize) -> u64 {
    let header = MAGIC_HEADER.len() as u64 + 1 + 1 + 4; // magic + model(1) + dim(1) + steps(4)
    let record_size = 4 + 1 + eigenvalues_per_run as u64 * 8; // seed: 4 bytes (u32), count: 1 byte (u8)
    let metadata = EOF_MARKER.len() as u64 + 8 + 1; // eof_marker + total_count + eigenvalues_per_run(u8)
    header + record_size * num_runs as u64 + metadata
}

/// 根據檔案大小計算最佳讀取緩衝區大小
fn calculate_read_buffer_size(file_size: u64) -> usize {
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
        buffer_size.min(4 * 1024 * 1024).max(MIN_READ_BUFFER_SIZE)
    } else {
        MAX_READ_BUFFER_SIZE
    }
}

/// 追加寫入器 - 支援高效的數據追加和斷點續傳
pub struct AppendOnlyWriter {
    writer: BufWriter<File>,
    written_count: usize,
    eigenvalues_per_run: Option<usize>,
    model: u8,
    dim: u8,
    steps: u32,
    quiet: bool,
}

impl AppendOnlyWriter {
    /// 創建新的追加寫入器，並可選擇預先配置檔案大小
    pub fn with_expected_size<P: AsRef<Path>>(
        path: P,
        expected_size: Option<u64>,
        model: u8,
        dim: u8,
        steps: u32,
        quiet: bool,
    ) -> std::io::Result<Self> {
        let path_ref = path.as_ref();
        let is_new_file = !path_ref.exists();

        let mut written_count = 0;
        let mut eigenvalues_per_run = None;

        if is_new_file {
            // 新檔案：直接創建並寫入魔術標頭和元數據
            let file = OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .open(path_ref)?;

            // 如果指定了預期大小，預先分配檔案空間（暫時禁用以避免影響斷點續傳）
            if let Some(_size) = expected_size {
                if !quiet {
                    println!(
                        "File pre-allocation disabled to ensure resume functionality works correctly"
                    );
                }
            }

            let mut writer = BufWriter::with_capacity(WRITE_BUFFER_CAPACITY, file);
            writer.write_all(MAGIC_HEADER)?;
            writer.write_all(&model.to_le_bytes())?;
            writer.write_all(&dim.to_le_bytes())?;
            writer.write_all(&steps.to_le_bytes())?;
            writer.flush()?;

            Ok(Self {
                writer,
                written_count: 0,
                eigenvalues_per_run: None,
                model,
                dim,
                steps,
                quiet,
            })
        } else {
            // 既有檔案：檢查數據並移除 EOF 標記
            // 先讀取檔案內容來獲取計數 (保持原始容錯邏輯)
            match read_append_file(&path) {
                Ok((existing_data, file_model, file_dim, file_steps)) => {
                    // 驗證參數是否匹配
                    if file_model != model {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "Model mismatch: file has model {}, expected {}",
                                file_model, model
                            ),
                        ));
                    }
                    if file_dim != dim {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "Dimension mismatch: file has dim {}, expected {}",
                                file_dim, dim
                            ),
                        ));
                    }
                    if file_steps != steps {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "Steps mismatch: file has steps {}, expected {}",
                                file_steps, steps
                            ),
                        ));
                    }

                    written_count = existing_data.len();
                    if let Some((_, eigenvalues)) = existing_data.first() {
                        eigenvalues_per_run = Some(eigenvalues.len());
                    }
                    if !quiet {
                        println!(
                            "Detected existing file with {} data records",
                            format_number_with_commas(written_count)
                        );
                    }
                }
                Err(e)
                    if e.to_string()
                        .contains("File format error: magic header mismatch") =>
                {
                    // 文件格式不兼容，刪除舊文件並重新創建
                    if !quiet {
                        println!("WARNING: Incompatible file format detected, recreating file...");
                    }
                    std::fs::remove_file(&path)?;

                    // 重新創建新文件
                    let file = OpenOptions::new()
                        .create(true)
                        .read(true)
                        .write(true)
                        .open(path_ref)?;

                    let mut writer = BufWriter::with_capacity(WRITE_BUFFER_CAPACITY, file);
                    writer.write_all(MAGIC_HEADER)?;
                    writer.write_all(&model.to_le_bytes())?;
                    writer.write_all(&dim.to_le_bytes())?;
                    writer.write_all(&steps.to_le_bytes())?;
                    writer.flush()?;

                    return Ok(Self {
                        writer,
                        written_count: 0,
                        eigenvalues_per_run: None,
                        model,
                        dim,
                        steps,
                        quiet,
                    });
                }
                Err(_) => {
                    // 其他讀取錯誤，採用容錯策略
                    if !quiet {
                        println!(
                            "WARNING: Could not read existing file, will attempt to append..."
                        );
                    }
                }
            }

            // 然後移除 EOF 標記：打開檔案並截斷到數據結束位置
            let mut file = OpenOptions::new().read(true).write(true).open(path_ref)?;
            let file_len = file.metadata()?.len();

            // 檢查檔案結尾是否真的包含 EOF 標記
            if file_len >= 18 + 17 {
                // magic(12) + model(1) + dim(1) + steps(4) + eof_marker(8) + count(8) + eigenvalues_per_run(1) = 35
                file.seek(SeekFrom::End(-17))?; // eof_marker(8) + count(8) + eigenvalues_per_run(1) = 17
                let mut eof_buf = [0u8; 8];
                if let Ok(()) = file.read_exact(&mut eof_buf) {
                    if &eof_buf == EOF_MARKER {
                        let new_len = file_len - 17;
                        file.set_len(new_len)?;
                        if !quiet {
                            println!("Removed EOF marker to enable append mode");
                        }
                    }
                }
            }

            // 設置為追加模式
            let file = OpenOptions::new().append(true).open(path_ref)?;

            let writer = BufWriter::with_capacity(WRITE_BUFFER_CAPACITY, file);

            Ok(Self {
                writer,
                written_count,
                eigenvalues_per_run,
                model,
                dim,
                steps,
                quiet,
            })
        }
    }

    /// 追加特徵值數據
    pub fn append_eigenvalues(&mut self, seed: u32, eigenvalues: &[f64]) -> std::io::Result<()> {
        // 檢查特徵值數量是否在 u8 範圍內
        if eigenvalues.len() > u8::MAX as usize {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Too many eigenvalues: {} exceeds maximum of {}",
                    format_number_with_commas(eigenvalues.len()),
                    u8::MAX
                ),
            ));
        }

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
                        "Eigenvalue count mismatch: expected {}, actual {} (model {}, dim {}, steps {})",
                        format_number_with_commas(expected_len),
                        format_number_with_commas(eigenvalues.len()),
                        self.model,
                        self.dim,
                        self.steps
                    ),
                ));
            }
        }

        // 寫入數據塊：[seed: 4 bytes (u32)] [eigenvalue_count: 1 byte] [eigenvalues: count * 8 bytes]
        self.writer.write_all(&seed.to_le_bytes())?;
        self.writer
            .write_all(&(eigenvalues.len() as u8).to_le_bytes())?;

        for &val in eigenvalues {
            self.writer.write_all(&val.to_le_bytes())?;
        }

        self.written_count += 1;

        // 定期刷新緩衝區
        if self.written_count % FLUSH_INTERVAL == 0 {
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
            // 檢查 eigenvalues_per_run 是否在 u8 範圍內
            if eigenvalues_per_run > u8::MAX as usize {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Too many eigenvalues per run: {} exceeds maximum of {}",
                        format_number_with_commas(eigenvalues_per_run),
                        u8::MAX
                    ),
                ));
            }
            self.writer
                .write_all(&(eigenvalues_per_run as u8).to_le_bytes())?;
        } else {
            self.writer.write_all(&0u8.to_le_bytes())?;
        }

        self.writer.flush()?;

        if !self.quiet {
            println!(
                "SUCCESS: append write completed, wrote {} data records for model {}, dim {}, steps {}",
                format_number_with_commas(self.written_count),
                self.model,
                self.dim,
                self.steps
            );
        }

        Ok(())
    }
}

/// 讀取追加格式的檔案
pub fn read_append_file<P: AsRef<Path>>(
    path: P,
) -> std::io::Result<(Vec<(u32, Vec<f64>)>, u8, u8, u32)> {
    let file = File::open(&path)?;
    let file_size = file.metadata()?.len();

    // 根據檔案大小計算最佳緩衝區大小
    let buffer_size = calculate_read_buffer_size(file_size);
    let mut reader = BufReader::with_capacity(buffer_size, file);

    // 檢查魔術標頭
    let mut magic_buf = [0u8; 12];
    reader.read_exact(&mut magic_buf)?;
    if &magic_buf != MAGIC_HEADER {
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
    if &eof_buf != EOF_MARKER {
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
        let mut seed_buf = [0u8; 4]; // 改為 4 bytes (u32)
        let mut count_buf = [0u8; 1]; // 1 byte (u8)

        reader.read_exact(&mut seed_buf)?;
        reader.read_exact(&mut count_buf)?;

        let seed = u32::from_le_bytes(seed_buf);
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

    loop {
        let mut seed_buf = [0u8; 4]; // 改為 4 bytes (u32)
        let mut count_buf = [0u8; 1]; // 1 byte (u8)

        // 嘗試讀取 seed
        if reader.read_exact(&mut seed_buf).is_err() {
            break; // 到達檔案末尾
        }

        // 檢查是否是 EOF 標記
        // 由於 seed 現在是 4 bytes，而 EOF_MARKER 是 8 bytes，我們需要謹慎檢查
        if seed_buf == [b'E', b'O', b'F', b'_'] {
            // 可能是 EOF 標記的開始，檢查接下來的 4 字節
            let mut remaining_eof = [0u8; 4];
            if reader.read_exact(&mut remaining_eof).is_ok()
                && remaining_eof == [b'M', b'A', b'R', b'K']
            {
                break; // 確認是 EOF 標記
            } else {
                // 不是完整的 EOF 標記，回退並繼續處理
                reader.seek(SeekFrom::Current(-4))?;
            }
        }

        // 檢查是否是全零（預分配的空白區域）
        if seed_buf == [0u8; 4] {
            // 檢查後續是否也是零，如果是則認為到達了預分配的空白區域
            if reader.read_exact(&mut count_buf).is_ok() && count_buf == [0u8; 1] {
                break; // 遇到預分配的空白區域
            } else {
                // 如果不是全零的 count，則繼續處理（seed 為 0 是有效的）
                reader.seek(SeekFrom::Current(-1))?; // 回退 count_buf (1 byte)
            }
        }

        // 讀取特徵值數量
        if reader.read_exact(&mut count_buf).is_err() {
            break; // 不完整的數據塊
        }

        let seed = u32::from_le_bytes(seed_buf);
        let eigenvalue_count_u8 = u8::from_le_bytes(count_buf);
        let eigenvalue_count = eigenvalue_count_u8 as usize;

        // 檢查特徵值數量是否合理（u8 已經限制在 0-255 範圍內）
        if eigenvalue_count == 0 {
            break; // 零計數表示可能到達了預分配的空白區域
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

/// 檢查檔案進度（追加格式）並驗證參數匹配
pub fn check_append_progress<P: AsRef<Path>>(
    path: P,
    expected_model: u8,
    expected_dim: u8,
    expected_steps: u32,
) -> std::io::Result<(usize, Vec<u32>)> {
    if !path.as_ref().exists() {
        return Ok((0, Vec::new()));
    }

    match read_append_file(&path) {
        Ok((data, file_model, file_dim, file_steps)) => {
            // 驗證參數是否匹配
            if file_model != expected_model {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Model mismatch: file has model {}, expected {}",
                        file_model, expected_model
                    ),
                ));
            }
            if file_dim != expected_dim {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Dimension mismatch: file has dim {}, expected {}",
                        file_dim, expected_dim
                    ),
                ));
            }
            if file_steps != expected_steps {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Steps mismatch: file has steps {}, expected {}",
                        file_steps, expected_steps
                    ),
                ));
            }

            let completed_runs = data.len();
            let completed_seeds: Vec<u32> = data.iter().map(|(seed, _)| *seed).collect();
            Ok((completed_runs, completed_seeds))
        }
        Err(_) => Ok((0, Vec::new())), // 檔案損壞或無法讀取，重新開始
    }
}

/// 獲取尚未完成的seed列表
pub fn get_remaining_seeds(total_runs: usize, completed_seeds: &[u32]) -> Vec<u32> {
    let completed_set: HashSet<u32> = completed_seeds.iter().copied().collect();
    (1..=total_runs as u32)
        .filter(|seed| !completed_set.contains(seed))
        .collect()
}

/// 啟動追加寫入執行緒
pub fn spawn_append_writer_thread(
    filename: String,
    receiver: mpsc::Receiver<(u32, Vec<f64>)>,
    total_runs: usize,
    completed_runs: usize,
    dim: usize,
    steps: usize,
    model: crate::johansen_models::JohansenModel,
    quiet: bool,
) -> thread::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>> {
    thread::spawn(move || {
        let eigenvalues_per_run = match model {
            crate::johansen_models::JohansenModel::InterceptNoTrendWithInterceptInCoint
            | crate::johansen_models::JohansenModel::InterceptTrendWithTrendInCoint => dim + 1,
            _ => dim,
        };

        let expected_size = calculate_expected_file_size(total_runs, eigenvalues_per_run);

        let mut writer = AppendOnlyWriter::with_expected_size(
            &filename,
            Some(expected_size),
            model.to_number(),
            dim as u8,
            steps as u32,
            quiet,
        )?;
        let mut count = 0;
        let start_time = std::time::Instant::now();

        while let Ok((seed, eigenvalues)) = receiver.recv() {
            writer.append_eigenvalues(seed, &eigenvalues)?;
            count += 1;

            let current_total = completed_runs + count;
            if current_total % PROGRESS_REPORT_INTERVAL == 0 && !quiet {
                let progress_ratio = current_total as f64 / total_runs as f64;
                let elapsed = start_time.elapsed();

                if progress_ratio > 0.0 {
                    // 計算剩餘時間時，只使用當前執行的進度和時間
                    let remaining_runs = total_runs - completed_runs;
                    println!(
                        "Simulation progress: {}/{} ({:.2}%) - {}",
                        format_number_with_commas(current_total),
                        format_number_with_commas(total_runs),
                        progress_ratio * 100.0,
                        format_remaining_time(elapsed, count, remaining_runs)
                    );
                } else {
                    println!(
                        "Simulation progress: {}/{} ({:.2}%)",
                        format_number_with_commas(current_total),
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
