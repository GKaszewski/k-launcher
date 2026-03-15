use async_trait::async_trait;
use k_launcher_kernel::{LaunchAction, Plugin, ResultId, ResultTitle, Score, SearchResult};

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

fn strip_numeric_separators(expr: &str) -> String {
    expr.replace('_', "")
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
    fn name(&self) -> &str {
        "calc"
    }

    async fn search(&self, query: &str) -> Vec<SearchResult> {
        if !should_eval(query) {
            return vec![];
        }
        let raw = query.strip_prefix('=').unwrap_or(query);
        let expr_owned = strip_numeric_separators(raw);
        let expr = expr_owned.as_str();
        match evalexpr::eval_number(expr) {
            Ok(n) if n.is_finite() => {
                let value_str = if n.fract() == 0.0 {
                    format!("{}", n as i64)
                } else {
                    format!("{n}")
                };
                let display = format!("= {value_str}");
                vec![SearchResult {
                    id: ResultId::new("calc-result"),
                    title: ResultTitle::new(display),
                    description: Some(format!("{expr_owned} · Enter to copy")),
                    icon: None,
                    score: Score::new(90),
                    action: LaunchAction::CopyToClipboard(value_str),
                    on_select: None,
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
}
