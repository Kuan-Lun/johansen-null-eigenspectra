use super::*;

#[test]
fn test_read_all_data() {
    // 使用唯一的參數組合避免與其他測試檔案衝突
    // 清理所有現有檔案
    for test_model in JohansenModel::all_models() {
        EigenvalueSimulation::new(test_model, 3, 52, 3).remove_data_files();
    }

    // 確保有一些數據
    for &test_model in &JohansenModel::all_models() {
        let test_simulation = EigenvalueSimulation::new(test_model, 3, 52, 3);
        test_simulation.run_simulation_quiet();
    }

    // 逐個模型讀取資料
    for &test_model in &JohansenModel::all_models() {
        let sim = EigenvalueSimulation::new(test_model, 3, 52, 3);

        // read_data：依 seed 順序的完整資料
        let data = sim.read_data().unwrap();
        assert_eq!(data.len(), 3);

        // read_all_data：帶明確 seed，內容應與 read_data 一致
        let all_data = sim.read_all_data().unwrap();
        assert_eq!(all_data.len(), 3);
        for (i, (seed, eigenvalues)) in all_data.iter().enumerate() {
            assert_eq!(*seed, i as u32 + 1);
            assert_eq!(eigenvalues, &data[i]);
        }

        // 檢查檔案名稱是否正確
        let expected_filename = sim.get_filename(test_model);
        assert!(std::path::Path::new(&expected_filename).exists());
    }

    // 清理測試檔案
    for test_model in JohansenModel::all_models() {
        EigenvalueSimulation::new(test_model, 3, 52, 3).remove_data_files();
    }
}
