mod cli;
mod data_storage;
mod display_utils;
mod johansen_models;
mod johansen_statistics;
mod matrix_utils;
mod rng_matrix;

use cli::CliArgs;
use data_storage::EigenvalueSimulation;
use display_utils::{format_duration, format_number_with_commas};
use johansen_models::JohansenModel;
use std::time::Instant;

/// 輸出百分位數統計資訊
fn print_percentile_statistics(sorted_eigenvalues: &[f64], percentiles: &[f64]) {
    println!(
        "Total calculated {} eigenvalue sums",
        format_number_with_commas(sorted_eigenvalues.len())
    );

    // 輸出各個百分位數
    for &percentile in percentiles {
        let index = ((sorted_eigenvalues.len() as f64) * percentile) as usize;
        let value = sorted_eigenvalues[index.min(sorted_eigenvalues.len() - 1)];
        println!("{:.0}th percentile value: {:.6}", percentile * 100.0, value);
    }
}

/// 從已有的數據文件中收集和分析統計信息
fn analyze_simulation_statistics(simulation: &EigenvalueSimulation, model: JohansenModel) {
    match simulation.read_data(model) {
        Ok(data) => {
            if !data.is_empty() {
                let eigenvalue_sums: Vec<f64> = data
                    .iter()
                    .map(|(_, eigenvalues)| eigenvalues.iter().sum())
                    .collect();

                let mut sorted_eigenvalue_sums = eigenvalue_sums;
                sorted_eigenvalue_sums.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let percentiles = vec![0.5, 0.75, 0.8, 0.85, 0.9, 0.95, 0.975, 0.99];
                println!("Statistics for model {model}:");
                print_percentile_statistics(&sorted_eigenvalue_sums, &percentiles);
            }
        }
        Err(_) => {
            // 如果讀取失敗，忽略這個模型
        }
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

        // 輸出配置信息（僅在非安靜模式下）
        if !args.quiet {
            println!(
                "Starting large-scale eigenvalue simulation (supports resuming from checkpoint)..."
            );
            println!(
                "Dimensions: {}, Steps: {}, Runs: {}",
                format_number_with_commas(dim),
                format_number_with_commas(args.steps),
                format_number_with_commas(args.num_runs)
            );
        }

        let models_vec = args
            .models
            .clone()
            .unwrap_or_else(|| JohansenModel::all_models().to_vec());

        let simulation = EigenvalueSimulation::new(dim, args.steps, args.num_runs);

        // 對每個模型運行模擬
        for &model in &models_vec {
            if args.quiet {
                simulation.run_simulation_quiet(model);
            } else {
                simulation.run_simulation(model);
                // 收集並顯示統計數據（在每個模型運行完後立即分析）
                analyze_simulation_statistics(&simulation, model);
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
    let simulation = EigenvalueSimulation::new(args.dim_start, args.steps, args.num_runs);
    conditional_println!(
        args.quiet,
        "Simulation config: {} dimensions, {} steps, {} runs",
        simulation.dim,
        format_number_with_commas(simulation.steps),
        format_number_with_commas(simulation.num_runs)
    );
    let model = JohansenModel::NoInterceptNoTrend; // 使用無截距無趨勢模型作為範例
    match simulation.read_data(model) {
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
            let preview_data: Vec<_> = data.iter().take(5).collect();
            let max_seed_width = preview_data
                .iter()
                .map(|(seed, _)| seed.to_string().len())
                .max()
                .unwrap_or(3);
            let max_sum_width = preview_data
                .iter()
                .map(|(_, eigenvalues)| format!("{:.6}", eigenvalues.iter().sum::<f64>()).len())
                .max()
                .unwrap_or(8);
            let max_eigenvalue_width = preview_data
                .iter()
                .flat_map(|(_, eigenvalues)| eigenvalues.iter())
                .map(|x| format!("{x:.6}").len())
                .max()
                .unwrap_or(8);

            // 預先計算所有 eigenvalue_str 以找出最大寬度
            let eigenvalue_strs: Vec<String> = preview_data
                .iter()
                .map(|(_, eigenvalues)| {
                    eigenvalues
                        .iter()
                        .map(|x| format!("{x:max_eigenvalue_width$.6}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .collect();
            let max_eigenvalue_str_width =
                eigenvalue_strs.iter().map(|s| s.len()).max().unwrap_or(20);

            for (i, (seed, eigenvalues)) in preview_data.iter().enumerate() {
                let eigenvalue_str = &eigenvalue_strs[i];
                conditional_println!(
                    args.quiet,
                    "  Record {:2}: seed={:width1$}, eigenvalue sum={:width2$.6}, eigenvalues=[{:width3$}]",
                    i + 1,
                    seed,
                    eigenvalues.iter().sum::<f64>(),
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
    let all_data = simulation.read_all_data();
    for (model, result) in all_data {
        match result {
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
