use super::*;
use crate::data_storage::file_format::{crc_path, hash_path};

#[test]
fn test_basic_simulation_api() {
    let model = JohansenModel::NoInterceptNoTrend;
    let simulation = test_sim("basic_api", model, 2, 101, 5);
    let data_path = simulation.data_path();

    // 清理現有檔案
    simulation.delete_data();

    // 測試檔案路徑生成
    let path_str = data_path.to_string_lossy();
    assert!(path_str.contains("model0"));
    assert!(path_str.contains("dim2"));
    assert!(path_str.contains("steps101"));
    assert!(path_str.ends_with(".dat"));

    // 運行模擬（帶進度回報）
    for &model in &JohansenModel::ALL {
        let model_simulation = test_sim("basic_api", model, 2, 101, 5);
        let mut events: Vec<crate::data_storage::Progress> = Vec::new();
        model_simulation
            .run_with_progress(|progress| events.push(progress))
            .unwrap();

        // 首個事件為續傳起始點（新檔案 → 0），最後一個事件應達到 total
        assert!(!events.is_empty());
        assert_eq!(events[0].completed, 0);
        assert!(events.iter().all(|p| p.total == 5));
        assert_eq!(events.last().unwrap().completed, 5);
    }

    // 檢查檔案是否存在
    assert!(data_path.exists());

    // 完成後：CRC sidecar 應已刪除，SHA-256 雜湊檔應存在
    assert!(!crc_path(&data_path).exists());
    assert!(hash_path(&data_path).exists());

    // 載入數據（依 seed 順序，第 i 筆即 seed = i+1）
    let data = simulation.load().unwrap();
    assert!(data.is_complete());
    assert_eq!(data.len(), 5);
    for record in data.records() {
        assert_eq!(record.eigenvalues.len(), 2); // 2x2 矩陣有 2 個特徵值
    }

    // load_partial 帶明確 seed 且與 load 一致
    let partial = simulation.load_partial().unwrap();
    assert!(partial.is_complete());
    assert_eq!(partial.len(), 5);
    for (i, record) in partial.records().iter().enumerate() {
        assert_eq!(record.seed, i as u32 + 1);
        assert_eq!(record, &data.records()[i]);
    }

    // 完成的檔案應能通過 SHA-256 驗證
    simulation.verify().unwrap();

    // 已完成的檔案重跑：首個進度事件即 completed == total
    let mut events: Vec<crate::data_storage::Progress> = Vec::new();
    simulation
        .run_with_progress(|progress| events.push(progress))
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].completed, events[0].total);

    // 清理測試檔案
    for &model in &JohansenModel::ALL {
        test_sim("basic_api", model, 2, 101, 5).delete_data();
    }
}
