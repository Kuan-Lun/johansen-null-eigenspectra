use super::*;

#[test]
fn test_load_partial() {
    // 清理所有現有檔案
    for test_model in JohansenModel::ALL {
        test_sim("load_partial", test_model, 3, 52, 3).delete_data();
    }

    // 確保有一些數據
    for &test_model in &JohansenModel::ALL {
        test_sim("load_partial", test_model, 3, 52, 3)
            .run()
            .unwrap();
    }

    // 逐個模型讀取資料
    for &test_model in &JohansenModel::ALL {
        let sim = test_sim("load_partial", test_model, 3, 52, 3);

        // load：依 seed 順序的完整資料
        let data = sim.load().unwrap();
        assert!(data.is_complete());
        assert_eq!(data.len(), 3);

        // load_partial：帶明確 seed，內容應與 load 一致
        let partial = sim.load_partial().unwrap();
        assert!(partial.is_complete());
        assert_eq!(partial.len(), 3);
        for (i, record) in partial.records().iter().enumerate() {
            assert_eq!(record.seed, i as u32 + 1);
            assert_eq!(record, &data.records()[i]);
        }

        // 檢查檔案路徑是否正確
        assert!(sim.data_path().exists());
    }

    // 清理測試檔案
    for test_model in JohansenModel::ALL {
        test_sim("load_partial", test_model, 3, 52, 3).delete_data();
    }
}
