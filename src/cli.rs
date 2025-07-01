//! 命令行參數解析模組
//!
//! 提供命令行參數的解析、驗證和幫助信息顯示功能。

use crate::johansen_models::JohansenModel;
use std::io::{self, Write};

/// 命令行參數配置
#[derive(Debug, Clone)]
pub struct CliArgs {
    pub num_threads: Option<usize>,
    pub steps: usize,
    pub num_runs: usize,
    pub dim_start: usize,
    pub dim_end: usize,
    pub models: Option<Vec<JohansenModel>>,
    pub quiet: bool,
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            num_threads: None,
            steps: 1e4 as usize,
            num_runs: 1e7 as usize,
            dim_start: 1,
            dim_end: 12,
            models: None,
            quiet: false, // 預設為 false
        }
    }
}

impl CliArgs {
    /// 從命令行參數解析配置
    pub fn parse() -> Option<Self> {
        let args: Vec<String> = std::env::args().collect();
        let mut config = Self::default();

        // 顯示幫助信息
        if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
            Self::print_help(&args[0]);
            return None;
        }

        // 參數解析
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--threads" => {
                    if let Some(value) = Self::parse_next_arg(&args, i, "--threads")? {
                        config.num_threads = Some(value);
                    } else {
                        return None;
                    }
                    i += 2;
                }
                "--steps" => {
                    if let Some(value) = Self::parse_next_arg(&args, i, "--steps")? {
                        config.steps = value;
                    } else {
                        return None;
                    }
                    i += 2;
                }
                "--runs" => {
                    if let Some(value) = Self::parse_next_arg(&args, i, "--runs")? {
                        config.num_runs = value;
                    } else {
                        return None;
                    }
                    i += 2;
                }
                "--dim-start" => {
                    if let Some(value) = Self::parse_next_arg(&args, i, "--dim-start")? {
                        config.dim_start = value;
                    } else {
                        return None;
                    }
                    i += 2;
                }
                "--dim-end" => {
                    if let Some(value) = Self::parse_next_arg(&args, i, "--dim-end")? {
                        config.dim_end = value;
                    } else {
                        return None;
                    }
                    i += 2;
                }
                "--dim" => {
                    if let Some(value) = Self::parse_next_arg(&args, i, "--dim")? {
                        config.dim_start = value;
                        config.dim_end = value;
                    } else {
                        return None;
                    }
                    i += 2;
                }
                "--model" => {
                    if let Some(value) = Self::parse_next_string(&args, i, "--model")? {
                        match Self::parse_models(&value) {
                            Ok(models) => config.models = Some(models),
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                return None;
                            }
                        }
                    } else {
                        return None;
                    }
                    i += 2;
                }
                "--quiet" => {
                    config.quiet = true;
                    i += 1;
                }
                _ => {
                    eprintln!("Error: unknown argument '{}'", args[i]);
                    eprintln!("Use --help to see available options");
                    return None;
                }
            }
        }

        // 參數驗證
        if !config.validate() {
            return None;
        }

        Some(config)
    }

    /// 解析下一個參數值（支援逗號分隔的數字）
    fn parse_next_arg(args: &[String], index: usize, param_name: &str) -> Option<Option<usize>> {
        if index + 1 >= args.len() {
            eprintln!("Error: {} parameter requires a value", param_name);
            return None;
        }

        let input = &args[index + 1];
        // 移除逗號後再解析
        let cleaned_input = input.replace(',', "");

        match cleaned_input.parse::<usize>() {
            Ok(value) => Some(Some(value)),
            Err(_) => {
                eprintln!(
                    "Error: {} parameter must be a positive integer (commas allowed, e.g., 1,000,000)",
                    param_name
                );
                None
            }
        }
    }
    /// 解析下一個參數字串
    fn parse_next_string(
        args: &[String],
        index: usize,
        param_name: &str,
    ) -> Option<Option<String>> {
        if index + 1 >= args.len() {
            eprintln!("Error: {} parameter requires a value", param_name);
            return None;
        }
        Some(Some(args[index + 1].clone()))
    }

    /// 從逗號分隔的字串解析模型列表
    fn parse_models(s: &str) -> Result<Vec<JohansenModel>, String> {
        let mut models = Vec::new();
        for part in s.split(',') {
            let part_trim = part.trim();
            if part_trim.is_empty() {
                continue;
            }
            match part_trim
                .parse::<u8>()
                .ok()
                .and_then(JohansenModel::from_number)
            {
                Some(m) => models.push(m),
                None => return Err(format!("無效的模型代號: {}", part_trim)),
            }
        }
        if models.is_empty() {
            Err("模型列表不可為空".to_string())
        } else {
            Ok(models)
        }
    }

    /// 驗證參數的有效性
    fn validate(&self) -> bool {
        // 檢查維度範圍
        if self.dim_start > self.dim_end {
            eprintln!(
                "Error: dimension start ({}) cannot be greater than end ({})",
                self.dim_start, self.dim_end
            );
            return false;
        }

        // 檢查正整數參數
        if self.steps == 0 || self.num_runs == 0 || self.dim_start == 0 {
            eprintln!("Error: steps, runs, and dimension parameters must be greater than 0");
            return false;
        }

        // 檢查線程數量
        if let Some(threads) = self.num_threads {
            if !self.validate_thread_count(threads) {
                return false;
            }
        }

        true
    }

    /// 驗證線程數量
    fn validate_thread_count(&self, threads: usize) -> bool {
        let available_threads = num_cpus::get();

        if threads > available_threads {
            eprintln!(
                "Warning: specified thread count ({}) exceeds available logical cores ({})",
                threads, available_threads
            );
            eprintln!(
                "This may cause performance degradation due to thread contention. Recommend using no more than {} threads.",
                available_threads
            );

            // 如果超過太多，直接拒絕
            if threads > available_threads * 2 {
                eprintln!("Error: excessive thread count rejected to avoid system overload.");
                return false;
            }

            // 詢問是否繼續
            print!("Do you want to continue? (y/N): ");
            io::stdout().flush().expect("Failed to flush stdout");

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");
            let input = input.trim().to_lowercase();

            if input != "y" && input != "yes" {
                println!("Execution cancelled.");
                return false;
            }
        }

        true
    }

    /// 顯示幫助信息
    fn print_help(program_name: &str) {
        println!("Usage: {} [OPTIONS]", program_name);
        println!();
        println!("Options:");
        println!(
            "  --threads <int>      number of threads for parallel computation (default: {} logical cores)",
            num_cpus::get()
        );
        println!("  --steps <int>        number of simulation steps (default: 10,000)");
        println!("  --runs <int>         number of runs per model (default: 10,000,000)");
        println!("  --dim-start <int>    starting matrix dimension (default: 1)");
        println!("  --dim-end <int>      ending matrix dimension (default: 12)");
        println!(
            "  --dim <int>          run a single dimension (sets start and end to the same value)"
        );
        println!(
            "  --model <list>       comma separated list of model numbers to compute (default: 0,1,2,3,4)"
        );
        println!("  --quiet              suppress progress output");
        println!("  -h, --help           show this help message");
        println!();
        println!("Examples:");
        println!(
            "  {} --threads 4 --steps 5,000 --runs 1,000,000",
            program_name
        );
        println!("  {} --dim 5 --threads 8", program_name);
        println!(
            "  {} --dim-start 2 --dim-end 8 --runs 500,000",
            program_name
        );
        println!("  {} --model 0,2 --runs 100,000", program_name);
    }

    /// 配置 Rayon 線程池
    pub fn configure_rayon(&self) {
        if let Some(threads) = self.num_threads {
            println!("Using {} threads for parallel computation", threads);
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build_global()
                .expect("Failed to build thread pool");
        } else {
            println!(
                "Using default thread count: {}",
                rayon::current_num_threads()
            );
        }
    }
}
