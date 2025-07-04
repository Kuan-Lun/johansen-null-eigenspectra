use crate::johansen_models::JohansenModel;

#[test]
fn test_model_numbers() {
    assert_eq!(JohansenModel::NoInterceptNoTrend.to_number(), 0);
    assert_eq!(
        JohansenModel::InterceptNoTrendWithInterceptInCoint.to_number(),
        1
    );
    assert_eq!(
        JohansenModel::InterceptNoTrendUnrestrictedIntercept.to_number(),
        2
    );
    assert_eq!(JohansenModel::InterceptTrendUnrestrictedInterceptRestrictedTrend.to_number(), 3);
    assert_eq!(JohansenModel::InterceptTrendUnrestrictedBoth.to_number(), 4);
}

#[test]
fn test_from_number() {
    assert_eq!(
        JohansenModel::from_number(0),
        Some(JohansenModel::NoInterceptNoTrend)
    );
    assert_eq!(
        JohansenModel::from_number(1),
        Some(JohansenModel::InterceptNoTrendWithInterceptInCoint)
    );
    assert_eq!(
        JohansenModel::from_number(2),
        Some(JohansenModel::InterceptNoTrendUnrestrictedIntercept)
    );
    assert_eq!(
        JohansenModel::from_number(3),
        Some(JohansenModel::InterceptTrendUnrestrictedInterceptRestrictedTrend)
    );
    assert_eq!(
        JohansenModel::from_number(4),
        Some(JohansenModel::InterceptTrendUnrestrictedBoth)
    );
    assert_eq!(JohansenModel::from_number(5), None);
    assert_eq!(JohansenModel::from_number(255), None);
}

#[test]
fn test_model_descriptions() {
    assert_eq!(
        JohansenModel::NoInterceptNoTrend.description(),
        "No intercept, no trend"
    );
    assert_eq!(
        JohansenModel::InterceptNoTrendWithInterceptInCoint.description(),
        "Intercept, no trend, intercept in cointegration"
    );
    assert_eq!(
        JohansenModel::InterceptNoTrendUnrestrictedIntercept.description(),
        "Intercept, no trend, intercept not fully explained by cointegration"
    );
    assert_eq!(
        JohansenModel::InterceptTrendUnrestrictedInterceptRestrictedTrend.description(),
        "Intercept, trend, trend in cointegration"
    );
    assert_eq!(
        JohansenModel::InterceptTrendUnrestrictedBoth.description(),
        "Intercept, trend, intercept and trend not fully explained by cointegration"
    );
}

#[test]
fn test_has_intercept() {
    assert!(!JohansenModel::NoInterceptNoTrend.has_intercept());
    assert!(JohansenModel::InterceptNoTrendWithInterceptInCoint.has_intercept());
    assert!(JohansenModel::InterceptNoTrendUnrestrictedIntercept.has_intercept());
    assert!(JohansenModel::InterceptTrendUnrestrictedInterceptRestrictedTrend.has_intercept());
    assert!(JohansenModel::InterceptTrendUnrestrictedBoth.has_intercept());
}

#[test]
fn test_has_trend() {
    assert!(!JohansenModel::NoInterceptNoTrend.has_trend());
    assert!(!JohansenModel::InterceptNoTrendWithInterceptInCoint.has_trend());
    assert!(!JohansenModel::InterceptNoTrendUnrestrictedIntercept.has_trend());
    assert!(JohansenModel::InterceptTrendUnrestrictedInterceptRestrictedTrend.has_trend());
    assert!(JohansenModel::InterceptTrendUnrestrictedBoth.has_trend());
}

#[test]
fn test_intercept_fully_explained_by_cointegration() {
    assert!(!JohansenModel::NoInterceptNoTrend.intercept_fully_explained_by_cointegration());
    assert!(
        JohansenModel::InterceptNoTrendWithInterceptInCoint
            .intercept_fully_explained_by_cointegration()
    );
    assert!(
        !JohansenModel::InterceptNoTrendUnrestrictedIntercept
            .intercept_fully_explained_by_cointegration()
    );
    assert!(
        !JohansenModel::InterceptTrendUnrestrictedInterceptRestrictedTrend.intercept_fully_explained_by_cointegration()
    );
    assert!(
        !JohansenModel::InterceptTrendUnrestrictedBoth.intercept_fully_explained_by_cointegration()
    );
}

#[test]
fn test_trend_fully_explained_by_cointegration() {
    assert!(!JohansenModel::NoInterceptNoTrend.trend_fully_explained_by_cointegration());
    assert!(
        !JohansenModel::InterceptNoTrendWithInterceptInCoint
            .trend_fully_explained_by_cointegration()
    );
    assert!(
        !JohansenModel::InterceptNoTrendUnrestrictedIntercept.trend_fully_explained_by_cointegration()
    );
    assert!(JohansenModel::InterceptTrendUnrestrictedInterceptRestrictedTrend.trend_fully_explained_by_cointegration());
    assert!(!JohansenModel::InterceptTrendUnrestrictedBoth.trend_fully_explained_by_cointegration());
}

#[test]
fn test_all_models() {
    let all_models = JohansenModel::all_models();
    assert_eq!(all_models.len(), 5);
    assert_eq!(all_models[0], JohansenModel::NoInterceptNoTrend);
    assert_eq!(
        all_models[1],
        JohansenModel::InterceptNoTrendWithInterceptInCoint
    );
    assert_eq!(
        all_models[2],
        JohansenModel::InterceptNoTrendUnrestrictedIntercept
    );
    assert_eq!(all_models[3], JohansenModel::InterceptTrendUnrestrictedInterceptRestrictedTrend);
    assert_eq!(all_models[4], JohansenModel::InterceptTrendUnrestrictedBoth);
}

#[test]
fn test_default() {
    let default_model = JohansenModel::default();
    assert_eq!(
        default_model,
        JohansenModel::InterceptNoTrendUnrestrictedIntercept
    );
    assert_eq!(default_model.to_number(), 2);
}

#[test]
fn test_display() {
    assert_eq!(
        JohansenModel::NoInterceptNoTrend.to_string(),
        "Model 0: No intercept, no trend"
    );
    assert_eq!(
        JohansenModel::InterceptNoTrendWithInterceptInCoint.to_string(),
        "Model 1: Intercept, no trend, intercept in cointegration"
    );
    assert_eq!(
        JohansenModel::InterceptNoTrendUnrestrictedIntercept.to_string(),
        "Model 2: Intercept, no trend, intercept not fully explained by cointegration"
    );
    assert_eq!(
        JohansenModel::InterceptTrendUnrestrictedInterceptRestrictedTrend.to_string(),
        "Model 3: Intercept, trend, trend in cointegration"
    );
    assert_eq!(
        JohansenModel::InterceptTrendUnrestrictedBoth.to_string(),
        "Model 4: Intercept, trend, intercept and trend not fully explained by cointegration"
    );
}

#[test]
fn test_clone_copy_traits() {
    let model = JohansenModel::InterceptNoTrendUnrestrictedIntercept;
    let cloned = model.clone();
    let copied = model;

    assert_eq!(model, cloned);
    assert_eq!(model, copied);
}

#[test]
fn test_debug_trait() {
    let model = JohansenModel::InterceptNoTrendUnrestrictedIntercept;
    let debug_str = format!("{:?}", model);
    assert_eq!(debug_str, "InterceptNoTrendUnrestrictedIntercept");
}

#[test]
fn test_comprehensive_model_properties() {
    // 測試每個模型的屬性組合是否正確

    let model0 = JohansenModel::NoInterceptNoTrend;
    assert!(!model0.has_intercept());
    assert!(!model0.has_trend());
    assert!(!model0.intercept_fully_explained_by_cointegration());
    assert!(!model0.trend_fully_explained_by_cointegration());

    let model1 = JohansenModel::InterceptNoTrendWithInterceptInCoint;
    assert!(model1.has_intercept());
    assert!(!model1.has_trend());
    assert!(model1.intercept_fully_explained_by_cointegration());
    assert!(!model1.trend_fully_explained_by_cointegration());

    let model2 = JohansenModel::InterceptNoTrendUnrestrictedIntercept;
    assert!(model2.has_intercept());
    assert!(!model2.has_trend());
    assert!(!model2.intercept_fully_explained_by_cointegration());
    assert!(!model2.trend_fully_explained_by_cointegration());

    let model3 = JohansenModel::InterceptTrendUnrestrictedInterceptRestrictedTrend;
    assert!(model3.has_intercept());
    assert!(model3.has_trend());
    assert!(!model3.intercept_fully_explained_by_cointegration());
    assert!(model3.trend_fully_explained_by_cointegration());

    let model4 = JohansenModel::InterceptTrendUnrestrictedBoth;
    assert!(model4.has_intercept());
    assert!(model4.has_trend());
    assert!(!model4.intercept_fully_explained_by_cointegration());
    assert!(!model4.trend_fully_explained_by_cointegration());
}
