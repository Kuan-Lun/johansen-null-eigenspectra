//! CLI binary：透過 crate 的公開 API 執行模擬並呈現結果。
//!
//! 進度輸出、統計表格與「不相容檔案刪檔重建」政策都屬於應用程式職責，
//! 在此實作；函式庫本身不輸出任何內容，也不主動刪檔。

mod cli;
mod display_utils;

use cli::CliArgs;
use display_utils::{
    display_percentiles_table, format_duration, format_number_with_commas, format_remaining_time,
};
use johansen_null_eigenspectra::{EigenvalueSimulation, JohansenModel, Progress, Statistic};
use std::time::Instant;

/// 非安靜模式的進度輸出：第一個事件描述斷點續傳的起始點，之後每批一行
struct ProgressPrinter {
    start: Instant,
    initial: Option<u64>,
}

impl ProgressPrinter {
    fn new() -> Self {
        Self {
            start: Instant::now(),
            initial: None,
        }
    }

    fn print(&mut self, progress: Progress) {
        let Some(initial) = self.initial else {
            self.initial = Some(progress.completed);
            if progress.completed >= progress.total {
                println!("SUCCESS: calculation for this model already completed, skipping");
            } else {
                if progress.completed > 0 {
                    println!(
                        "Detected existing file with {} completed records",
                        format_number_with_commas(progress.completed as usize)
                    );
                }
                println!(
                    "Remaining {} calculations to complete",
                    format_number_with_commas((progress.total - progress.completed) as usize)
                );
                self.start = Instant::now();
            }
            return;
        };

        let batch_done = (progress.completed - initial) as usize;
        let batch_total = (progress.total - initial) as usize;
        println!(
            "Simulation progress: {}/{} ({:.2}%) - {}",
            format_number_with_commas(progress.completed as usize),
            format_number_with_commas(progress.total as usize),
            progress.completed as f64 / progress.total as f64 * 100.0,
            format_remaining_time(self.start.elapsed(), batch_done, batch_total)
        );
    }
}

/// 建立模擬配置；參數不合法時輸出錯誤並結束程式
fn new_simulation(model: JohansenModel, dim: usize, args: &CliArgs) -> EigenvalueSimulation {
    match EigenvalueSimulation::new(model, dim, args.steps, args.num_runs) {
        Ok(simulation) => simulation,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

/// 執行模擬。遇到可重建的不相容檔案（格式或參數不符）時刪檔重建一次；
/// 其餘錯誤直接失敗。資料皆可由 seed 重算，重建只損失計算時間。
fn run_simulation_cli(simulation: &EigenvalueSimulation, quiet: bool) {
    let mut recreated = false;
    loop {
        let result = if quiet {
            simulation.run()
        } else {
            let mut printer = ProgressPrinter::new();
            simulation.run_with_progress(|progress| printer.print(progress))
        };
        match result {
            Ok(()) => break,
            Err(e) if e.is_recreatable() && !recreated => {
                conditional_println!(quiet, "WARNING: Existing file is incompatible ({e})");
                conditional_println!(
                    quiet,
                    "  The file will be removed and recreated with correct parameters."
                );
                simulation.delete_data();
                recreated = true;
            }
            Err(e) => {
                eprintln!("Simulation failed: {e}");
                std::process::exit(1);
            }
        }
    }
    conditional_println!(
        quiet,
        "Saved to {}\n===============================\n",
        simulation.data_path().display()
    );
}

/// 讀取一次資料，計算並顯示 Trace 與 MaxEig 的百分位數統計
fn display_simulation_statistics(simulation: &EigenvalueSimulation, percentiles: &[f64]) {
    let Ok(data) = simulation.load() else {
        return; // 讀取失敗時跳過統計顯示
    };
    for (label, statistic) in [
        ("Trace", Statistic::Trace),
        ("MaxEig", Statistic::MaxEigenvalue),
    ] {
        let values = data.percentiles(statistic, percentiles);
        display_percentiles_table(
            &simulation.model().to_string(),
            label,
            percentiles,
            &values,
            data.len(),
        );
        println!();
    }
}

fn main() {
    // 解析命令行參數
    let args = match CliArgs::parse() {
        Some(args) => args,
        None => return, // 顯示幫助信息或解析錯誤
    };

    // 配置 Rayon 線程池
    args.configure_rayon();

    // 將 models_vec 提升到 main 函式較外層作用域
    let models_vec = args
        .models
        .clone()
        .unwrap_or_else(|| JohansenModel::ALL.to_vec());

    conditional_println!(args.quiet, "=== Large-scale Simulation Demo ===");
    conditional_println!(
        args.quiet,
        "Starting simulation (supports resuming from checkpoint)..."
    );
    conditional_println!(args.quiet, "Configuration:");
    conditional_println!(
        args.quiet,
        "  Steps: {}",
        format_number_with_commas(args.steps)
    );
    conditional_println!(
        args.quiet,
        "  Runs: {}",
        format_number_with_commas(args.num_runs)
    );
    conditional_println!(
        args.quiet,
        "  Dimension range: {} - {}",
        args.dim_start,
        args.dim_end
    );
    conditional_println!(args.quiet, "  Threads: {}", rayon::current_num_threads());
    conditional_println_empty!(args.quiet);

    for dim in args.dim_start..=args.dim_end {
        let start_time = Instant::now();
        conditional_println!(
            args.quiet,
            "Simulation config: {} dimensions, {} steps, {} runs",
            dim,
            format_number_with_commas(args.steps),
            format_number_with_commas(args.num_runs)
        );

        // 對每個模型運行模擬
        for &model in &models_vec {
            let simulation = new_simulation(model, dim, &args);
            conditional_println!(
                args.quiet,
                "Using model: {model} (supports resuming from checkpoint)"
            );
            run_simulation_cli(&simulation, args.quiet);

            if !args.quiet {
                // 收集並顯示統計數據（在每個模型運行完後立即分析）
                let percentiles = [0.5, 0.75, 0.8, 0.85, 0.9, 0.95, 0.975, 0.99];
                display_simulation_statistics(&simulation, &percentiles);
            }
        }

        let elapsed_time = start_time.elapsed();
        conditional_println!(
            args.quiet,
            "Simulation completed! Duration: {}",
            format_duration(elapsed_time)
        );
    }

    conditional_println!(args.quiet, "\n=== Result Reading Demo ===");

    // 讀取特定模型的數據
    conditional_println!(args.quiet, "Starting to read model data...");
    let model = models_vec[0]; // 使用第一個模型作為範例
    let simulation = new_simulation(model, args.dim_start, &args);
    conditional_println!(
        args.quiet,
        "Simulation config: {} dimensions, {} steps, {} runs",
        simulation.dim(),
        format_number_with_commas(simulation.steps()),
        format_number_with_commas(simulation.num_runs())
    );
    match simulation.load() {
        Ok(data) => {
            conditional_println!(
                args.quiet,
                "Successfully read data for model {}: {} records",
                model,
                format_number_with_commas(data.len())
            );

            // 顯示前5筆數據作為範例
            conditional_println!(args.quiet, "First 5 data records as examples:");

            // 計算前5筆數據的最大寬度以對齊顯示
            let preview_data = &data.records()[..data.len().min(5)];
            let max_seed_width = preview_data
                .iter()
                .map(|record| record.seed.to_string().len())
                .max()
                .unwrap_or(1);
            let max_sum_width = preview_data
                .iter()
                .map(|record| format!("{:.6}", record.eigenvalues.iter().sum::<f64>()).len())
                .max()
                .unwrap_or(8);
            let max_eigenvalue_width = preview_data
                .iter()
                .flat_map(|record| record.eigenvalues.iter())
                .map(|x| format!("{x:.6}").len())
                .max()
                .unwrap_or(8);

            // 預先計算所有 eigenvalue_str 以找出最大寬度
            let eigenvalue_strs: Vec<String> = preview_data
                .iter()
                .map(|record| {
                    record
                        .eigenvalues
                        .iter()
                        .map(|x| format!("{x:max_eigenvalue_width$.6}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .collect();
            let max_eigenvalue_str_width =
                eigenvalue_strs.iter().map(|s| s.len()).max().unwrap_or(20);

            for (i, record) in preview_data.iter().enumerate() {
                let eigenvalue_str = &eigenvalue_strs[i];
                conditional_println!(
                    args.quiet,
                    "  Record {:2}: seed={:width1$}, eigenvalue sum={:width2$.6}, eigenvalues=[{:width3$}]",
                    i + 1,
                    record.seed,
                    record.eigenvalues.iter().sum::<f64>(),
                    eigenvalue_str,
                    width1 = max_seed_width,
                    width2 = max_sum_width,
                    width3 = max_eigenvalue_str_width
                );
            }
        }
        Err(e) => conditional_println!(args.quiet, "Failed to read data: {}", e),
    }

    // 展示所有模型的狀態
    conditional_println!(args.quiet, "\n=== All Models Status ===");
    for &model in &models_vec {
        let sim = new_simulation(model, args.dim_start, &args);
        match sim.load() {
            Ok(data) => conditional_println!(
                args.quiet,
                "  {}: {} data records",
                model,
                format_number_with_commas(data.len())
            ),
            Err(_) => conditional_println!(args.quiet, "  {}: No data or read failed", model),
        }
    }

    conditional_println!(args.quiet, "\nDemo completed!");
}
