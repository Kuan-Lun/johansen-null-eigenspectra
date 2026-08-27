use super::*;

#[test]
fn test_multiple_models() {
    // 清理所有現有檔案
    for test_model in JohansenModel::ALL {
        test_sim("multiple_models", test_model, 2, 54, 3).delete_data();
    }

    // 運行所有模型的計算
    for &test_model in &JohansenModel::ALL {
        test_sim("multiple_models", test_model, 2, 54, 3)
            .run()
            .unwrap();
    }

    // 檢查每個模型都有對應的檔案
    for test_model in JohansenModel::ALL {
        let test_simulation = test_sim("multiple_models", test_model, 2, 54, 3);
        assert!(test_simulation.data_path().exists());

        let data = test_simulation.load().unwrap();
        assert_eq!(data.len(), 3);

        // 檢查數據格式：特徵值數量由模型決定（dim 或 dim+1）
        let expected_count = test_model.eigenvalues_per_run(2);
        for record in data.records() {
            assert_eq!(record.eigenvalues.len(), expected_count);
            assert!(record.eigenvalues.iter().all(|&x| x.is_finite()));
        }
    }

    // 清理測試檔案
    for test_model in JohansenModel::ALL {
        test_sim("multiple_models", test_model, 2, 54, 3).delete_data();
    }
}
