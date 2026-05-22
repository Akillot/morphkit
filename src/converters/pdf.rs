use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};

pub fn via_pandoc(input: &Path, output: &Path) -> Result<()> {
    let mut cmd = Command::new("pandoc");
    cmd.arg(input).arg("-o").arg(output);

    if output.extension().and_then(|e| e.to_str()) == Some("pdf") {
        if let Some(engine) = detect_pdf_engine() {
            cmd.arg("--pdf-engine").arg(engine);
        } else {
            bail!(
                "no PDF engine found — install tectonic (recommended):\n  \
                 brew install tectonic\n  \
                 or BasicTeX: brew install --cask basictex\n  \
                 or see: https://pandoc.org/MANUAL.html#creating-a-pdf"
            );
        }
    }

    let status = cmd
        .status()
        .context("pandoc not found — install: https://pandoc.org/installing.html")?;

    if !status.success() {
        bail!("pandoc exited with error");
    }
    Ok(())
}

fn detect_pdf_engine() -> Option<&'static str> {
    ["tectonic", "xelatex", "lualatex", "pdflatex", "wkhtmltopdf"]
        .iter()
        .copied()
        .find(|&engine| {
            Command::new(engine)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok()
        })
}
