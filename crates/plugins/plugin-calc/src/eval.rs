use std::sync::LazyLock;

pub(crate) const MATH_FNS: &[&str] = &[
    "sqrt", "sin", "cos", "tan", "asin", "acos", "atan", "ln", "log2", "log10", "exp", "abs",
    "ceil", "floor", "round",
];

pub(crate) fn strip_numeric_separators(expr: &str) -> String {
    expr.replace('_', "")
}

pub(crate) fn should_eval(query: &str) -> bool {
    let q = query.strip_prefix('=').unwrap_or(query);
    q.chars()
        .next()
        .map(|c| c.is_ascii_digit() || c == '(' || c == '-')
        .unwrap_or(false)
        || query.starts_with('=')
        || MATH_FNS.iter().any(|f| q.starts_with(f))
}

pub(crate) static MATH_CTX: LazyLock<evalexpr::HashMapContext<evalexpr::DefaultNumericTypes>> =
    LazyLock::new(|| {
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
    });
