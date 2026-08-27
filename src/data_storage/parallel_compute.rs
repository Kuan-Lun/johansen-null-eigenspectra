//! Parallel simulation engine.
//!
//! rayon worker 計算完成後直接以定位寫入將結果寫進各自的槽位
//! （槽位互不重疊，無需鎖），取代舊版的 channel + 寫入執行緒架構。
//! 以批次為單位同步落盤並透過回呼回報進度；批次大小依實測耗時
//! 向目標批次時間自適應收斂（見 `config` 模組的批次常數）。

use rayon::prelude::*;
use std::time::Instant;

use super::config::{
    BATCH_FLOOR_PER_THREAD, BATCH_SMOOTHING, CHECKPOINT_DIVISOR, TARGET_BATCH_SECS,
};
use super::simulation::EigenvalueSimulation;
use super::writer::{OpenResult, SlotWriter};
use crate::error::Error;
use crate::johansen_statistics::calculate_eigenvalues;

/// 模擬進度，於開檔完成後與每批落盤後回報
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    /// 已完成（已寫入並落盤）的模擬次數，含先前執行留下的成果
    pub completed: u64,
    /// 目標模擬次數（即配置的 `num_runs`）
    pub total: u64,
}

/// 支援斷點續傳的單一模型模擬計算。
///
/// 既有檔案與配置不相容時回傳錯誤，由呼叫端決定是否刪檔重建；
/// 本函數不輸出任何內容，進度一律透過 `on_progress` 回報。
pub fn run_model_simulation(
    simulation: &EigenvalueSimulation,
    mut on_progress: impl FnMut(Progress),
) -> Result<(), Error> {
    let total = simulation.num_runs() as u64;
    let data_path = simulation.data_path();

    let (writer, remaining_seeds, completed_runs) =
        match SlotWriter::open(&data_path, simulation.file_header())? {
            OpenResult::AlreadyComplete => {
                on_progress(Progress {
                    completed: total,
                    total,
                });
                return Ok(());
            }
            OpenResult::InProgress {
                writer,
                remaining_seeds,
                completed_runs,
            } => (writer, remaining_seeds, completed_runs),
        };

    // 首次回報：斷點續傳的起始點
    on_progress(Progress {
        completed: completed_runs as u64,
        total,
    });

    let total_remaining = remaining_seeds.len();
    let mut processed = 0usize;

    // 批次大小界限：上限由任務規模導出，下限由機器規模導出，衝突時上限優先
    let batch_max = simulation.num_runs().div_ceil(CHECKPOINT_DIVISOR).max(1);
    let batch_min = (rayon::current_num_threads() * BATCH_FLOOR_PER_THREAD).min(batch_max);
    let mut batch_size = batch_min;

    while processed < total_remaining {
        let chunk_end = (processed + batch_size).min(total_remaining);
        let chunk = &remaining_seeds[processed..chunk_end];
        let batch_start = Instant::now();

        // 並行計算並直接寫入各自的槽位
        chunk.par_iter().try_for_each(|&seed| {
            let eigenvalues = calculate_eigenvalues(
                simulation.dim(),
                simulation.steps(),
                seed,
                simulation.model(),
            );
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

        on_progress(Progress {
            completed: (completed_runs + processed) as u64,
            total,
        });
    }

    writer.finish()?;
    Ok(())
}
