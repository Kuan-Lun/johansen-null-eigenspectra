use super::config::BATCH_SIZE;
use super::progress::{check_append_progress, get_remaining_seeds};
use super::reader::read_append_file;
use super::thread_manager::spawn_append_writer_thread;
use crate::display_utils::format_number_with_commas;
use crate::johansen_models::JohansenModel;
use crate::johansen_statistics::calculate_eigenvalues;
use rayon::prelude::*;
use std::sync::mpsc;

/// 使用指定seeds進行並行計算
fn calculate_eigenvalues_parallel(
    dim: usize,
    steps: usize,
    seeds: &[u32],
    model: JohansenModel,
    sender: mpsc::Sender<(u32, Vec<f64>)>,
    quiet: bool,
) {
    let chunk_size = BATCH_SIZE;
    let total_seeds = seeds.len();
    let total_chunks = total_seeds.div_ceil(chunk_size);

    for chunk_idx in 0..total_chunks {
        let chunk_start = chunk_idx * chunk_size;
        let chunk_end = ((chunk_idx + 1) * chunk_size).min(total_seeds);
        let chunk_seeds = &seeds[chunk_start..chunk_end];

        // 並行計算這個chunk的結果
        chunk_seeds.into_par_iter().for_each(|&seed| {
            let eigenvalues = calculate_eigenvalues(dim, steps, seed, model);

            // 發送結果給寫入執行緒
            if sender.send((seed, eigenvalues)).is_err() && !quiet {
                eprintln!("Failed to send results to writer thread");
            }
        });
    }
}

/// 驗證檔案寫入結果
fn validate_output_file(filename: &str, expected_count: usize) {
    match read_append_file(filename) {
        Ok((loaded_data, _model, _dim, _steps)) => {
            if loaded_data.len() == expected_count {
                println!("SUCCESS: append file validation successful");
            } else {
                println!(
                    "ERROR: append file validation failed: data length mismatch (expected: {}, actual: {})",
                    expected_count,
                    loaded_data.len()
                );
            }
        }
        Err(e) => {
            // 對於魔術標頭不匹配這類嚴重的文件格式錯誤，應該 panic
            if e.to_string()
                .contains("File format error: magic header mismatch")
            {
                panic!("CRITICAL ERROR: File format incompatibility detected - {e}");
            } else {
                println!("ERROR: failed to read append file: {e}");
            }
        }
    }
}

/// 支援斷點續傳的單一模型模擬計算
pub fn run_model_simulation(
    dim: usize,
    steps: usize,
    num_runs: usize,
    get_filename_fn: impl Fn(JohansenModel) -> String,
    model: JohansenModel,
    quiet: bool,
) {
    if !quiet {
        println!("Using model: {model} (supports resuming from checkpoint)");
    }

    let filename = get_filename_fn(model);

    // 檢查已完成的進度
    match check_append_progress(&filename, model.to_number(), dim as u8, steps as u32) {
        Ok((completed_runs, completed_seeds)) => {
            if completed_runs >= num_runs {
                if !quiet {
                    println!("SUCCESS: calculation for this model already completed, skipping");
                }
                return;
            }

            if completed_runs > 0 && !quiet {
                let max_completed_seed = completed_seeds.iter().max().copied().unwrap_or(0);
                let min_completed_seed = completed_seeds.iter().min().copied().unwrap_or(0);
                if !quiet {
                    println!(
                        "Detected {} completed out of {} calculations, Seeds range: {}-{}",
                        format_number_with_commas(completed_runs),
                        format_number_with_commas(num_runs),
                        format_number_with_commas(min_completed_seed as usize),
                        format_number_with_commas(max_completed_seed as usize)
                    );
                }
            }

            // 獲取剩餘的seed
            let remaining_seeds = get_remaining_seeds(num_runs, &completed_seeds);
            let remaining_count = remaining_seeds.len();

            if remaining_count == 0 {
                if !quiet {
                    println!("SUCCESS: calculation for this model already completed");
                }
                return;
            }

            if !quiet {
                println!(
                    "Remaining {} calculations to complete",
                    format_number_with_commas(remaining_count)
                );
            }

            // 設置 channels
            let (sender, receiver) = mpsc::channel::<(u32, Vec<f64>)>();

            // 啟動支援斷點續傳的寫入執行緒
            let writer_config = crate::data_storage::thread_manager::WriterConfig {
                filename: filename.clone(),
                total_runs: num_runs,
                completed_runs,
                dim,
                steps,
                model,
                quiet,
            };
            let writer_handle = spawn_append_writer_thread(writer_config, receiver);

            // 執行剩餘的並行計算
            calculate_eigenvalues_parallel(dim, steps, &remaining_seeds, model, sender, quiet);

            // 等待寫入執行緒完成
            match writer_handle.join() {
                Ok(Ok(())) => {
                    if !quiet {
                        println!("Saved to {filename}");
                    }
                }
                Ok(Err(e)) => {
                    panic!("Writer thread error: {e}");
                }
                Err(_) => {
                    panic!("Writer thread panic");
                }
            }

            // 驗證檔案輸出
            if !quiet {
                validate_output_file(&filename, num_runs);
            }
        }
        Err(e) => {
            // 檢查是否是參數不匹配錯誤
            let error_msg = e.to_string();
            if error_msg.contains("mismatch") {
                if !quiet {
                    println!("WARNING: Existing file has incompatible parameters:");
                    println!("  {error_msg}");
                    println!(
                        "  The existing file will be removed and recreated with correct parameters."
                    );
                }
                // 刪除不兼容的檔案
                if let Err(remove_err) = std::fs::remove_file(&filename) {
                    if !quiet {
                        println!("WARNING: Failed to remove incompatible file: {remove_err}");
                    }
                }
                // 重新開始計算
                if !quiet {
                    println!("Starting fresh calculation with correct parameters...");
                }
                // 重新調用自己來重新開始計算
                return run_model_simulation(dim, steps, num_runs, get_filename_fn, model, quiet);
            } else {
                panic!("Failed to check progress: {e}");
            }
        }
    }

    if !quiet {
        println!("===============================\n");
    }
}
