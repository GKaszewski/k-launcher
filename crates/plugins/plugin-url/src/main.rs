use std::io::{self, BufRead, Write};

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Query {
    query: String,
}

#[derive(Serialize)]
struct Action {
    r#type: &'static str,
    cmd: String,
}

#[derive(Serialize)]
struct Result {
    id: &'static str,
    title: &'static str,
    description: String,
    score: u32,
    action: Action,
}

fn is_url(query: &str) -> bool {
    query.starts_with("http://") || query.starts_with("https://") || query.starts_with("www.")
}

fn normalize(query: &str) -> String {
    if query.starts_with("www.") {
        format!("https://{query}")
    } else {
        query.to_string()
    }
}

fn search(query: &str) -> Vec<Result> {
    if !is_url(query) {
        return vec![];
    }
    let url = normalize(query);
    vec![Result {
        id: "url-open",
        title: "Open in Browser",
        description: url.clone(),
        score: 95,
        action: Action {
            r#type: "SpawnProcess",
            cmd: format!("xdg-open {url}"),
        },
    }]
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line?;
        let q: Query = match serde_json::from_str(&line) {
            Ok(q) => q,
            Err(_) => continue,
        };
        let results = search(&q.query);
        writeln!(out, "{}", serde_json::to_string(&results).unwrap())?;
        out.flush()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(results[0].action.cmd, "xdg-open https://example.com");
    }

    #[test]
    fn search_returns_empty() {
        assert!(search("firefox").is_empty());
    }

    #[test]
    fn result_serializes() {
        let results = search("https://example.com");
        let json = serde_json::to_string(&results).unwrap();
        assert!(json.contains("SpawnProcess"));
        assert!(json.contains("xdg-open"));
    }
}
