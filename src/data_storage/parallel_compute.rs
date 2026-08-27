//! Parallel simulation engine.
//!
//! rayon worker 計算完成後直接以定位寫入將結果寫進各自的槽位
//! （槽位互不重疊，無需鎖），取代舊版的 channel + 寫入執行緒架構。
//! 以批次為單位同步落盤並回報進度；批次大小依實測耗時
//! 向目標批次時間自適應收斂（見 `config` 模組的批次常數）。

use rayon::prelude::*;
use std::time::Instant;

use super::config::{
    BATCH_FLOOR_PER_THREAD, BATCH_SMOOTHING, CHECKPOINT_DIVISOR, TARGET_BATCH_SECS,
};
use super::error::StorageError;
use super::simulation::EigenvalueSimulation;
use super::writer::{OpenResult, SlotWriter};
use crate::display_utils::{format_number_with_commas, format_remaining_time};
use crate::johansen_statistics::calculate_eigenvalues;

/// 支援斷點續傳的單一模型模擬計算
pub fn run_model_simulation(simulation: &EigenvalueSimulation, quiet: bool) {
    if !quiet {
        println!(
            "Using model: {} (supports resuming from checkpoint)",
            simulation.model
        );
    }

    let mut recreated = false;
    loop {
        match try_run(simulation, quiet) {
            Ok(()) => break,
            Err(e) if e.is_recreatable() && !recreated => {
                // 檔案格式或參數不相容：資料可由 seed 重算，刪除後重建一次
                if !quiet {
                    println!("WARNING: Existing file is incompatible ({e})");
                    println!("  The file will be removed and recreated with correct parameters.");
                }
                simulation.remove_data_files();
                recreated = true;
            }
            Err(e) => panic!("Simulation failed: {e}"),
        }
    }

    if !quiet {
        println!("===============================\n");
    }
}

/// 單次嘗試：打開（或建立）檔案並補齊缺少的槽位
fn try_run(simulation: &EigenvalueSimulation, quiet: bool) -> Result<(), StorageError> {
    let data_path = simulation.get_file_path(simulation.model);

    let (writer, remaining_seeds, completed_runs) =
        match SlotWriter::open(&data_path, simulation.file_header(), quiet)? {
            OpenResult::AlreadyComplete => {
                if !quiet {
                    println!("SUCCESS: calculation for this model already completed, skipping");
                }
                return Ok(());
            }
            OpenResult::InProgress {
                writer,
                remaining_seeds,
                completed_runs,
            } => (writer, remaining_seeds, completed_runs),
        };

    if !quiet {
        println!(
            "Remaining {} calculations to complete",
            format_number_with_commas(remaining_seeds.len())
        );
    }

    let start_time = Instant::now();
    let total_remaining = remaining_seeds.len();
    let mut processed = 0usize;

    // 批次大小界限：上限由任務規模導出，下限由機器規模導出，衝突時上限優先
    let batch_max = simulation.num_runs.div_ceil(CHECKPOINT_DIVISOR).max(1);
    let batch_min = (rayon::current_num_threads() * BATCH_FLOOR_PER_THREAD).min(batch_max);
    let mut batch_size = batch_min;

    while processed < total_remaining {
        let chunk_end = (processed + batch_size).min(total_remaining);
        let chunk = &remaining_seeds[processed..chunk_end];
        let batch_start = Instant::now();

        // 並行計算並直接寫入各自的槽位
        chunk.par_iter().try_for_each(|&seed| {
            let eigenvalues =
                calculate_eigenvalues(simulation.dim, simulation.steps, seed, simulation.model);
            writer.write_slot(seed, &eigenvalues)
        })?;

        // 每批落盤一次：斷電最多損失最近一批，由 CRC 掃描偵測後重算
        writer.sync()?;
        processed = chunk_end;

        // 依實測耗時向目標批次時間修正下一批的大小
        let measured_secs = batch_start.elapsed().as_secs_f64().max(1e-9);
        let correction = TARGET_BATCH_SECS / measured_secs;
        let next = batch_size as f64 * (1.0 - BATCH_SMOOTHING + BATCH_SMOOTHING * correction);
        batch_size = (next.round() as usize).clamp(batch_min, batch_max);

        if !quiet {
            let current_total = completed_runs + processed;
            let progress_ratio = current_total as f64 / simulation.num_runs as f64;
            println!(
                "Simulation progress: {}/{} ({:.2}%) - {}",
                format_number_with_commas(current_total),
                format_number_with_commas(simulation.num_runs),
                progress_ratio * 100.0,
                format_remaining_time(start_time.elapsed(), processed, total_remaining)
            );
        }
    }

    writer.finish()?;

    if !quiet {
        println!("Saved to {}", data_path.display());
    }
    Ok(())
}
