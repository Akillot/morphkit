use anyhow::{bail, Context, Result};
use std::path::Path;

use super::util::escape_cell;

pub fn json_to_md(input: &Path, output: &Path) -> Result<()> {
    let content = std::fs::read_to_string(input)?;
    let value: serde_json::Value = serde_json::from_str(&content).context("invalid JSON")?;

    let md = match &value {
        serde_json::Value::Array(rows) if !rows.is_empty() => {
            match &rows[0] {
                serde_json::Value::Object(_) => array_of_objects_to_table(rows)?,
                _ => bail!("JSON array must contain objects for table conversion"),
            }
        }
        serde_json::Value::Array(_) => String::from("*(empty array)*\n"),
        _ => bail!("JSON must be an array of objects"),
    };

    std::fs::write(output, md)?;
    Ok(())
}

fn array_of_objects_to_table(rows: &[serde_json::Value]) -> Result<String> {
    let headers: Vec<String> = match &rows[0] {
        serde_json::Value::Object(obj) => obj.keys().cloned().collect(),
        _ => unreachable!(),
    };

    let mut md = String::new();

    md.push('|');
    for h in &headers {
        md.push_str(&format!(" {} |", escape_cell(h)));
    }
    md.push('\n');

    md.push('|');
    for _ in &headers {
        md.push_str(" --- |");
    }
    md.push('\n');

    for row in rows {
        let obj = match row {
            serde_json::Value::Object(o) => o,
            _ => bail!("all array elements must be objects"),
        };
        md.push('|');
        for h in &headers {
            let cell = obj
                .get(h)
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Null => String::new(),
                    other => other.to_string(),
                })
                .unwrap_or_default();
            md.push_str(&format!(" {} |", escape_cell(&cell)));
        }
        md.push('\n');
    }

    Ok(md)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Map, Value};

    fn obj(pairs: &[(&str, Value)]) -> Value {
        let mut m = Map::new();
        for (k, v) in pairs {
            m.insert(k.to_string(), v.clone());
        }
        Value::Object(m)
    }

    #[test]
    fn basic_table() {
        let rows = vec![
            obj(&[("name", Value::String("Alice".into())), ("age", Value::Number(30.into()))]),
            obj(&[("name", Value::String("Bob".into())),   ("age", Value::Number(25.into()))]),
        ];
        let md = array_of_objects_to_table(&rows).unwrap();
        assert!(md.contains("| name | age |"));
        assert!(md.contains("| Alice | 30 |"));
        assert!(md.contains("| Bob | 25 |"));
    }

    #[test]
    fn null_value_renders_empty_cell() {
        let rows = vec![obj(&[("x", Value::Null)])];
        let md = array_of_objects_to_table(&rows).unwrap();
        assert!(md.contains("|  |"));
    }

    #[test]
    fn missing_key_renders_empty_cell() {
        let rows = vec![
            obj(&[("a", Value::String("1".into())), ("b", Value::String("2".into()))]),
            obj(&[("a", Value::String("3".into()))]),
        ];
        let md = array_of_objects_to_table(&rows).unwrap();
        let data_lines: Vec<&str> = md.lines().skip(2).collect();
        assert!(data_lines[1].contains("|  |") || data_lines[1].ends_with("| |"));
    }

    #[test]
    fn pipe_in_value_escaped() {
        let rows = vec![obj(&[("v", Value::String("foo|bar".into()))])];
        let md = array_of_objects_to_table(&rows).unwrap();
        assert!(md.contains(r"foo\|bar"));
    }

}
