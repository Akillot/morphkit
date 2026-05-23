use anyhow::{bail, Result};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Format {
    Json,
    Csv,
    Markdown,
    Html,
    Txt,
    Pdf,
    Docx,
    Xlsx,
}

impl Format {
    pub fn ext(&self) -> &str {
        match self {
            Format::Json     => "json",
            Format::Csv      => "csv",
            Format::Markdown => "md",
            Format::Html     => "html",
            Format::Txt      => "txt",
            Format::Pdf      => "pdf",
            Format::Docx     => "docx",
            Format::Xlsx     => "xlsx",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Format::Json     => "JSON",
            Format::Csv      => "CSV",
            Format::Markdown => "Markdown",
            Format::Html     => "HTML",
            Format::Txt      => "TXT",
            Format::Pdf      => "PDF",
            Format::Docx     => "DOCX",
            Format::Xlsx     => "XLSX",
        }
    }
}

pub fn from_path(path: &Path) -> Result<Format> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    from_ext(ext)
}

pub fn from_ext(ext: &str) -> Result<Format> {
    match ext.to_lowercase().as_str() {
        "json"           => Ok(Format::Json),
        "csv"            => Ok(Format::Csv),
        "md" | "markdown"=> Ok(Format::Markdown),
        "html" | "htm"   => Ok(Format::Html),
        "txt" | "text"   => Ok(Format::Txt),
        "pdf"            => Ok(Format::Pdf),
        "docx"           => Ok(Format::Docx),
        "xlsx"           => Ok(Format::Xlsx),
        other => bail!("unsupported format: '{}'", other),
    }
}
