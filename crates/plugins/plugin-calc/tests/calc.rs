use k_launcher_domain::{LaunchAction, Plugin};
use plugin_calc::CalcPlugin;

#[tokio::test]
async fn calc_valid_expr() {
    let p = CalcPlugin::new();
    let results = p.search("2+2").await;
    assert_eq!(results[0].title.as_str(), "= 4");
}

#[tokio::test]
async fn calc_non_numeric_returns_empty() {
    let p = CalcPlugin::new();
    assert!(p.search("firefox").await.is_empty());
}

#[tokio::test]
async fn calc_bad_expr_returns_empty() {
    let p = CalcPlugin::new();
    assert!(p.search("1/0").await.is_empty());
}

#[tokio::test]
async fn calc_sqrt() {
    let p = CalcPlugin::new();
    let results = p.search("sqrt(9)").await;
    assert_eq!(results[0].title.as_str(), "= 3");
}

#[tokio::test]
async fn calc_sin_pi() {
    let p = CalcPlugin::new();
    let results = p.search("sin(pi)").await;
    assert!(!results.is_empty());
    let title = results[0].title.as_str();
    let val: f64 = title.trim_start_matches("= ").parse().unwrap();
    assert!(val.abs() < 1e-10, "sin(pi) should be near zero, got {val}");
}

#[tokio::test]
async fn calc_underscore_separator() {
    let p = CalcPlugin::new();
    let results = p.search("1_000 * 2").await;
    assert_eq!(results[0].title.as_str(), "= 2000");
    assert_eq!(
        results[0].description.as_deref(),
        Some("1000 * 2 · Enter to copy")
    );
    assert!(matches!(
        &results[0].action,
        LaunchAction::CopyToClipboard(v) if v == "2000"
    ));
}
