use std::sync::Arc;

use async_trait::async_trait;
use k_launcher_kernel::{Plugin, PluginName, ResultId, ResultTitle, Score, SearchResult};

pub struct CalcPlugin;

impl CalcPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CalcPlugin {
    fn default() -> Self {
        Self::new()
    }
}

fn should_eval(query: &str) -> bool {
    query
        .chars()
        .next()
        .map(|c| c.is_ascii_digit() || c == '(' || c == '-')
        .unwrap_or(false)
        || query.starts_with('=')
}

#[async_trait]
impl Plugin for CalcPlugin {
    fn name(&self) -> PluginName {
        "calc"
    }

    async fn search(&self, query: &str) -> Vec<SearchResult> {
        if !should_eval(query) {
            return vec![];
        }
        let expr = query.strip_prefix('=').unwrap_or(query);
        match evalexpr::eval_number(expr) {
            Ok(n) if n.is_finite() => {
                let value_str = if n.fract() == 0.0 {
                    format!("{}", n as i64)
                } else {
                    format!("{n}")
                };
                let display = format!("= {value_str}");
                let expr_owned = expr.to_string();
                let clipboard_val = value_str;
                vec![SearchResult {
                    id: ResultId::new("calc-result"),
                    title: ResultTitle::new(display),
                    description: Some(format!("{expr_owned} · Enter to copy")),
                    icon: None,
                    score: Score::new(90),
                    on_execute: Arc::new(move || {
                        if std::process::Command::new("wl-copy").arg(&clipboard_val).spawn().is_err() {
                            use std::io::Write;
                            if let Ok(mut child) = std::process::Command::new("xclip")
                                .args(["-selection", "clipboard"])
                                .stdin(std::process::Stdio::piped())
                                .spawn()
                            {
                                if let Some(stdin) = child.stdin.as_mut() {
                                    let _ = stdin.write_all(clipboard_val.as_bytes());
                                }
                            }
                        }
                    }),
                }]
            }
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
