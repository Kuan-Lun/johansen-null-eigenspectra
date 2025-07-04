use super::*;

/// 重寫追加格式檔案的測試輔助函數
pub fn rewrite_append_file(
    filename: &str,
    data: &[(u32, Vec<f64>)],
    model: u8,
    dim: u8,
    steps: u32,
) -> std::io::Result<()> {
    // 刪除舊檔案
    let _ = std::fs::remove_file(filename);

    // 使用追加寫入器重建檔案
    let mut writer = AppendOnlyWriter::with_expected_size(filename, None, model, dim, steps, true)?;

    for (seed, eigenvalues) in data {
        writer.append_eigenvalues(*seed, eigenvalues)?;
    }

    writer.finish()
}

/// 從檔案中移除指定的seed數據（測試用）
pub fn remove_seed_from_file(
    simulation: &EigenvalueSimulation,
    model: JohansenModel,
    seeds_to_remove: &[u32],
) -> std::io::Result<usize> {
    let filename = simulation.get_filename(model);

    if !std::path::Path::new(&filename).exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "檔案不存在",
        ));
    }

    // 讀取現有數據並排序
    let mut data = simulation.read_data()?;
    data.sort_by_key(|(seed, _)| *seed);
    let original_count = data.len();

    // 過濾掉指定的seeds
    let filtered_data: Vec<(u32, Vec<f64>)> = data
        .into_iter()
        .filter(|(seed, _)| !seeds_to_remove.contains(seed))
        .collect();

    let removed_count = original_count - filtered_data.len();

    if removed_count > 0 {
        // 備份原檔案
        let backup_filename = format!("{}.test_backup", filename);
        std::fs::copy(&filename, &backup_filename)?;

        // 重寫檔案
        rewrite_append_file(
            &filename,
            &filtered_data,
            model.to_number(),
            simulation.dim as u8,
            simulation.steps as u32,
        )?;
    }

    Ok(removed_count)
}
