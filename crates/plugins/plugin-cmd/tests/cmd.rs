use k_launcher_domain::Plugin;
use plugin_cmd::CmdPlugin;

#[tokio::test]
async fn cmd_prefix_triggers() {
    let p = CmdPlugin::new();
    let results = p.search("> echo hello").await;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title.as_str(), "Run: echo hello");
    assert_eq!(results[0].score.value(), 95);
}

#[tokio::test]
async fn cmd_empty_remainder_returns_empty() {
    let p = CmdPlugin::new();
    assert!(p.search(">").await.is_empty());
    assert!(p.search(">   ").await.is_empty());
}

#[tokio::test]
async fn cmd_no_prefix_returns_empty() {
    let p = CmdPlugin::new();
    assert!(p.search("echo hello").await.is_empty());
    assert!(p.search("firefox").await.is_empty());
}
