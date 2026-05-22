use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

pub fn via_pandoc(input: &Path, output: &Path) -> Result<()> {
    let status = Command::new("pandoc")
        .arg(input)
        .arg("-o")
        .arg(output)
        .status()
        .context("pandoc not found — install: https://pandoc.org/installing.html")?;

    if !status.success() {
        bail!("pandoc exited with error");
    }
    Ok(())
}
