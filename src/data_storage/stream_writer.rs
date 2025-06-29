//! 基礎流式寫入器
//!
//! 提供高效的二進制數據流式寫入功能，支持批量刷新以優化性能。

use std::fs::File;
use std::io::{BufWriter, Seek, Write};
use std::path::Path;

/// 用於流式寫入特徵值的結構體（包含seed）
///
/// 這個寫入器會：
/// 1. 預留文件頭位置寫入元數據
/// 2. 流式寫入數據，定期刷新緩衝區
/// 3. 完成時回頭更新文件頭的計數信息
pub struct EigenvaluesStreamWriter {
    writer: BufWriter<File>,
    num_runs_written: u64,
    eigenvalues_per_run: Option<usize>,
    file_position_for_count: u64,
}

impl EigenvaluesStreamWriter {
    /// 創建新的流式寫入器
    ///
    /// # 參數
    /// - `path`: 目標文件路徑
    ///
    /// # 文件格式
    /// 創建的文件將遵循以下格式：
    /// - 8 bytes: 總運行次數 (初始為0，完成時更新)
    /// - 8 bytes: 每次運行的特徵值數量
    /// - 數據部分：每次運行的 seed + 特徵值
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // 預留位置寫入數據的總數量（稍後會回來更新這個值）
        let file_position_for_count = 0u64;
        writer.write_all(&0u64.to_le_bytes())?; // 暫時寫入0，稍後會更新

        Ok(Self {
            writer,
            num_runs_written: 0,
            eigenvalues_per_run: None,
            file_position_for_count,
        })
    }

    /// 寫入一組特徵值數據（包含seed）
    ///
    /// # 參數
    /// - `seed`: 隨機數種子
    /// - `eigenvalues`: 特徵值向量
    ///
    /// # 錯誤
    /// 如果特徵值數量與之前寫入的不一致，會返回錯誤
    pub fn write_eigenvalues(&mut self, seed: u64, eigenvalues: &[f64]) -> std::io::Result<()> {
        // 如果是第一次寫入，記錄特徵值的數量
        if self.eigenvalues_per_run.is_none() {
            self.eigenvalues_per_run = Some(eigenvalues.len());
            self.writer
                .write_all(&(eigenvalues.len() as u64).to_le_bytes())?;
        }

        // 檢查特徵值數量是否一致
        if let Some(expected_len) = self.eigenvalues_per_run {
            if eigenvalues.len() != expected_len {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "特徵值數量不一致: 期待 {}, 實際 {}",
                        expected_len,
                        eigenvalues.len()
                    ),
                ));
            }
        }

        // 寫入seed
        self.writer.write_all(&seed.to_le_bytes())?;

        // 寫入特徵值
        for &val in eigenvalues {
            self.writer.write_all(&val.to_le_bytes())?;
        }

        self.num_runs_written += 1;

        // 每寫入一定數量的數據就flush一次，避免緩衝區佔用太多記憶體
        if self.num_runs_written % 1000 == 0 {
            self.writer.flush()?;
        }

        Ok(())
    }

    /// 完成寫入並關閉檔案
    ///
    /// 這個方法會：
    /// 1. 刷新所有緩衝區
    /// 2. 回到文件開頭更新總計數
    /// 3. 確保所有數據都寫入磁盤
    pub fn finish(mut self) -> std::io::Result<()> {
        // 刷新緩衝區
        self.writer.flush()?;

        // 只有在非追加模式下才更新檔案開頭的計數
        if self.file_position_for_count != u64::MAX {
            // 回到檔案開頭，更新總數量
            let mut file = self.writer.into_inner()?;
            file.seek(std::io::SeekFrom::Start(self.file_position_for_count))?;
            file.write_all(&self.num_runs_written.to_le_bytes())?;
            file.flush()?;
        }

        Ok(())
    }
}
