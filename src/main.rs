mod data_storage;
mod johansen_models;
mod johansen_statistics;
mod matrix_utils;
mod rng_matrix;

use data_storage::EigenvalueSimulation;
use johansen_models::JohansenModel;
use johansen_statistics::calculate_eigenvalues;

fn main() {
    println!("=== Johansen Null Eigenspectra 演示程式 ===");

    let dim = 2;
    let steps = 1000;
    let seed = 42;
    let model = JohansenModel::NoInterceptNoTrend;

    println!("單次計算演示:");
    println!(
        "  維度: {}, 步驟: {}, 種子: {}, 模型: {}",
        dim, steps, seed, model
    );

    let eigenvalues = calculate_eigenvalues(dim, steps, seed, model);
    println!("  特徵值: {:?}", eigenvalues);
    println!("  特徵值總和: {:.6}", eigenvalues.iter().sum::<f64>());

    println!("\n=== 大規模模擬演示 ===");

    // 使用較小的規模進行演示
    let simulation = EigenvalueSimulation::new(dim, steps, 100); // 100次運行
    println!("模擬設定: {} 維度, {} 步驟, {} 次運行", dim, steps, 100);

    // 運行支援斷點續傳的大規模計算
    println!("開始運行模擬（支援斷點續傳）...");
    simulation.run_simulation();

    println!("\n=== 結果讀取演示 ===");

    // 讀取特定模型的數據
    match simulation.read_data(model) {
        Ok(data) => {
            println!("成功讀取模型 {} 的數據: {} 筆記錄", model, data.len());

            // 顯示前5筆數據作為範例
            println!("前5筆數據範例:");
            for (i, (seed, eigenvalues)) in data.iter().take(5).enumerate() {
                println!(
                    "  第{:2}筆: seed={:3}, 特徵值總和={:.6}",
                    i + 1,
                    seed,
                    eigenvalues.iter().sum::<f64>()
                );
            }

            // 如果有 seed=42 的數據，特別顯示
            if let Some((_, eigenvalues)) = data.iter().find(|(s, _)| *s == 42) {
                println!("  seed=42 的詳細結果: {:?}", eigenvalues);
            }
        }
        Err(e) => println!("讀取失敗: {}", e),
    }

    // 展示所有模型的狀態
    println!("\n=== 所有模型狀態 ===");
    let all_data = simulation.read_all_data();
    for (model, result) in all_data {
        match result {
            Ok(data) => println!("  {} : {} 筆數據", model, data.len()),
            Err(_) => println!("  {} : 無數據或讀取失敗", model),
        }
    }

    println!("\n=== 檔案資訊 ===");
    println!("檔案命名範例: {}", simulation.get_filename(model));

    println!("\n演示完成！");
    println!("提示: 如需測試斷點續傳功能，請運行 'cargo test' 執行測試套件");
}
