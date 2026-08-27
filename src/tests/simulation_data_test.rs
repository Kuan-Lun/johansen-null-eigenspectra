//! `SimulationData` 統計分析的單元測試：聚合方式與線性內插百分位

use crate::simulation_data::{SimulationData, SimulationRecord, Statistic};

fn data_from_eigenvalues(eigenvalues_per_record: Vec<Vec<f64>>) -> SimulationData {
    let records = eigenvalues_per_record
        .into_iter()
        .enumerate()
        .map(|(i, eigenvalues)| SimulationRecord {
            seed: i as u32 + 1,
            eigenvalues,
        })
        .collect();
    SimulationData::new(records, true)
}

#[test]
fn test_trace_aggregates_by_sum() {
    // 每筆記錄的 trace 值：3.0, 7.0, 11.0
    let data = data_from_eigenvalues(vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]]);
    // 0th/50th/100th 百分位即最小值、中位數、最大值
    let values = data.percentiles(Statistic::Trace, &[0.0, 0.5, 1.0]);
    assert_eq!(values, vec![3.0, 7.0, 11.0]);
}

#[test]
fn test_maxeig_aggregates_by_max() {
    // 每筆記錄的最大特徵值：2.0, 9.0, 6.0
    let data = data_from_eigenvalues(vec![vec![1.0, 2.0], vec![9.0, 4.0], vec![5.0, 6.0]]);
    let values = data.percentiles(Statistic::MaxEigenvalue, &[0.0, 0.5, 1.0]);
    assert_eq!(values, vec![2.0, 6.0, 9.0]);
}

#[test]
fn test_percentile_linear_interpolation() {
    // 排序後的 trace 值為 [1.0, 2.0, 3.0, 4.0]；
    // 0.5 percentile 落在 rank 1.5 → 內插為 2.5
    let data = data_from_eigenvalues(vec![vec![2.0], vec![4.0], vec![1.0], vec![3.0]]);
    let values = data.percentiles(Statistic::Trace, &[0.5]);
    assert_eq!(values, vec![2.5]);
}

#[test]
fn test_percentiles_result_matches_input_length() {
    let data = data_from_eigenvalues(vec![vec![1.0], vec![2.0]]);
    let percentiles = [0.1, 0.5, 0.9, 0.95, 0.99];
    let values = data.percentiles(Statistic::Trace, &percentiles);
    assert_eq!(values.len(), percentiles.len());
}

#[test]
fn test_empty_data_yields_nan() {
    let data = SimulationData::new(vec![], false);
    assert!(data.is_empty());
    assert_eq!(data.len(), 0);
    let values = data.percentiles(Statistic::Trace, &[0.5, 0.95]);
    assert_eq!(values.len(), 2);
    assert!(values.iter().all(|v| v.is_nan()));
}

#[test]
fn test_accessors() {
    let data = data_from_eigenvalues(vec![vec![1.0], vec![2.0]]);
    assert_eq!(data.len(), 2);
    assert!(!data.is_empty());
    assert!(data.is_complete());
    assert_eq!(data.records()[0].seed, 1);
    assert_eq!(data.records()[1].eigenvalues, vec![2.0]);
}
