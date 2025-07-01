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

fn main() {
    // 解析命令行參數
    let args = match CliArgs::parse() {
        Some(args) => args,
        None => return, // 顯示幫助信息或解析錯誤
    };

    // 配置 Rayon 線程池
    args.configure_rayon();

    println!("=== Large-scale Simulation Demo ===");
    println!("Starting simulation (supports resuming from checkpoint)...");
    println!("Configuration:");
    println!("  Steps: {}", format_number_with_commas(args.steps));
    println!("  Runs: {}", format_number_with_commas(args.num_runs));
    println!("  Dimension range: {} - {}", args.dim_start, args.dim_end);
    println!("  Threads: {}", rayon::current_num_threads());
    println!();

    for dim in args.dim_start..=args.dim_end {
        let start_time = Instant::now();
        println!(
            "Simulation config: {} dimensions, {} steps, {} runs",
            dim,
            format_number_with_commas(args.steps),
            format_number_with_commas(args.num_runs)
        );
        let models_vec = args
            .models
            .clone()
            .unwrap_or_else(|| JohansenModel::all_models().to_vec());
        if args.quiet {
            EigenvalueSimulation::new(dim, args.steps, args.num_runs)
                .run_simulation_quiet(&models_vec);
        } else {
            EigenvalueSimulation::new(dim, args.steps, args.num_runs).run_simulation(&models_vec);
        }
        let elapsed_time = start_time.elapsed();
        println!(
            "Simulation completed! Duration: {}",
            format_duration(elapsed_time)
        );
    }

    println!("\n=== Result Reading Demo ===");

    // 讀取特定模型的數據
    println!("Starting to read model data...");
    let simulation = EigenvalueSimulation::new(args.dim_start, args.steps, args.num_runs);
    println!(
        "Simulation config: {} dimensions, {} steps, {} runs",
        simulation.dim,
        format_number_with_commas(simulation.steps),
        format_number_with_commas(simulation.num_runs)
    );
    let model = JohansenModel::NoInterceptNoTrend; // 使用無截距無趨勢模型作為範例
    match simulation.read_data(model) {
        Ok(data) => {
            println!(
                "Successfully read data for model {}: {} records",
                model,
                format_number_with_commas(data.len())
            );

            // 顯示前5筆數據作為範例
            println!("First 5 data records as examples:");

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
                .map(|x| format!("{:.6}", x).len())
                .max()
                .unwrap_or(8);

            // 預先計算所有 eigenvalue_str 以找出最大寬度
            let eigenvalue_strs: Vec<String> = preview_data
                .iter()
                .map(|(_, eigenvalues)| {
                    eigenvalues
                        .iter()
                        .map(|x| format!("{:width$.6}", x, width = max_eigenvalue_width))
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .collect();
            let max_eigenvalue_str_width =
                eigenvalue_strs.iter().map(|s| s.len()).max().unwrap_or(20);

            for (i, (seed, eigenvalues)) in preview_data.iter().enumerate() {
                let eigenvalue_str = &eigenvalue_strs[i];
                println!(
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
        Err(e) => println!("Failed to read data: {}", e),
    }

    // 展示所有模型的狀態
    println!("\n=== All Models Status ===");
    let all_data = simulation.read_all_data();
    for (model, result) in all_data {
        match result {
            Ok(data) => println!(
                "  {}: {} data records",
                model,
                format_number_with_commas(data.len())
            ),
            Err(_) => println!("  {}: No data or read failed", model),
        }
    }

    println!("\nDemo completed!");
}
