/// Johansen 協整檢驗的模型類型
///
/// 這個 enum 定義了 Johansen 協整檢驗中常用的五種模型規格，
/// 每種模型對應不同的趨勢和截距假設。
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
    InterceptNoTrendNoInterceptInCoint,

    /// Model 3: 有截距項，有趨勢項，截距無法完全由協整解釋，協整關係中有趨勢
    /// μ_t = μ_0 + α ρ_1 t
    InterceptTrendWithTrendInCoint,

    /// Model 4: 有截距項，有趨勢項，截距無法完全由協整解釋，趨勢無法完全由協整解釋
    /// μ_t = μ_0 + μ_1 t = μ_0 + (α ρ_1 t + α_⊥ γ_1) t
    InterceptTrendNoTrendInCoint,
}

#[allow(dead_code)]
impl JohansenModel {
    /// 返回模型的數字標識符（0-4）
    pub fn to_number(&self) -> u8 {
        match self {
            JohansenModel::NoInterceptNoTrend => 0,
            JohansenModel::InterceptNoTrendWithInterceptInCoint => 1,
            JohansenModel::InterceptNoTrendNoInterceptInCoint => 2,
            JohansenModel::InterceptTrendWithTrendInCoint => 3,
            JohansenModel::InterceptTrendNoTrendInCoint => 4,
        }
    }

    /// 從數字標識符創建模型
    pub fn from_number(n: u8) -> Option<Self> {
        match n {
            0 => Some(JohansenModel::NoInterceptNoTrend),
            1 => Some(JohansenModel::InterceptNoTrendWithInterceptInCoint),
            2 => Some(JohansenModel::InterceptNoTrendNoInterceptInCoint),
            3 => Some(JohansenModel::InterceptTrendWithTrendInCoint),
            4 => Some(JohansenModel::InterceptTrendNoTrendInCoint),
            _ => None,
        }
    }

    /// 返回模型的描述名稱
    pub fn description(&self) -> &'static str {
        match self {
            JohansenModel::NoInterceptNoTrend => "No intercept, no trend",
            JohansenModel::InterceptNoTrendWithInterceptInCoint => {
                "Intercept, no trend, intercept in cointegration"
            }
            JohansenModel::InterceptNoTrendNoInterceptInCoint => {
                "Intercept, no trend, intercept not fully explained by cointegration"
            }
            JohansenModel::InterceptTrendWithTrendInCoint => {
                "Intercept, trend, trend in cointegration"
            }
            JohansenModel::InterceptTrendNoTrendInCoint => {
                "Intercept, trend, intercept and trend not fully explained by cointegration"
            }
        }
    }

    /// 檢查模型是否包含截距項
    pub fn has_intercept(&self) -> bool {
        match self {
            JohansenModel::NoInterceptNoTrend => false,
            JohansenModel::InterceptNoTrendWithInterceptInCoint
            | JohansenModel::InterceptNoTrendNoInterceptInCoint
            | JohansenModel::InterceptTrendWithTrendInCoint
            | JohansenModel::InterceptTrendNoTrendInCoint => true,
        }
    }

    /// 檢查模型是否包含趨勢項
    pub fn has_trend(&self) -> bool {
        match self {
            JohansenModel::NoInterceptNoTrend
            | JohansenModel::InterceptNoTrendWithInterceptInCoint
            | JohansenModel::InterceptNoTrendNoInterceptInCoint => false,
            JohansenModel::InterceptTrendWithTrendInCoint
            | JohansenModel::InterceptTrendNoTrendInCoint => true,
        }
    }

    /// 檢查截距是否能完全由協整關係解釋
    /// 返回 true 表示截距完全由協整解釋，false 表示截距無法完全由協整解釋
    pub fn intercept_fully_explained_by_cointegration(&self) -> bool {
        match self {
            JohansenModel::NoInterceptNoTrend => false, // 沒有截距項
            JohansenModel::InterceptNoTrendWithInterceptInCoint => true, // μ_t = α ρ_0
            JohansenModel::InterceptNoTrendNoInterceptInCoint => false, // μ_t = α ρ_0 + α_⊥ γ_0
            JohansenModel::InterceptTrendWithTrendInCoint => false, // μ_t = μ_0 + α ρ_1 t
            JohansenModel::InterceptTrendNoTrendInCoint => false, // μ_t = μ_0 + (α ρ_1 + α_⊥ γ_1) t
        }
    }

    /// 檢查趨勢是否能完全由協整關係解釋
    /// 返回 true 表示趨勢完全由協整解釋，false 表示趨勢無法完全由協整解釋
    pub fn trend_fully_explained_by_cointegration(&self) -> bool {
        match self {
            JohansenModel::NoInterceptNoTrend
            | JohansenModel::InterceptNoTrendWithInterceptInCoint
            | JohansenModel::InterceptNoTrendNoInterceptInCoint => false, // 沒有趨勢項
            JohansenModel::InterceptTrendWithTrendInCoint => true, // μ_t = μ_0 + α ρ_1 t
            JohansenModel::InterceptTrendNoTrendInCoint => false, // μ_t = μ_0 + (α ρ_1 + α_⊥ γ_1) t
        }
    }

    /// 返回所有可用的模型
    pub fn all_models() -> [JohansenModel; 5] {
        [
            JohansenModel::NoInterceptNoTrend,
            JohansenModel::InterceptNoTrendWithInterceptInCoint,
            JohansenModel::InterceptNoTrendNoInterceptInCoint,
            JohansenModel::InterceptTrendWithTrendInCoint,
            JohansenModel::InterceptTrendNoTrendInCoint,
        ]
    }
}

impl Default for JohansenModel {
    /// 默認使用 Model 2（最常用的模型）
    fn default() -> Self {
        JohansenModel::InterceptNoTrendNoInterceptInCoint
    }
}

impl std::fmt::Display for JohansenModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Model {}: {}", self.to_number(), self.description())
    }
}
