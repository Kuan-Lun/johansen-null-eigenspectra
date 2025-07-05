//! 執行緒管理器 - 負責寫入執行緒的配置和管理
//!
//! 提供了寫入執行緒的配置結構體和生成函數

use crate::display_utils::{format_number_with_commas, format_remaining_time};
use std::sync::mpsc;
use std::thread;

use super::config::PROGRESS_REPORT_INTERVAL;
use super::file_format::calculate_expected_file_size;
use super::writer::AppendOnlyWriter;

/// 寫入執行緒配置
pub struct WriterConfig {
    pub filename: String,
    pub total_runs: usize,
    pub completed_runs: usize,
    pub dim: usize,
    pub steps: usize,
    pub model: crate::johansen_models::JohansenModel,
    pub quiet: bool,
}

/// 啟動追加寫入執行緒
pub fn spawn_append_writer_thread(
    config: WriterConfig,
    receiver: mpsc::Receiver<(u32, Vec<f64>)>,
) -> thread::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>> {
    thread::spawn(move || {
        let WriterConfig {
            filename,
            total_runs,
            completed_runs,
            dim,
            steps,
            model,
            quiet,
        } = config;

        let eigenvalues_per_run = match model {
            crate::johansen_models::JohansenModel::InterceptNoTrendWithInterceptInCoint
            | crate::johansen_models::JohansenModel::InterceptTrendUnrestrictedInterceptRestrictedTrend => dim + 1,
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
