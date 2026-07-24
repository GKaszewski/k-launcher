use k_launcher_domain::Plugin;
use plugin_files::FilesPlugin;

#[tokio::test]
async fn files_ignores_non_path_query() {
    let p = FilesPlugin::new();
    assert!(p.search("firefox").await.is_empty());
}

#[tokio::test]
async fn files_handles_root() {
    let p = FilesPlugin::new();
    let results = p.search("/").await;
    assert!(!results.is_empty());
}
