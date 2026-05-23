pub fn coerce_value(s: &str) -> serde_json::Value {
    if s.is_empty() {
        return serde_json::Value::Null;
    }
    if s == "true" {
        return serde_json::Value::Bool(true);
    }
    if s == "false" {
        return serde_json::Value::Bool(false);
    }
    if let Ok(n) = s.parse::<i64>() {
        return serde_json::Value::Number(n.into());
    }
    if let Ok(f) = s.parse::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return serde_json::Value::Number(n);
        }
    }
    serde_json::Value::String(s.to_string())
}

pub fn escape_cell(s: &str) -> String {
    s.replace('|', r"\|").replace('\n', "<br>").replace('\r', "")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn coerce_empty_is_null() {
        assert_eq!(coerce_value(""), Value::Null);
    }

    #[test]
    fn coerce_bool_true() {
        assert_eq!(coerce_value("true"), Value::Bool(true));
    }

    #[test]
    fn coerce_bool_false() {
        assert_eq!(coerce_value("false"), Value::Bool(false));
    }

    #[test]
    fn coerce_integer() {
        assert_eq!(coerce_value("42"), Value::Number(42.into()));
    }

    #[test]
    fn coerce_float() {
        let got = coerce_value("3.14");
        assert!(matches!(got, Value::Number(_)));
        if let Value::Number(n) = got {
            let f = n.as_f64().unwrap();
            assert!((f - 3.14).abs() < 1e-9);
        }
    }

    #[test]
    fn coerce_string_fallback() {
        assert_eq!(coerce_value("hello"), Value::String("hello".into()));
    }

    #[test]
    fn escape_pipe() {
        assert_eq!(escape_cell("a|b"), r"a\|b");
    }

    #[test]
    fn escape_newline_to_br() {
        assert_eq!(escape_cell("line1\nline2"), "line1<br>line2");
    }

    #[test]
    fn escape_crlf_becomes_br() {
        assert_eq!(escape_cell("a\r\nb"), "a<br>b");
    }

    #[test]
    fn escape_empty() {
        assert_eq!(escape_cell(""), "");
    }
}
