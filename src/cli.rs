//! 命令行參數解析模組
//!
//! 提供命令行參數的解析、驗證和幫助信息顯示功能。

use std::io::{self, Write};
use crate::johansen_models::JohansenModel;

/// 命令行參數配置
#[derive(Debug, Clone)]
pub struct CliArgs {
    pub num_threads: Option<usize>,
    pub steps: usize,
    pub num_runs: usize,
    pub dim_start: usize,
    pub dim_end: usize,
    pub models: Option<Vec<JohansenModel>>, // 新增模型參數
    pub quiet: bool, // 新增 quiet 參數
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
                                eprintln!("錯誤: {}", e);
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
                    eprintln!("錯誤: 未知參數 '{}'", args[i]);
                    eprintln!("使用 --help 查看可用選項");
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

    /// 解析下一個參數值
    fn parse_next_arg(args: &[String], index: usize, param_name: &str) -> Option<Option<usize>> {
        if index + 1 >= args.len() {
            eprintln!("錯誤: {} 參數缺少數值", param_name);
            return None;
        }

        match args[index + 1].parse::<usize>() {
            Ok(value) => Some(Some(value)),
            Err(_) => {
                eprintln!("錯誤: {} 參數必須是正整數", param_name);
                None
            }
        }
    }

    /// 解析下一個參數字串
    fn parse_next_string(args: &[String], index: usize, param_name: &str) -> Option<Option<String>> {
        if index + 1 >= args.len() {
            eprintln!("錯誤: {} 參數缺少數值", param_name);
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
                "錯誤: 維度範圍開始 ({}) 不能大於結束 ({})",
                self.dim_start, self.dim_end
            );
            return false;
        }

        // 檢查正整數參數
        if self.steps == 0 || self.num_runs == 0 || self.dim_start == 0 {
            eprintln!("錯誤: steps、runs 和維度參數必須大於 0");
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
                "警告: 指定的線程數 ({}) 超過系統可用的邏輯線程數 ({})",
                threads, available_threads
            );
            eprintln!(
                "這可能會導致性能下降，因為會有線程競爭。建議使用不超過 {} 個線程。",
                available_threads
            );

            // 如果超過太多，直接拒絕
            if threads > available_threads * 2 {
                eprintln!("錯誤: 線程數過多，已拒絕執行以避免系統過載。");
                return false;
            }

            // 詢問是否繼續
            print!("是否要繼續執行？(y/N): ");
            io::stdout().flush().expect("Failed to flush stdout");

            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("讀取輸入失敗");
            let input = input.trim().to_lowercase();

            if input != "y" && input != "yes" {
                println!("已取消執行。");
                return false;
            }
        }

        true
    }

    /// 顯示幫助信息
    fn print_help(program_name: &str) {
        println!("用法: {} [選項]", program_name);
        println!();
        println!("選項:");
        println!(
            "  --threads <int>      設定並行計算使用的線程數量 (預設: {} 個線程)",
            num_cpus::get()
        );
        println!("  --steps <int>        設定模擬步驟數 (預設: 10,000)");
        println!("  --runs <int>         設定運行次數 (預設: 10,000,000)");
        println!("  --dim-start <int>    設定維度範圍開始 (預設: 1)");
        println!("  --dim-end <int>      設定維度範圍結束 (預設: 12)");
        println!("  --dim <int>          設定單一維度 (等同於設定相同的 start 和 end)");
        println!("  --model <list>       指定要計算的模型代號，使用逗號分隔，如 0,2,4");
        println!("  -h, --help           顯示此幫助信息");
        println!();
        println!("範例:");
        println!("  {} --threads 4 --steps 5000 --runs 1000000", program_name);
        println!("  {} --dim 5 --threads 8", program_name);
        println!("  {} --dim-start 2 --dim-end 8 --runs 500000", program_name);
        println!("  {} --model 0,2 --runs 100000", program_name);
    }

    /// 配置 Rayon 線程池
    pub fn configure_rayon(&self) {
        if let Some(threads) = self.num_threads {
            println!("設定使用 {} 個線程進行並行計算", threads);
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build_global()
                .expect("Failed to build thread pool");
        } else {
            println!("使用預設線程數: {} 個", rayon::current_num_threads());
        }
    }
}
