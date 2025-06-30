use super::append_writer::{
    check_append_progress, get_remaining_seeds, read_append_file, spawn_append_writer_thread,
};
use super::config::BATCH_SIZE;
use crate::display_utils::format_number_with_commas;
use crate::johansen_models::JohansenModel;
use crate::johansen_statistics::calculate_eigenvalues;
use rayon::prelude::*;
use std::sync::mpsc;
use std::thread;

/// 啟動統計收集執行緒
fn spawn_statistics_collector(
    statistics_receiver: mpsc::Receiver<f64>,
) -> thread::JoinHandle<Vec<f64>> {
    thread::spawn(move || {
        let mut eigenvalue_sums = Vec::new();
        while let Ok(sum) = statistics_receiver.recv() {
            eigenvalue_sums.push(sum);
        }
        eigenvalue_sums
    })
}

/// 使用指定seeds進行並行計算
fn calculate_eigenvalues_parallel(
    dim: usize,
    steps: usize,
    seeds: &[u64],
    model: JohansenModel,
    sender: mpsc::Sender<(u64, Vec<f64>)>,
    statistics_sender: mpsc::Sender<f64>,
    quiet: bool,
) {
    let chunk_size = BATCH_SIZE;
    let total_seeds = seeds.len();
    let total_chunks = (total_seeds + chunk_size - 1) / chunk_size;

    for chunk_idx in 0..total_chunks {
        let chunk_start = chunk_idx * chunk_size;
        let chunk_end = ((chunk_idx + 1) * chunk_size).min(total_seeds);
        let chunk_seeds = &seeds[chunk_start..chunk_end];

        // 注意：進度報告已移至 append_writer，避免重複顯示
        // if !quiet {
        //     let progress_ratio = chunk_end as f64 / total_seeds as f64;
        //
        //     if progress_ratio > 0.0 {
        //         println!(
        //             "處理進度: {}/{} ({:.1}%) - {}",
        //             chunk_end,
        //             total_seeds,
        //             progress_ratio * 100.0,
        //             format_remaining_time(elapsed, chunk_end, total_seeds)
        //         );
        //     } else {
        //         println!(
        //             "處理進度: {}/{} ({:.1}%)",
        //             chunk_end,
        //             total_seeds,
        //             progress_ratio * 100.0
        //         );
        //     }
        // }

        // 並行計算這個chunk的結果
        chunk_seeds.into_par_iter().for_each(|&seed| {
            let eigenvalues = calculate_eigenvalues(dim, steps, seed, model);
            let eigenvalue_sum = eigenvalues.iter().sum::<f64>();

            // 發送結果給寫入執行緒
            if let Err(_) = sender.send((seed, eigenvalues)) {
                if !quiet {
                    eprintln!("發送結果到寫入執行緒失敗");
                }
            }

            // 發送統計資料
            if let Err(_) = statistics_sender.send(eigenvalue_sum) {
                if !quiet {
                    eprintln!("發送統計資料失敗");
                }
            }
        });
    }
}

/// 驗證檔案寫入結果
fn validate_output_file(filename: &str, expected_count: usize) {
    match read_append_file(filename) {
        Ok(loaded_data) => {
            if loaded_data.len() == expected_count {
                println!("SUCCESS: 追加檔案驗證成功");
            } else {
                println!(
                    "ERROR: 追加檔案驗證失敗：數據長度不匹配 (期待: {}, 實際: {})",
                    expected_count,
                    loaded_data.len()
                );
            }
        }
        Err(e) => println!("ERROR: 讀取追加檔案失敗: {}", e),
    }
}

/// 輸出百分位數統計資訊
fn print_percentile_statistics(sorted_eigenvalues: &[f64], percentiles: &[f64]) {
    println!(
        "總共計算了 {} 個特徵值總和",
        format_number_with_commas(sorted_eigenvalues.len())
    );

    // 輸出各個百分位數
    for &percentile in percentiles {
        let index = ((sorted_eigenvalues.len() as f64) * percentile) as usize;
        let value = sorted_eigenvalues[index.min(sorted_eigenvalues.len() - 1)];
        println!("第{:.0}百分位的值: {:.6}", percentile * 100.0, value);
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
    run_single_model_simulation(dim, steps, num_runs, get_filename_fn, model, quiet);
}

/// 支援斷點續傳的單一模型模擬計算（內部實現）
fn run_single_model_simulation(
    dim: usize,
    steps: usize,
    num_runs: usize,
    get_filename_fn: impl Fn(JohansenModel) -> String,
    model: JohansenModel,
    quiet: bool,
) {
    if !quiet {
        println!("使用模型: {} (支援斷點續傳)", model);
    }

    let filename = get_filename_fn(model);

    // 檢查已完成的進度
    match check_append_progress(&filename) {
        Ok((completed_runs, completed_seeds)) => {
            if completed_runs >= num_runs {
                if !quiet {
                    println!("SUCCESS: 此模型的計算已完成，跳過");
                }
                return;
            }

            if completed_runs > 0 && !quiet {
                let max_completed_seed = completed_seeds.iter().max().copied().unwrap_or(0);
                let min_completed_seed = completed_seeds.iter().min().copied().unwrap_or(0);
                if !quiet {
                    println!(
                        "檢測到已完成 {}/{} 次計算，Seeds 範圍: {}-{}",
                        completed_runs, num_runs, min_completed_seed, max_completed_seed
                    );
                }
            }

            // 獲取剩餘的seed
            let remaining_seeds = get_remaining_seeds(num_runs, &completed_seeds);
            let remaining_count = remaining_seeds.len();

            if remaining_count == 0 {
                if !quiet {
                    println!("SUCCESS: 此模型的計算已完成");
                }
                return;
            }

            if !quiet {
                println!(
                    "剩餘 {} 次計算需要完成",
                    format_number_with_commas(remaining_count)
                );
            }

            // 設置 channels
            let (sender, receiver) = mpsc::channel::<(u64, Vec<f64>)>();
            let (statistics_sender, statistics_receiver) = mpsc::channel::<f64>();

            // 啟動支援斷點續傳的寫入執行緒
            let writer_handle =
                spawn_append_writer_thread(filename.clone(), receiver, num_runs, quiet);
            let statistics_handle = spawn_statistics_collector(statistics_receiver);

            // 執行剩餘的並行計算
            calculate_eigenvalues_parallel(
                dim,
                steps,
                &remaining_seeds,
                model,
                sender,
                statistics_sender,
                quiet,
            );

            // 等待寫入執行緒完成
            match writer_handle.join() {
                Ok(Ok(())) => {
                    if !quiet {
                        println!("已儲存到 {}", filename);
                    }
                }
                Ok(Err(e)) => {
                    if !quiet {
                        eprintln!("寫入執行緒出錯: {}", e);
                    }
                }
                Err(_) => {
                    if !quiet {
                        eprintln!("寫入執行緒 panic");
                    }
                }
            }

            // 收集並處理統計資料
            match statistics_handle.join() {
                Ok(mut eigenvalue_sums) => {
                    if !eigenvalue_sums.is_empty() && !quiet {
                        eigenvalue_sums.sort_by(|a, b| a.partial_cmp(b).unwrap());
                        let percentiles = vec![0.5, 0.75, 0.8, 0.85, 0.9, 0.95, 0.975, 0.99];
                        print_percentile_statistics(&eigenvalue_sums, &percentiles);
                    }
                }
                Err(_) => {
                    if !quiet {
                        eprintln!("統計執行緒 panic");
                    }
                }
            }

            // 驗證檔案輸出
            if !quiet {
                validate_output_file(&filename, num_runs);
            }
        }
        Err(e) => {
            if !quiet {
                eprintln!("檢查進度失敗: {}", e);
            }
        }
    }

    if !quiet {
        println!("===============================\n");
    }
}

/// 進行大規模模擬的主函數
#[allow(dead_code)]
pub fn run_large_scale_simulation(
    dim: usize,
    steps: usize,
    num_runs: usize,
    get_filename_fn: impl Fn(JohansenModel) -> String + Copy,
    quiet: bool,
) {
    if !quiet {
        println!("開始大規模模擬 - 支援斷點續傳");
        println!(
            "維度: {}, 步數: {}, 運行次數: {}",
            dim,
            format_number_with_commas(steps),
            format_number_with_commas(num_runs)
        );
        println!(
            "支援的模型: {}",
            JohansenModel::all_models()
                .iter()
                .map(|m| format!("{}", m))
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!("===============================");
    }

    // 計算所有模型
    for model in JohansenModel::all_models() {
        run_model_simulation(dim, steps, num_runs, get_filename_fn, model, quiet);
    }
}
