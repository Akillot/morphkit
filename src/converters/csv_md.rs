use anyhow::Result;
use std::path::Path;

pub fn csv_to_md(input: &Path, output: &Path) -> Result<()> {
    let mut rdr = csv::Reader::from_path(input)?;
    let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();

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

    for result in rdr.records() {
        let record = result?;
        md.push('|');
        for field in record.iter() {
            md.push_str(&format!(" {} |", field));
        }
        md.push('\n');
    }

    std::fs::write(output, md)?;
    Ok(())
}
