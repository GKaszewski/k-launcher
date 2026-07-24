use plugin_url::{is_url, normalize, search};

#[test]
fn is_url_https() {
    assert!(is_url("https://example.com"));
}

#[test]
fn is_url_http() {
    assert!(is_url("http://example.com"));
}

#[test]
fn is_url_www() {
    assert!(is_url("www.foo.com"));
}

#[test]
fn is_url_plain() {
    assert!(!is_url("firefox"));
}

#[test]
fn is_url_empty() {
    assert!(!is_url(""));
}

#[test]
fn normalize_www() {
    assert_eq!(normalize("www.foo.com"), "https://www.foo.com");
}

#[test]
fn normalize_https() {
    assert_eq!(normalize("https://example.com"), "https://example.com");
}

#[test]
fn search_returns_result() {
    let results = search("https://example.com");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].action.path, "https://example.com");
}

#[test]
fn search_returns_empty() {
    assert!(search("firefox").is_empty());
}

#[test]
fn result_serializes() {
    let results = search("https://example.com");
    let json = serde_json::to_string(&results).unwrap();
    assert!(json.contains("OpenPath"));
    assert!(json.contains("https://example.com"));
}
