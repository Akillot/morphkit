use anyhow::Result;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::path::Path;

pub fn txt_to_md(input: &Path, output: &Path) -> Result<()> {
    let content = std::fs::read_to_string(input)?;
    let mut md = String::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let next_trimmed = lines.get(i + 1).copied().unwrap_or("").trim();

        if !line.trim().is_empty() && next_trimmed.len() >= 2 {
            if next_trimmed.chars().all(|c| c == '=') {
                md.push_str(&format!("# {}\n\n", line.trim()));
                i += 2;
                continue;
            }
            if next_trimmed.chars().all(|c| c == '-') {
                md.push_str(&format!("## {}\n\n", line.trim()));
                i += 2;
                continue;
            }
        }

        if line.trim().is_empty() {
            md.push('\n');
        } else {
            md.push_str(line);
            md.push('\n');
        }
        i += 1;
    }

    let result = collapse_blank_lines(&md);
    std::fs::write(output, result)?;
    Ok(())
}

pub fn md_to_txt(input: &Path, output: &Path) -> Result<()> {
    let content = std::fs::read_to_string(input)?;
    let parser = Parser::new_ext(&content, Options::ENABLE_TABLES);

    let mut text = String::new();
    let mut list_stack: Vec<Option<u64>> = Vec::new();
    let mut item_counters: Vec<u64> = Vec::new();
    let mut pending_blank = false;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                flush_blank(&mut text, &mut pending_blank);
                let depth = heading_depth(level);
                for _ in 0..depth {
                    text.push('#');
                }
                text.push(' ');
            }
            Event::End(TagEnd::Heading(_)) => {
                text.push('\n');
                pending_blank = true;
            }
            Event::Start(Tag::Paragraph) => {
                flush_blank(&mut text, &mut pending_blank);
            }
            Event::End(TagEnd::Paragraph) => {
                text.push('\n');
                pending_blank = true;
            }
            Event::Start(Tag::List(start)) => {
                list_stack.push(start);
                item_counters.push(start.unwrap_or(0));
            }
            Event::End(TagEnd::List(_)) => {
                list_stack.pop();
                item_counters.pop();
                if list_stack.is_empty() {
                    pending_blank = true;
                }
            }
            Event::Start(Tag::Item) => {
                flush_blank(&mut text, &mut pending_blank);
                let indent = "  ".repeat(list_stack.len().saturating_sub(1));
                if let Some(Some(_)) = list_stack.last().copied() {
                    let n = item_counters.last_mut().unwrap();
                    text.push_str(&format!("{}{}. ", indent, n));
                    *n += 1;
                } else {
                    text.push_str(&format!("{}- ", indent));
                }
            }
            Event::End(TagEnd::Item) => {
                text.push('\n');
            }
            Event::Start(Tag::CodeBlock(_)) => {
                flush_blank(&mut text, &mut pending_blank);
            }
            Event::End(TagEnd::CodeBlock) => {
                text.push('\n');
                pending_blank = true;
            }
            Event::Text(t) => text.push_str(&t),
            Event::Code(t) => text.push_str(&t),
            Event::SoftBreak => text.push(' '),
            Event::HardBreak => text.push('\n'),
            Event::End(TagEnd::BlockQuote) => {
                pending_blank = true;
            }
            _ => {}
        }
    }

    std::fs::write(output, text.trim().to_string() + "\n")?;
    Ok(())
}

fn flush_blank(text: &mut String, pending_blank: &mut bool) {
    if *pending_blank && !text.is_empty() {
        text.push('\n');
        *pending_blank = false;
    }
}

fn heading_depth(level: HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn collapse_blank_lines(s: &str) -> String {
    let mut result = String::new();
    let mut blank_run = 0u32;
    for line in s.lines() {
        if line.trim().is_empty() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::HeadingLevel;
    use std::sync::atomic::{AtomicU64, Ordering};

    static ID: AtomicU64 = AtomicU64::new(0);

    fn run_t2m(txt: &str) -> String {
        let id = ID.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir();
        let inp = dir.join(format!("conkit_t2m_in_{}.txt", id));
        let out = dir.join(format!("conkit_t2m_out_{}.md", id));
        std::fs::write(&inp, txt).unwrap();
        txt_to_md(&inp, &out).unwrap();
        let r = std::fs::read_to_string(&out).unwrap();
        let _ = std::fs::remove_file(&inp);
        let _ = std::fs::remove_file(&out);
        r
    }

    fn run_m2t(md: &str) -> String {
        let id = ID.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir();
        let inp = dir.join(format!("conkit_m2t_in_{}.md", id));
        let out = dir.join(format!("conkit_m2t_out_{}.txt", id));
        std::fs::write(&inp, md).unwrap();
        md_to_txt(&inp, &out).unwrap();
        let r = std::fs::read_to_string(&out).unwrap();
        let _ = std::fs::remove_file(&inp);
        let _ = std::fs::remove_file(&out);
        r
    }

    #[test]
    fn collapse_limits_to_one_blank() {
        let out = collapse_blank_lines("a\n\n\n\nb");
        assert!(!out.contains("\n\n\n"));
        assert!(out.contains("a"));
        assert!(out.contains("b"));
    }

    #[test]
    fn collapse_preserves_single_blank() {
        let out = collapse_blank_lines("a\n\nb");
        assert!(out.contains("\n\n"));
    }

    #[test]
    fn heading_depth_h1() {
        assert_eq!(heading_depth(HeadingLevel::H1), 1);
    }

    #[test]
    fn heading_depth_h6() {
        assert_eq!(heading_depth(HeadingLevel::H6), 6);
    }

    #[test]
    fn setext_h1_converted() {
        let out = run_t2m("My Title\n========\n\nSome text.\n");
        assert!(out.starts_with("# My Title"));
        assert!(out.contains("Some text."));
    }

    #[test]
    fn setext_h2_converted() {
        let out = run_t2m("Section\n-------\n\nParagraph.\n");
        assert!(out.starts_with("## Section"));
        assert!(out.contains("Paragraph."));
    }

    #[test]
    fn plain_text_passes_through() {
        let out = run_t2m("Just a plain sentence.\n");
        assert!(out.contains("Just a plain sentence."));
        assert!(!out.starts_with('#'));
    }

    #[test]
    fn multiple_blank_lines_collapsed() {
        let out = run_t2m("a\n\n\n\nb\n");
        assert!(!out.contains("\n\n\n"));
    }

    #[test]
    fn heading_gets_hash_prefix() {
        let out = run_m2t("# Title\n\nParagraph.\n");
        assert!(out.contains("# Title"));
        assert!(out.contains("Paragraph."));
    }

    #[test]
    fn h2_gets_two_hashes() {
        let out = run_m2t("## Subtitle\n");
        assert!(out.contains("## Subtitle"));
    }

    #[test]
    fn unordered_list_markers_preserved() {
        let out = run_m2t("- alpha\n- beta\n- gamma\n");
        assert!(out.contains("- alpha"));
        assert!(out.contains("- beta"));
        assert!(out.contains("- gamma"));
    }

    #[test]
    fn ordered_list_markers_preserved() {
        let out = run_m2t("1. first\n2. second\n3. third\n");
        assert!(out.contains("1. first"));
        assert!(out.contains("2. second"));
        assert!(out.contains("3. third"));
    }

    #[test]
    fn nested_list_indented() {
        let out = run_m2t("- top\n  - nested\n");
        assert!(out.contains("- top"));
        assert!(out.contains("  - nested"));
    }
}
