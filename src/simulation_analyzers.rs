use crate::data_storage::EigenvalueSimulation;
use crate::display_utils::format_number_with_commas;

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

/// 分析 trait，定義分析方法接口
pub trait SimulationAnalyzer {
    fn analyze(&self, simulation: &EigenvalueSimulation);
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

/// 模板分析器，使用聚合函數和自定義標題
pub struct TemplateAnalyzer<A: Aggregator> {
    aggregator: A,
    title: String,
}

impl<A: Aggregator> TemplateAnalyzer<A> {
    pub fn new(aggregator: A, title: impl Into<String>) -> Self {
        Self {
            aggregator,
            title: title.into(),
        }
    }
}

impl<A: Aggregator> SimulationAnalyzer for TemplateAnalyzer<A> {
    fn analyze(&self, simulation: &EigenvalueSimulation) {
        match simulation.read_data() {
            Ok(data) => {
                if !data.is_empty() {
                    let values: Vec<f64> = data
                        .iter()
                        .map(|(_, eigenvalues)| self.aggregator.aggregate(eigenvalues))
                        .collect();
                    let mut sorted_values = values;
                    sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let percentiles = vec![0.5, 0.75, 0.8, 0.85, 0.9, 0.95, 0.975, 0.99];
                    println!("{} for model {}:", self.title, simulation.model);
                    println!(
                        "Total calculated {} values",
                        format_number_with_commas(sorted_values.len())
                    );
                    for &percentile in &percentiles {
                        let value = get_percentile_value(&sorted_values, percentile);
                        println!("{:.0}th percentile value: {:.6}", percentile * 100.0, value);
                    }
                }
            }
            Err(_) => {
                // 如果讀取失敗，忽略這個模型
            }
        }
    }
}

// 使用模板分析器替代原本的 TraceAnalyzer 和 MaxEigAnalyzer
pub type TraceAnalyzer = TemplateAnalyzer<SumAggregator>;
pub type MaxEigAnalyzer = TemplateAnalyzer<MaxAggregator>;

impl EigenvalueSimulation {
    pub fn analyze_trace(&self) {
        let analyzer = TraceAnalyzer::new(SumAggregator, "Trace");
        analyzer.analyze(self);
    }

    pub fn analyze_maxeig(&self) {
        let analyzer = MaxEigAnalyzer::new(MaxAggregator, "MaxEig");
        analyzer.analyze(self);
    }
}
