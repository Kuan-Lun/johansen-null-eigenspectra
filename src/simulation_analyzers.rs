use crate::data_storage::EigenvalueSimulation;

/// 輸出百分位數統計資訊，使用內插法計算百分位值
fn get_percentile_value(sorted_values: &[f64], percentile: f64) -> f64 {
    let n = sorted_values.len();
    if n == 0 {
        return f64::NAN;
    }
    let rank = percentile * (n - 1) as f64;
    let lower_index = rank.floor() as usize;
    let upper_index = rank.ceil() as usize;
    if lower_index == upper_index {
        sorted_values[lower_index]
    } else {
        let weight = rank - lower_index as f64;
        sorted_values[lower_index] * (1.0 - weight) + sorted_values[upper_index] * weight
    }
}

/// 聚合函數 trait
pub trait Aggregator {
    fn aggregate(&self, eigenvalues: &[f64]) -> f64;
}

/// 聚合函數實作：計算總和
pub struct SumAggregator;
impl Aggregator for SumAggregator {
    fn aggregate(&self, eigenvalues: &[f64]) -> f64 {
        eigenvalues.iter().sum()
    }
}

/// 聚合函數實作：計算最大值
pub struct MaxAggregator;
impl Aggregator for MaxAggregator {
    fn aggregate(&self, eigenvalues: &[f64]) -> f64 {
        eigenvalues.iter().cloned().fold(f64::MIN, f64::max)
    }
}

/// 計算指定百分位數的值
pub fn calculate_percentiles<A: Aggregator>(
    simulation: &EigenvalueSimulation,
    aggregator: A,
    percentiles: &[f64],
) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    let data = simulation.read_data()?;
    if data.is_empty() {
        return Ok(vec![]);
    }

    let values: Vec<f64> = data
        .iter()
        .map(|(_, eigenvalues)| aggregator.aggregate(eigenvalues))
        .collect();
    let mut sorted_values = values;
    sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let results = percentiles
        .iter()
        .map(|&percentile| get_percentile_value(&sorted_values, percentile))
        .collect();

    Ok(results)
}

impl EigenvalueSimulation {
    pub fn calculate_trace_percentiles(
        &self,
        percentiles: &[f64],
    ) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
        calculate_percentiles(self, SumAggregator, percentiles)
    }

    pub fn calculate_maxeig_percentiles(
        &self,
        percentiles: &[f64],
    ) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
        calculate_percentiles(self, MaxAggregator, percentiles)
    }
}
