use super::*;
use crate::data_storage::file_format::{crc_path, hash_path};

#[test]
fn test_basic_simulation_api() {
    // 使用唯一的參數組合避免與其他測試檔案衝突
    let model = JohansenModel::NoInterceptNoTrend;
    let simulation = EigenvalueSimulation::new(model, 2, 101, 5);
    let data_path = simulation.get_file_path(model);

    // 清理現有檔案
    simulation.remove_data_files();

    // 測試檔案路徑生成
    let path_str = data_path.to_string_lossy();
    assert!(path_str.contains("model0"));
    assert!(path_str.contains("dim2"));
    assert!(path_str.contains("steps101"));
    assert!(path_str.ends_with(".dat"));

    // 運行模擬
    for &model in &JohansenModel::all_models() {
        let model_simulation = EigenvalueSimulation::new(model, 2, 101, 5);
        model_simulation.run_simulation_quiet();
    }

    // 檢查檔案是否存在
    assert!(data_path.exists());

    // 完成後：CRC sidecar 應已刪除，SHA-256 雜湊檔應存在
    assert!(!crc_path(&data_path).exists());
    assert!(hash_path(&data_path).exists());

    // 讀取數據（依 seed 順序，第 i 筆即 seed = i+1）
    let data = simulation.read_data().unwrap();
    assert_eq!(data.len(), 5);
    for eigenvalues in &data {
        assert_eq!(eigenvalues.len(), 2); // 2x2 矩陣有 2 個特徵值
    }

    // read_all_data 帶明確 seed 且與 read_data 一致
    let all_data = simulation.read_all_data().unwrap();
    assert_eq!(all_data.len(), 5);
    for (i, (seed, eigenvalues)) in all_data.iter().enumerate() {
        assert_eq!(*seed, i as u32 + 1);
        assert_eq!(eigenvalues, &data[i]);
    }

    // 完成的檔案應能通過 SHA-256 驗證
    simulation.verify_file().unwrap();

    // 清理測試檔案
    for &model in &JohansenModel::all_models() {
        EigenvalueSimulation::new(model, 2, 101, 5).remove_data_files();
    }
}
