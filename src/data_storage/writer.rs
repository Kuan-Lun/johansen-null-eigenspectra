//! 追加寫入器 - 高性能的數據寫入
//!
//! 實現真正的追加寫入，避免每次都重寫整個檔案

use crate::display_utils::format_number_with_commas;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::Path;

use super::config::{FLUSH_INTERVAL, WRITE_BUFFER_CAPACITY};
use super::file_format::{EOF_MARKER, MAGIC_HEADER};
use super::reader::read_append_file;

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
                .truncate(true)
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
                                "Model mismatch: file has model {file_model}, expected {model}"
                            ),
                        ));
                    }
                    if file_dim != dim {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Dimension mismatch: file has dim {file_dim}, expected {dim}"),
                        ));
                    }
                    if file_steps != steps {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "Steps mismatch: file has steps {file_steps}, expected {steps}"
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
                        .truncate(true)
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
            Self::remove_eof_marker(path_ref, quiet)?;

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

    /// 移除 EOF 標記以啟用追加模式
    fn remove_eof_marker<P: AsRef<Path>>(path: P, quiet: bool) -> std::io::Result<()> {
        use std::io::Read;

        let mut file = OpenOptions::new().read(true).write(true).open(path)?;
        let file_len = file.metadata()?.len();

        // 檢查檔案結尾是否真的包含 EOF 標記
        if file_len >= 18 + 17 {
            // magic(12) + model(1) + dim(1) + steps(4) + eof_marker(8) + count(8) + eigenvalues_per_run(1) = 35
            file.seek(SeekFrom::End(-17))?; // eof_marker(8) + count(8) + eigenvalues_per_run(1) = 17
            let mut eof_buf = [0u8; 8];
            if let Ok(()) = file.read_exact(&mut eof_buf) {
                if eof_buf == EOF_MARKER {
                    let new_len = file_len - 17;
                    file.set_len(new_len)?;
                    if !quiet {
                        println!("Removed EOF marker to enable append mode");
                    }
                }
            }
        }
        Ok(())
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
