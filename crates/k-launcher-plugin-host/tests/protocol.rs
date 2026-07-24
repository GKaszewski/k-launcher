use k_launcher_plugin_host::{ExternalAction, ExternalPlugin, ExternalResult, Query};

#[test]
fn query_serializes_correctly() {
    let q = Query {
        query: "firefox".to_string(),
    };
    assert_eq!(serde_json::to_string(&q).unwrap(), r#"{"query":"firefox"}"#);
}

#[test]
fn result_parses_spawn_action() {
    let json = r#"[{"id":"1","title":"Firefox","score":80,"action":{"type":"SpawnProcess","cmd":"firefox"}}]"#;
    let results: Vec<ExternalResult> = serde_json::from_str(json).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "1");
    assert_eq!(results[0].title, "Firefox");
    assert_eq!(results[0].score, 80);
    assert!(matches!(&results[0].action, ExternalAction::SpawnProcess { cmd } if cmd == "firefox"));
}

#[test]
fn result_parses_copy_action() {
    let json =
        r#"[{"id":"c","title":"= 4","score":90,"action":{"type":"CopyToClipboard","text":"4"}}]"#;
    let results: Vec<ExternalResult> = serde_json::from_str(json).unwrap();
    assert!(matches!(&results[0].action, ExternalAction::CopyToClipboard { text } if text == "4"));
}

#[test]
fn result_parses_open_path_action() {
    let json = r#"[{"id":"f","title":"/home/user","score":50,"action":{"type":"OpenPath","path":"/home/user"}}]"#;
    let results: Vec<ExternalResult> = serde_json::from_str(json).unwrap();
    assert!(
        matches!(&results[0].action, ExternalAction::OpenPath { path } if path == "/home/user")
    );
}

#[test]
fn result_parses_spawn_in_terminal_action() {
    let json = r#"[{"id":"t","title":"htop","score":70,"action":{"type":"SpawnInTerminal","cmd":"htop"}}]"#;
    let results: Vec<ExternalResult> = serde_json::from_str(json).unwrap();
    assert!(matches!(&results[0].action, ExternalAction::SpawnInTerminal { cmd } if cmd == "htop"));
}

#[test]
fn result_parses_optional_fields() {
    let json = r#"[{"id":"x","title":"X","score":10,"description":"desc","icon":"/icon.png","action":{"type":"SpawnProcess","cmd":"x"}}]"#;
    let results: Vec<ExternalResult> = serde_json::from_str(json).unwrap();
    assert_eq!(results[0].description.as_deref(), Some("desc"));
    assert_eq!(results[0].icon.as_deref(), Some("/icon.png"));
}

#[test]
fn result_parses_missing_optional_fields() {
    let json = r#"[{"id":"x","title":"X","score":10,"action":{"type":"SpawnProcess","cmd":"x"}}]"#;
    let results: Vec<ExternalResult> = serde_json::from_str(json).unwrap();
    assert!(results[0].description.is_none());
    assert!(results[0].icon.is_none());
}

#[test]
fn invalid_json_is_err() {
    assert!(serde_json::from_str::<Vec<ExternalResult>>("not json").is_err());
}

fn _assert_send_sync() {
    fn check<T: Send + Sync>() {}
    check::<ExternalPlugin>();
}
