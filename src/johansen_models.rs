use crate::error::Error;

/// Johansen 協整檢驗的模型類型
///
/// 這個 enum 定義了 Johansen 協整檢驗中常用的五種模型規格，
/// 每種模型對應不同的趨勢和截距假設。
///
/// 五種模型在數學上是封閉集合，因此**不**標記 `#[non_exhaustive]`：
/// 呼叫端可以放心窮舉 match。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JohansenModel {
    /// Model 0: 無截距項，無趨勢項
    /// μ_t = 0
    NoInterceptNoTrend,

    /// Model 1: 有截距項，無趨勢項，協整關係中有截距
    /// μ_t = α ρ_0
    InterceptNoTrendWithInterceptInCoint,

    /// Model 2: 有截距項，無趨勢項，截距無法完全由協整解釋
    /// μ_t = μ_0 = (α ρ_0 + α_⊥ γ_0) 1
    InterceptNoTrendUnrestrictedIntercept,

    /// Model 3: 有截距項，有趨勢項，截距無法完全由協整解釋，協整關係中有趨勢
    /// μ_t = μ_0 + α ρ_1 t
    InterceptTrendUnrestrictedInterceptRestrictedTrend,

    /// Model 4: 有截距項，有趨勢項，截距無法完全由協整解釋，趨勢無法完全由協整解釋
    /// μ_t = μ_0 + μ_1 t = μ_0 + (α ρ_1 t + α_⊥ γ_1) t
    InterceptTrendUnrestrictedBoth,
}

impl JohansenModel {
    /// 全部五種模型，依模型編號 0–4 排序
    pub const ALL: [JohansenModel; 5] = [
        JohansenModel::NoInterceptNoTrend,
        JohansenModel::InterceptNoTrendWithInterceptInCoint,
        JohansenModel::InterceptNoTrendUnrestrictedIntercept,
        JohansenModel::InterceptTrendUnrestrictedInterceptRestrictedTrend,
        JohansenModel::InterceptTrendUnrestrictedBoth,
    ];

    /// Number of eigenvalues produced per simulation run for a given dimension.
    ///
    /// Models 1 and 3 augment the system with an extra deterministic component,
    /// yielding `dim + 1` eigenvalues; all other models yield `dim`.
    pub fn eigenvalues_per_run(&self, dim: usize) -> usize {
        match self {
            JohansenModel::InterceptNoTrendWithInterceptInCoint
            | JohansenModel::InterceptTrendUnrestrictedInterceptRestrictedTrend => dim + 1,
            _ => dim,
        }
    }

    /// 模型的描述名稱（`Display` 的組成部分）
    fn description(&self) -> &'static str {
        match self {
            JohansenModel::NoInterceptNoTrend => "No intercept, no trend",
            JohansenModel::InterceptNoTrendWithInterceptInCoint => {
                "Intercept, no trend, intercept in cointegration"
            }
            JohansenModel::InterceptNoTrendUnrestrictedIntercept => {
                "Intercept, no trend, intercept not fully explained by cointegration"
            }
            JohansenModel::InterceptTrendUnrestrictedInterceptRestrictedTrend => {
                "Intercept, trend, trend in cointegration"
            }
            JohansenModel::InterceptTrendUnrestrictedBoth => {
                "Intercept, trend, intercept and trend not fully explained by cointegration"
            }
        }
    }
}

/// 模型的數字標識符（0–4），與 CLI 的 `--model` 參數及檔案標頭一致
impl From<JohansenModel> for u8 {
    fn from(model: JohansenModel) -> u8 {
        match model {
            JohansenModel::NoInterceptNoTrend => 0,
            JohansenModel::InterceptNoTrendWithInterceptInCoint => 1,
            JohansenModel::InterceptNoTrendUnrestrictedIntercept => 2,
            JohansenModel::InterceptTrendUnrestrictedInterceptRestrictedTrend => 3,
            JohansenModel::InterceptTrendUnrestrictedBoth => 4,
        }
    }
}

/// 從數字標識符（0–4）建立模型；超出範圍回傳 `Error::InvalidConfig`
impl TryFrom<u8> for JohansenModel {
    type Error = Error;

    fn try_from(n: u8) -> Result<Self, Error> {
        match n {
            0 => Ok(JohansenModel::NoInterceptNoTrend),
            1 => Ok(JohansenModel::InterceptNoTrendWithInterceptInCoint),
            2 => Ok(JohansenModel::InterceptNoTrendUnrestrictedIntercept),
            3 => Ok(JohansenModel::InterceptTrendUnrestrictedInterceptRestrictedTrend),
            4 => Ok(JohansenModel::InterceptTrendUnrestrictedBoth),
            _ => Err(Error::InvalidConfig(format!(
                "invalid model number {n}; expected 0..=4"
            ))),
        }
    }
}

impl Default for JohansenModel {
    /// 默認使用 Model 2（最常用的模型）
    fn default() -> Self {
        JohansenModel::InterceptNoTrendUnrestrictedIntercept
    }
}

impl std::fmt::Display for JohansenModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Model {}: {}", u8::from(*self), self.description())
    }
}
