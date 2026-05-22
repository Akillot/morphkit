use anyhow::{bail, Context, Result};
use std::path::Path;

pub fn json_to_csv(input: &Path, output: &Path) -> Result<()> {
    let content = std::fs::read_to_string(input)?;
    let value: serde_json::Value = serde_json::from_str(&content).context("invalid JSON")?;

    let rows = match &value {
        serde_json::Value::Array(arr) => arr,
        _ => bail!("JSON must be an array of objects for CSV conversion"),
    };

    if rows.is_empty() {
        std::fs::write(output, "")?;
        return Ok(());
    }

    let headers: Vec<String> = match &rows[0] {
        serde_json::Value::Object(obj) => obj.keys().cloned().collect(),
        _ => bail!("JSON array elements must be objects"),
    };

    let mut wtr = csv::Writer::from_path(output)?;
    wtr.write_record(&headers)?;

    for (i, row) in rows.iter().enumerate() {
        let obj = match row {
            serde_json::Value::Object(obj) => obj,
            _ => bail!("element at index {} is not an object", i),
        };
        let record: Vec<String> = headers
            .iter()
            .map(|h| {
                obj.get(h)
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Null => String::new(),
                        other => other.to_string(),
                    })
                    .unwrap_or_default()
            })
            .collect();
        wtr.write_record(&record)?;
    }

    wtr.flush()?;
    Ok(())
}

pub fn csv_to_json(input: &Path, output: &Path) -> Result<()> {
    let mut rdr = csv::Reader::from_path(input)?;
    let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();

    let mut records = Vec::new();
    for result in rdr.records() {
        let record = result?;
        let mut obj = serde_json::Map::new();
        for (i, field) in record.iter().enumerate() {
            if let Some(header) = headers.get(i) {
                obj.insert(header.clone(), coerce_value(field));
            }
        }
        records.push(serde_json::Value::Object(obj));
    }

    let json = serde_json::to_string_pretty(&records)?;
    std::fs::write(output, json)?;
    Ok(())
}

fn coerce_value(s: &str) -> serde_json::Value {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::sync::atomic::{AtomicU64, Ordering};

    static ID: AtomicU64 = AtomicU64::new(0);

    fn run_j2c(json: &str) -> String {
        let id = ID.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir();
        let inp = dir.join(format!("morphkit_j2c_in_{}.json", id));
        let out = dir.join(format!("morphkit_j2c_out_{}.csv", id));
        std::fs::write(&inp, json).unwrap();
        json_to_csv(&inp, &out).unwrap();
        let r = std::fs::read_to_string(&out).unwrap();
        let _ = std::fs::remove_file(&inp);
        let _ = std::fs::remove_file(&out);
        r
    }

    fn run_c2j(csv: &str) -> Vec<Value> {
        let id = ID.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir();
        let inp = dir.join(format!("morphkit_c2j_in_{}.csv", id));
        let out = dir.join(format!("morphkit_c2j_out_{}.json", id));
        std::fs::write(&inp, csv).unwrap();
        csv_to_json(&inp, &out).unwrap();
        let s = std::fs::read_to_string(&out).unwrap();
        let _ = std::fs::remove_file(&inp);
        let _ = std::fs::remove_file(&out);
        serde_json::from_str(&s).unwrap()
    }

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
    fn j2c_basic_column_order_preserved() {
        let json = r#"[{"name":"Alice","age":30},{"name":"Bob","age":25}]"#;
        let csv = run_j2c(json);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[0], "name,age");
        assert_eq!(lines[1], "Alice,30");
        assert_eq!(lines[2], "Bob,25");
    }

    #[test]
    fn j2c_empty_array_produces_empty_file() {
        assert_eq!(run_j2c("[]").trim(), "");
    }

    #[test]
    fn j2c_null_field_is_empty_csv_cell() {
        let json = r#"[{"a":null,"b":"x"}]"#;
        let csv = run_j2c(json);
        assert!(csv.contains(",x") || csv.contains("x,"));
        let data_line = csv.lines().nth(1).unwrap();
        assert!(data_line.starts_with(',') || data_line.ends_with(','));
    }

    #[test]
    fn c2j_type_coercion() {
        let rows = run_c2j("name,score,active\nAlice,95,true\n");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["name"],   Value::String("Alice".into()));
        assert_eq!(rows[0]["score"],  Value::Number(95.into()));
        assert_eq!(rows[0]["active"], Value::Bool(true));
    }

    #[test]
    fn c2j_empty_field_is_null() {
        let rows = run_c2j("a,b\n1,\n");
        assert_eq!(rows[0]["b"], Value::Null);
    }

    #[test]
    fn c2j_multiple_rows() {
        let rows = run_c2j("x\n1\n2\n3\n");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[2]["x"], Value::Number(3.into()));
    }
}
