use anyhow::{bail, Context, Result};
use std::path::Path;

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
        md.push_str(&format!(" {} |", h));
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
            md.push_str(&format!(" {} |", cell));
        }
        md.push('\n');
    }

    Ok(md)
}
