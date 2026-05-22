use anyhow::Result;
use pulldown_cmark::{Event, Options, Parser, TagEnd};
use std::path::Path;

pub fn txt_to_md(input: &Path, output: &Path) -> Result<()> {
    let content = std::fs::read_to_string(input)?;

    let md = content
        .split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");

    std::fs::write(output, md + "\n")?;
    Ok(())
}

pub fn md_to_txt(input: &Path, output: &Path) -> Result<()> {
    let content = std::fs::read_to_string(input)?;
    let parser = Parser::new_ext(&content, Options::empty());

    let mut text = String::new();
    let mut pending_newline = false;

    for event in parser {
        match event {
            Event::Text(t) => {
                if pending_newline && !text.is_empty() {
                    text.push('\n');
                    pending_newline = false;
                }
                text.push_str(&t);
            }
            Event::Code(t) => text.push_str(&t),
            Event::SoftBreak => text.push(' '),
            Event::HardBreak => text.push('\n'),
            Event::End(
                TagEnd::Paragraph
                | TagEnd::Heading(_)
                | TagEnd::Item
                | TagEnd::BlockQuote
                | TagEnd::CodeBlock,
            ) => pending_newline = true,
            _ => {}
        }
    }

    std::fs::write(output, text.trim().to_string() + "\n")?;
    Ok(())
}
