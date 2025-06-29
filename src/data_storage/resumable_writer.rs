use std::collections::HashSet;
use std::path::Path;
use std::sync::mpsc;
use std::thread;

use super::binary_io::read_binary_file;
use super::stream_writer::EigenvaluesStreamWriter;

/// 檢查檔案的進度，返回 (已完成運行數, 已完成的seed列表)
pub fn check_progress<P: AsRef<Path>>(path: P) -> std::io::Result<(usize, Vec<u64>)> {
    if !path.as_ref().exists() {
        return Ok((0, Vec::new())); // 檔案不存在，進度為0，沒有已完成的seeds
    }

    match read_binary_file(&path) {
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
    let expected_seeds: HashSet<u64> = (1..=total_runs as u64).collect();
    let completed_set: HashSet<u64> = completed_seeds.iter().cloned().collect();

    let mut remaining: Vec<u64> = expected_seeds.difference(&completed_set).cloned().collect();
    remaining.sort(); // 確保順序一致
    remaining
}

/// 支援斷點續傳的流式寫入器
pub struct ResumableStreamWriter {
    final_filename: String,
    completed_runs: usize,
    is_resuming: bool,
    pending_data: Vec<(u64, Vec<f64>)>,
    quiet: bool,
}

impl ResumableStreamWriter {
    /// 創建可恢復的流式寫入器
    pub fn new<P: AsRef<Path>>(
        final_path: P,
        _total_expected_runs: usize,
        quiet: bool,
    ) -> std::io::Result<Self> {
        let final_filename = final_path.as_ref().to_string_lossy().to_string();

        // 檢查是否已有進度
        let (completed_runs, _) = check_progress(&final_filename)?;
        let is_resuming = completed_runs > 0;

        if is_resuming && !quiet {
            println!("檢測到已完成 {} 次計算，將繼續從中斷處開始", completed_runs);
        }

        Ok(Self {
            final_filename,
            completed_runs,
            is_resuming,
            pending_data: Vec::new(),
            quiet,
        })
    }

    /// 寫入特徵值數據
    pub fn write_eigenvalues(&mut self, seed: u64, eigenvalues: &[f64]) -> std::io::Result<()> {
        // 暫存新數據
        self.pending_data.push((seed, eigenvalues.to_vec()));
        self.completed_runs += 1;

        // 每1000次或積累一定數量時保存進度
        if self.pending_data.len() >= 1000 {
            self.flush_pending_data()?;
        }

        Ok(())
    }

    /// 刷新暫存的數據到檔案 - 無序高效版本
    fn flush_pending_data(&mut self) -> std::io::Result<()> {
        if self.pending_data.is_empty() {
            return Ok(());
        }

        // 讀取現有數據（保持原始順序，不排序）
        let mut all_data = if std::path::Path::new(&self.final_filename).exists() {
            read_binary_file(&self.final_filename)?
        } else {
            Vec::new()
        };

        // 直接合併新數據（不排序，保持並行計算的完成順序）
        all_data.extend(self.pending_data.drain(..));

        // 重寫整個檔案（無需排序）
        self.rewrite_entire_file(&all_data)?;

        if !self.quiet {
            println!("已保存進度: {} 次計算", all_data.len());
        }
        Ok(())
    }

    /// 重寫整個檔案
    fn rewrite_entire_file(&self, data: &[(u64, Vec<f64>)]) -> std::io::Result<()> {
        let mut writer = EigenvaluesStreamWriter::new(&self.final_filename)?;

        for (seed, eigenvalues) in data {
            writer.write_eigenvalues(*seed, eigenvalues)?;
        }

        writer.finish()
    }

    /// 完成寫入
    pub fn finish(mut self) -> std::io::Result<()> {
        // 保存剩餘的暫存數據
        self.flush_pending_data()?;

        if !self.quiet {
            if self.is_resuming {
                println!(
                    "SUCCESS: 斷點續傳完成，所有數據已安全保存到 {}",
                    self.final_filename
                );
            } else {
                println!("SUCCESS: 所有數據已安全保存到 {}", self.final_filename);
            }
        }
        Ok(())
    }

    /// 獲取已完成的運行次數
    pub fn completed_runs(&self) -> usize {
        self.completed_runs
    }
}

/// 啟動支援斷點續傳的寫入執行緒
pub fn spawn_writer_thread(
    filename: String,
    receiver: mpsc::Receiver<(u64, Vec<f64>)>,
    total_runs: usize,
    quiet: bool,
) -> thread::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>> {
    thread::spawn(move || {
        let mut binary_writer = ResumableStreamWriter::new(&filename, total_runs, quiet)?;
        let mut count = 0;

        while let Ok((seed, eigenvalues)) = receiver.recv() {
            binary_writer.write_eigenvalues(seed, &eigenvalues)?;
            count += 1;

            if count % 1000 == 0 && !quiet {
                println!(
                    "已寫入 {} 個結果 (總完成: {})",
                    count,
                    binary_writer.completed_runs()
                );
            }
        }

        binary_writer.finish()?;
        Ok(())
    })
}
