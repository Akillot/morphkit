use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

use crate::converters;
use crate::detect::Format;

pub fn run(input: &Path, from: Format, to: Format, output: Option<&str>) -> Result<()> {
    if from == to {
        bail!("input and output formats are the same ({})", from.label());
    }

    let out_path: PathBuf = match output {
        Some(p) => PathBuf::from(p),
        None => {
            let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
            PathBuf::from(format!("{}.{}", stem, to.ext()))
        }
    };

    eprintln!("  {} → {}", from.label(), to.label());

    match (&from, &to) {
        (Format::Json, Format::Csv) => converters::json_csv::json_to_csv(input, &out_path)?,
        (Format::Csv, Format::Json) => converters::json_csv::csv_to_json(input, &out_path)?,
        (Format::Json, Format::Markdown) => converters::json_md::json_to_md(input, &out_path)?,
        (Format::Csv, Format::Markdown) => converters::csv_md::csv_to_md(input, &out_path)?,
        (Format::Markdown, Format::Html) => converters::md_html::md_to_html(input, &out_path)?,
        (Format::Markdown, Format::Txt) => converters::txt_md::md_to_txt(input, &out_path)?,
        (Format::Html, Format::Txt) => converters::html_txt::html_to_txt(input, &out_path)?,
        (Format::Txt, Format::Markdown) => converters::txt_md::txt_to_md(input, &out_path)?,
        (Format::Markdown, Format::Pdf)
        | (Format::Html, Format::Pdf)
        | (Format::Txt, Format::Pdf) => converters::pdf::via_pandoc(input, &out_path)?,
        (f, t) => bail!(
            "conversion {} → {} is not supported yet",
            f.label(),
            t.label()
        ),
    }

    eprintln!("  done → {}", out_path.display());
    Ok(())
}
