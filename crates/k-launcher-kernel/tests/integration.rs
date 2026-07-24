use std::sync::Arc;

use k_launcher_kernel::Kernel;
use plugin_calc::CalcPlugin;
use plugin_cmd::CmdPlugin;

fn make_kernel() -> Kernel {
    Kernel::new(
        vec![Arc::new(CalcPlugin::new()), Arc::new(CmdPlugin::new())],
        8,
    )
}

#[tokio::test]
async fn full_pipeline_calc() {
    let kernel = make_kernel();
    let results = kernel.search("2+2").await;
    assert!(!results.is_empty());
    assert_eq!(results[0].title.as_str(), "= 4");
}

#[tokio::test]
async fn full_pipeline_cmd() {
    let kernel = make_kernel();
    let results = kernel.search("> echo hello").await;
    assert!(!results.is_empty());
    assert_eq!(results[0].title.as_str(), "Run: echo hello");
}

#[tokio::test]
async fn full_pipeline_no_match() {
    let kernel = make_kernel();
    let results = kernel.search("xyzzy").await;
    assert!(results.is_empty());
}

#[tokio::test]
async fn full_pipeline_empty_query() {
    let kernel = make_kernel();
    let results = kernel.search("").await;
    assert!(results.is_empty());
}
