use std::sync::Arc;

use async_trait::async_trait;
use evalexpr::eval_number_with_context;
use k_launcher_domain::{LaunchAction, Plugin, ResultId, ResultTitle, Score, SearchResult};

use crate::eval::{MATH_CTX, should_eval, strip_numeric_separators};

const RESULT_ID: &str = "calc-result";
const RESULT_SCORE: u32 = 90;

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
        match eval_number_with_context(expr, &*MATH_CTX) {
            Ok(n) if n.is_finite() => {
                let value_str = if n.fract() == 0.0 {
                    format!("{}", n as i64)
                } else {
                    format!("{n}")
                };
                let display = format!("= {value_str}");
                vec![SearchResult {
                    id: ResultId::new(RESULT_ID),
                    title: ResultTitle::new(display),
                    description: Some(Arc::from(format!("{expr_owned} · Enter to copy"))),
                    icon: None,
                    score: Score::new(RESULT_SCORE),
                    action: LaunchAction::CopyToClipboard(value_str),
                }]
            }
            _ => vec![],
        }
    }
}
