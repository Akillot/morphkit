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
