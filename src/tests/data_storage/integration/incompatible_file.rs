//! 驗證「不相容檔案」的新契約：`run()` 回傳可重建錯誤而**不會**自行刪檔，
//! 刪檔重建是呼叫端透過 `delete_data()` 做的顯式決策。

use super::*;

#[test]
fn test_incompatible_file_is_reported_not_deleted() {
    let model = JohansenModel::NoInterceptNoTrend;
    let simulation = test_sim("incompatible_file", model, 2, 107, 3);
    simulation.delete_data();

    // 在資料路徑放一個非 V7 格式的檔案
    let data_path = simulation.data_path();
    std::fs::create_dir_all(data_path.parent().unwrap()).unwrap();
    std::fs::write(&data_path, b"this is not a V7 data file at all!").unwrap();

    // run 應回傳可重建的錯誤，且不得動到既有檔案
    let err = simulation.run().unwrap_err();
    assert!(err.is_recreatable(), "unexpected error kind: {err}");
    assert!(
        data_path.exists(),
        "run() must not delete the incompatible file on its own"
    );

    // load 應回報同類錯誤
    assert!(simulation.load().unwrap_err().is_recreatable());

    // 呼叫端顯式刪檔重建後，一切恢復正常
    simulation.delete_data();
    assert!(!data_path.exists());
    simulation.run().unwrap();
    let data = simulation.load().unwrap();
    assert_eq!(data.len(), 3);

    simulation.delete_data();
}
