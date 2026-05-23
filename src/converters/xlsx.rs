use anyhow::{Context, Result};
use calamine::{open_workbook, Data, Range, Reader, Xlsx};
use std::path::Path;

use super::util::{coerce_value, escape_cell};

fn open_first_sheet(path: &Path) -> Result<Range<Data>> {
    let mut wb: Xlsx<_> = open_workbook(path).context("cannot open xlsx")?;
    let name = wb
        .sheet_names()
        .into_iter()
        .next()
        .context("workbook has no sheets")?;
    wb.worksheet_range(&name).context("cannot read sheet")
}

fn cell_str(cell: &Data) -> String {
    match cell {
        Data::Int(i) => i.to_string(),
        Data::Float(f) => f.to_string(),
        Data::String(s) => s.clone(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(f) => f.to_string(),
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(_) | Data::Empty => String::new(),
    }
}

pub fn xlsx_to_csv(input: &Path, output: &Path) -> Result<()> {
    let range = open_first_sheet(input)?;
    let mut rows = range.rows();
    let headers: Vec<String> = match rows.next() {
        Some(r) => r.iter().map(cell_str).collect(),
        None => {
            std::fs::write(output, "")?;
            return Ok(());
        }
    };
    let mut wtr = csv::Writer::from_path(output)?;
    wtr.write_record(&headers)?;
    for row in rows {
        wtr.write_record(row.iter().map(cell_str).collect::<Vec<_>>())?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn xlsx_to_json(input: &Path, output: &Path) -> Result<()> {
    let range = open_first_sheet(input)?;
    let mut rows = range.rows();
    let headers: Vec<String> = match rows.next() {
        Some(r) => r.iter().map(cell_str).collect(),
        None => {
            std::fs::write(output, "[]")?;
            return Ok(());
        }
    };
    let records: Vec<serde_json::Value> = rows
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (i, cell) in row.iter().enumerate() {
                if let Some(h) = headers.get(i) {
                    obj.insert(h.clone(), coerce_value(&cell_str(cell)));
                }
            }
            serde_json::Value::Object(obj)
        })
        .collect();
    std::fs::write(output, serde_json::to_string_pretty(&records)?)?;
    Ok(())
}

pub fn xlsx_to_md(input: &Path, output: &Path) -> Result<()> {
    let range = open_first_sheet(input)?;
    let mut rows = range.rows();
    let headers: Vec<String> = match rows.next() {
        Some(r) => r.iter().map(cell_str).collect(),
        None => {
            std::fs::write(output, "")?;
            return Ok(());
        }
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
        md.push('|');
        for cell in row.iter() {
            md.push_str(&format!(" {} |", escape_cell(&cell_str(cell))));
        }
        md.push('\n');
    }
    std::fs::write(output, md)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::Data;

    #[test]
    fn cell_str_int() {
        assert_eq!(cell_str(&Data::Int(42)), "42");
    }

    #[test]
    fn cell_str_float() {
        assert_eq!(cell_str(&Data::Float(1.5)), "1.5");
    }

    #[test]
    fn cell_str_string() {
        assert_eq!(cell_str(&Data::String("hello".into())), "hello");
    }

    #[test]
    fn cell_str_bool() {
        assert_eq!(cell_str(&Data::Bool(true)), "true");
        assert_eq!(cell_str(&Data::Bool(false)), "false");
    }

    #[test]
    fn cell_str_empty() {
        assert_eq!(cell_str(&Data::Empty), "");
    }
}
