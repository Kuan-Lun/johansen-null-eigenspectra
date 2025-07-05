use super::*;

#[test]
fn test_filename_consistency() {
    // 測試不同模型的檔案名稱
    for (i, test_model) in JohansenModel::all_models().iter().enumerate() {
        let test_simulation = EigenvalueSimulation::new(*test_model, 3, 500, 1000);
        let filename = test_simulation.get_filename(*test_model);
        assert!(filename.contains(&format!("model{}", i)));
        assert!(filename.contains("dim3"));
        assert!(filename.contains("steps500"));
        assert!(filename.ends_with(".dat"));
    }
}
