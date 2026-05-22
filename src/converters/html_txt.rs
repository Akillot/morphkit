use anyhow::Result;
use std::path::Path;

pub fn html_to_txt(input: &Path, output: &Path) -> Result<()> {
    let content = std::fs::read_to_string(input)?;
    let text = strip_tags(&content);
    std::fs::write(output, text)?;
    Ok(())
}

fn strip_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_skip = false; 
    let mut tag_buf = String::new();

    let mut chars = html.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '<' => {
                in_tag = true;
                tag_buf.clear();
            }
            '>' => {
                let tag = tag_buf.trim().to_lowercase();
                if tag == "script" || tag == "style" {
                    in_skip = true;
                } else if tag == "/script" || tag == "/style" {
                    in_skip = false;
                }
            
                if matches!(
                    tag.trim_start_matches('/'),
                    "p" | "div" | "br" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
                        | "li" | "tr" | "blockquote"
                ) {
                    out.push('\n');
                }
                in_tag = false;
                tag_buf.clear();
            }
            _ if in_tag => {
                
                if !tag_buf.contains(' ') && c != '/' || tag_buf.is_empty() {
                    tag_buf.push(c);
                }
            }
            _ if in_skip => {}
            _ => out.push(c),
        }
    }

    let cleaned: String = out
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n");

    let mut result = String::new();
    let mut blank_run = 0u32;
    for line in cleaned.lines() {
        if line.is_empty() {
            blank_run += 1;
            if blank_run <= 1 {
                result.push('\n');
            }
        } else {
            blank_run = 0;
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}