use crate::johansen_models::JohansenModel;

#[test]
fn test_model_numbers() {
    assert_eq!(u8::from(JohansenModel::NoInterceptNoTrend), 0);
    assert_eq!(
        u8::from(JohansenModel::InterceptNoTrendWithInterceptInCoint),
        1
    );
    assert_eq!(
        u8::from(JohansenModel::InterceptNoTrendUnrestrictedIntercept),
        2
    );
    assert_eq!(
        u8::from(JohansenModel::InterceptTrendUnrestrictedInterceptRestrictedTrend),
        3
    );
    assert_eq!(u8::from(JohansenModel::InterceptTrendUnrestrictedBoth), 4);
}

#[test]
fn test_try_from_number() {
    assert_eq!(
        JohansenModel::try_from(0).unwrap(),
        JohansenModel::NoInterceptNoTrend
    );
    assert_eq!(
        JohansenModel::try_from(1).unwrap(),
        JohansenModel::InterceptNoTrendWithInterceptInCoint
    );
    assert_eq!(
        JohansenModel::try_from(2).unwrap(),
        JohansenModel::InterceptNoTrendUnrestrictedIntercept
    );
    assert_eq!(
        JohansenModel::try_from(3).unwrap(),
        JohansenModel::InterceptTrendUnrestrictedInterceptRestrictedTrend
    );
    assert_eq!(
        JohansenModel::try_from(4).unwrap(),
        JohansenModel::InterceptTrendUnrestrictedBoth
    );
    assert!(JohansenModel::try_from(5).is_err());
    assert!(JohansenModel::try_from(255).is_err());
}

#[test]
fn test_number_roundtrip() {
    for model in JohansenModel::ALL {
        assert_eq!(JohansenModel::try_from(u8::from(model)).unwrap(), model);
    }
}

#[test]
fn test_eigenvalues_per_run() {
    // Models 1 和 3 有額外的確定性成分：dim + 1 個特徵值
    assert_eq!(JohansenModel::NoInterceptNoTrend.eigenvalues_per_run(3), 3);
    assert_eq!(
        JohansenModel::InterceptNoTrendWithInterceptInCoint.eigenvalues_per_run(3),
        4
    );
    assert_eq!(
        JohansenModel::InterceptNoTrendUnrestrictedIntercept.eigenvalues_per_run(3),
        3
    );
    assert_eq!(
        JohansenModel::InterceptTrendUnrestrictedInterceptRestrictedTrend.eigenvalues_per_run(3),
        4
    );
    assert_eq!(
        JohansenModel::InterceptTrendUnrestrictedBoth.eigenvalues_per_run(3),
        3
    );
}

#[test]
fn test_all_models() {
    assert_eq!(JohansenModel::ALL.len(), 5);
    // ALL 依模型編號 0–4 排序
    for (i, model) in JohansenModel::ALL.iter().enumerate() {
        assert_eq!(u8::from(*model), i as u8);
    }
}

#[test]
fn test_default() {
    let default_model = JohansenModel::default();
    assert_eq!(
        default_model,
        JohansenModel::InterceptNoTrendUnrestrictedIntercept
    );
    assert_eq!(u8::from(default_model), 2);
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
    let cloned = model;
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
