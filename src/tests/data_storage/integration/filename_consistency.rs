use super::*;

#[test]
fn test_filename_consistency() {
    // 測試不同模型的檔案路徑（預設資料根目錄為 data/）
    for (i, test_model) in JohansenModel::ALL.iter().enumerate() {
        let test_simulation = EigenvalueSimulation::new(*test_model, 3, 500, 1000).unwrap();
        let path = test_simulation.data_path();
        let expected = std::path::Path::new("data")
            .join("steps500")
            .join(format!("model{i}"))
            .join(format!("eigenvalues_steps500_model{i}_dim3.dat"));
        assert_eq!(path, expected);
    }
}

#[test]
fn test_custom_data_dir() {
    // with_data_dir 只替換根目錄，不影響其餘路徑結構
    let sim = EigenvalueSimulation::new(JohansenModel::NoInterceptNoTrend, 3, 500, 1000)
        .unwrap()
        .with_data_dir("/some/where/else");
    let expected = std::path::Path::new("/some/where/else")
        .join("steps500")
        .join("model0")
        .join("eigenvalues_steps500_model0_dim3.dat");
    assert_eq!(sim.data_path(), expected);
}
