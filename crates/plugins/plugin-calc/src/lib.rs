use async_trait::async_trait;
use evalexpr::eval_number_with_context;
use k_launcher_kernel::{LaunchAction, Plugin, ResultId, ResultTitle, Score, SearchResult};
use std::sync::LazyLock;

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

const MATH_FNS: &[&str] = &[
    "sqrt", "sin", "cos", "tan", "asin", "acos", "atan", "ln", "log2", "log10", "exp", "abs",
    "ceil", "floor", "round",
];

fn should_eval(query: &str) -> bool {
    let q = query.strip_prefix('=').unwrap_or(query);
    q.chars()
        .next()
        .map(|c| c.is_ascii_digit() || c == '(' || c == '-')
        .unwrap_or(false)
        || query.starts_with('=')
        || MATH_FNS.iter().any(|f| q.starts_with(f))
}

static MATH_CTX: LazyLock<evalexpr::HashMapContext<evalexpr::DefaultNumericTypes>> = LazyLock::new(
    || {
        use evalexpr::*;
        context_map! {
            "pi" => float std::f64::consts::PI,
            "e"  => float std::f64::consts::E,
            "sqrt"  => Function::new(|a: &Value<DefaultNumericTypes>| Ok(Value::from_float(a.as_number()?.sqrt()))),
            "sin"   => Function::new(|a: &Value<DefaultNumericTypes>| Ok(Value::from_float(a.as_number()?.sin()))),
            "cos"   => Function::new(|a: &Value<DefaultNumericTypes>| Ok(Value::from_float(a.as_number()?.cos()))),
            "tan"   => Function::new(|a: &Value<DefaultNumericTypes>| Ok(Value::from_float(a.as_number()?.tan()))),
            "asin"  => Function::new(|a: &Value<DefaultNumericTypes>| Ok(Value::from_float(a.as_number()?.asin()))),
            "acos"  => Function::new(|a: &Value<DefaultNumericTypes>| Ok(Value::from_float(a.as_number()?.acos()))),
            "atan"  => Function::new(|a: &Value<DefaultNumericTypes>| Ok(Value::from_float(a.as_number()?.atan()))),
            "ln"    => Function::new(|a: &Value<DefaultNumericTypes>| Ok(Value::from_float(a.as_number()?.ln()))),
            "log2"  => Function::new(|a: &Value<DefaultNumericTypes>| Ok(Value::from_float(a.as_number()?.log2()))),
            "log10" => Function::new(|a: &Value<DefaultNumericTypes>| Ok(Value::from_float(a.as_number()?.log10()))),
            "exp"   => Function::new(|a: &Value<DefaultNumericTypes>| Ok(Value::from_float(a.as_number()?.exp()))),
            "abs"   => Function::new(|a: &Value<DefaultNumericTypes>| Ok(Value::from_float(a.as_number()?.abs()))),
            "ceil"  => Function::new(|a: &Value<DefaultNumericTypes>| Ok(Value::from_float(a.as_number()?.ceil()))),
            "floor" => Function::new(|a: &Value<DefaultNumericTypes>| Ok(Value::from_float(a.as_number()?.floor()))),
            "round" => Function::new(|a: &Value<DefaultNumericTypes>| Ok(Value::from_float(a.as_number()?.round())))
        }
        .expect("static math context must be valid")
    },
);

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
}
