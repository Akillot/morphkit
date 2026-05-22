use anyhow::Result;
use std::path::Path;

pub fn csv_to_md(input: &Path, output: &Path) -> Result<()> {
    let mut rdr = csv::Reader::from_path(input)?;
    let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();

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

    for result in rdr.records() {
        let record = result?;
        md.push('|');
        for field in record.iter() {
            md.push_str(&format!(" {} |", escape_cell(field)));
        }
        md.push('\n');
    }

    std::fs::write(output, md)?;
    Ok(())
}

fn escape_cell(s: &str) -> String {
    s.replace('|', r"\|").replace('\n', "<br>").replace('\r', "")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static ID: AtomicU64 = AtomicU64::new(0);

    fn run(csv: &str) -> String {
        let id = ID.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir();
        let inp = dir.join(format!("conkit_csvin_{}.csv", id));
        let out = dir.join(format!("conkit_csvout_{}.md", id));
        std::fs::write(&inp, csv).unwrap();
        csv_to_md(&inp, &out).unwrap();
        let r = std::fs::read_to_string(&out).unwrap();
        let _ = std::fs::remove_file(&inp);
        let _ = std::fs::remove_file(&out);
        r
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
    fn basic_table_structure() {
        let md = run("name,age\nAlice,30\nBob,25\n");
        assert!(md.contains("| name | age |"));
        assert!(md.contains("| --- |"));
        assert!(md.contains("| Alice | 30 |"));
        assert!(md.contains("| Bob | 25 |"));
    }

    #[test]
    fn pipe_in_cell_escaped() {
        let md = run("a,b\nfoo|bar,baz\n");
        assert!(md.contains(r"foo\|bar"));
    }

    #[test]
    fn single_column() {
        let md = run("word\nhello\nworld\n");
        assert!(md.contains("| word |"));
        assert!(md.contains("| hello |"));
    }
}
