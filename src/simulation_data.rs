//! In-memory simulation results and their statistical analysis.
//!
//! `load()` / `load_partial()` 將檔案內容一次載入為 [`SimulationData`]，
//! 之後的統計分析（百分位數）皆在記憶體中進行，不再重複讀檔。

/// 單筆模擬記錄：seed 與該次模擬產生的特徵值
#[derive(Debug, Clone, PartialEq)]
pub struct SimulationRecord {
    /// 產生此記錄的隨機數種子
    pub seed: u32,
    /// 該次模擬產生的特徵值
    pub eigenvalues: Vec<f64>,
}

/// 檢定統計量：將一筆記錄的特徵值向量縮減為單一數值的方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Statistic {
    /// Trace 統計量：特徵值總和
    Trace,
    /// Maximum eigenvalue 統計量：最大特徵值
    MaxEigenvalue,
}

impl Statistic {
    fn aggregate(self, eigenvalues: &[f64]) -> f64 {
        match self {
            Statistic::Trace => eigenvalues.iter().sum(),
            Statistic::MaxEigenvalue => eigenvalues.iter().copied().fold(f64::MIN, f64::max),
        }
    }
}

/// 已載入記憶體的模擬結果集，由
/// [`EigenvalueSimulation::load`](crate::EigenvalueSimulation::load) 或
/// [`EigenvalueSimulation::load_partial`](crate::EigenvalueSimulation::load_partial)
/// 產生
#[derive(Debug, Clone, PartialEq)]
pub struct SimulationData {
    records: Vec<SimulationRecord>,
    complete: bool,
}

impl SimulationData {
    pub(crate) fn new(records: Vec<SimulationRecord>, complete: bool) -> Self {
        Self { records, complete }
    }

    /// 依 seed 順序排列的全部記錄
    pub fn records(&self) -> &[SimulationRecord] {
        &self.records
    }

    /// 是否包含配置要求的每一個 seed（`1..=num_runs`）而無缺漏。
    /// `load()` 成功時必為 `true`；`load_partial()` 讀到部分資料時為 `false`。
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    /// 記錄筆數
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// 是否沒有任何記錄
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// 計算指定統計量在給定百分位數（0.0–1.0）上的值，使用線性內插。
    ///
    /// 回傳向量與 `percentiles` 等長；沒有任何記錄時每個位置為 NaN。
    pub fn percentiles(&self, statistic: Statistic, percentiles: &[f64]) -> Vec<f64> {
        let mut values: Vec<f64> = self
            .records
            .iter()
            .map(|record| statistic.aggregate(&record.eigenvalues))
            .collect();
        values.sort_by(f64::total_cmp);

        percentiles
            .iter()
            .map(|&percentile| percentile_value(&values, percentile))
            .collect()
    }
}

/// 以線性內插取排序值的百分位值
fn percentile_value(sorted_values: &[f64], percentile: f64) -> f64 {
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
