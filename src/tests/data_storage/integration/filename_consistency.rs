use super::*;

#[test]
fn test_filename_consistency() {
    // 測試不同模型的檔案路徑
    for (i, test_model) in JohansenModel::all_models().iter().enumerate() {
        let test_simulation = EigenvalueSimulation::new(*test_model, 3, 500, 1000);
        let path = test_simulation.get_file_path(*test_model);
        let expected = std::path::Path::new("data")
            .join("steps500")
            .join(format!("model{i}"))
            .join(format!("eigenvalues_steps500_model{i}_dim3.dat"));
        assert_eq!(path, expected);
    }
}
