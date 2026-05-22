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
    let mut rest = html;

    while !rest.is_empty() {
        match rest.find('<') {
            None => {
                out.push_str(&decode_entities(rest));
                break;
            }
            Some(tag_start) => {
                out.push_str(&decode_entities(&rest[..tag_start]));
                rest = &rest[tag_start..];

                if rest.starts_with("<!--") {
                    let after = &rest[4..];
                    if let Some(end) = after.find("-->") {
                        rest = &after[end + 3..];
                    } else {
                        break;
                    }
                    continue;
                }

                if rest.starts_with("<!") {
                    if let Some(end) = rest.find('>') {
                        rest = &rest[end + 1..];
                    } else {
                        break;
                    }
                    continue;
                }

                let Some(tag_end) = rest.find('>') else {
                    break;
                };
                let tag_content = &rest[1..tag_end];
                let tag_lower = tag_content.trim().to_ascii_lowercase();
                let is_closing = tag_lower.starts_with('/');
                let tag_name = tag_lower
                    .trim_start_matches('/')
                    .split(|c: char| c.is_ascii_whitespace() || c == '/')
                    .next()
                    .unwrap_or("");

                if !is_closing && (tag_name == "script" || tag_name == "style") {
                    let close_tag = format!("</{}>", tag_name);
                    let after = &rest[tag_end + 1..];
                    let lower_after = after.to_ascii_lowercase();
                    if let Some(pos) = lower_after.find(&close_tag) {
                        if let Some(gt) = after[pos..].find('>') {
                            rest = &after[pos + gt + 1..];
                        } else {
                            rest = &after[after.len()..];
                        }
                    } else {
                        rest = &after[after.len()..];
                    }
                    continue;
                }

                if tag_name == "br"
                    || (is_closing
                        && matches!(
                            tag_name,
                            "p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
                                | "li" | "tr" | "blockquote"
                        ))
                {
                    out.push('\n');
                }

                rest = &rest[tag_end + 1..];
            }
        }
    }

    collapse_whitespace(out)
}

fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn collapse_whitespace(s: String) -> String {
    let mut result = String::new();
    let mut blank_run = 0u32;
    for line in s.lines().map(str::trim) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_tag_stripped() {
        assert_eq!(strip_tags("<p>Hello</p>"), "Hello\n");
    }

    #[test]
    fn block_elements_add_newlines() {
        let out = strip_tags("<h1>Title</h1><p>Body</p>");
        let title_pos = out.find("Title").unwrap();
        let body_pos = out.find("Body").unwrap();
        assert!(title_pos < body_pos);
        assert!(out[title_pos..body_pos].contains('\n'));
    }

    #[test]
    fn script_content_skipped() {
        let html = "<p>before</p><script>if (a < b) { alert(); }</script><p>after</p>";
        let out = strip_tags(html);
        assert!(out.contains("before"));
        assert!(out.contains("after"));
        assert!(!out.contains("alert"));
        assert!(!out.contains("if (a"));
    }

    #[test]
    fn style_content_skipped() {
        let html = "<style>body { color: red > green; }</style><p>text</p>";
        let out = strip_tags(html);
        assert!(!out.contains("color"));
        assert!(out.contains("text"));
    }

    #[test]
    fn comment_with_gt_removed() {
        let html = "<!-- a > b --><p>visible</p>";
        let out = strip_tags(html);
        assert!(!out.contains("a > b"));
        assert!(out.contains("visible"));
    }

    #[test]
    fn doctype_removed() {
        let html = "<!DOCTYPE html><p>content</p>";
        let out = strip_tags(html);
        assert!(!out.to_lowercase().contains("doctype"));
        assert!(out.contains("content"));
    }

    #[test]
    fn entities_decoded() {
        assert_eq!(decode_entities("&amp;&lt;&gt;&quot;&nbsp;"), "&<>\" ");
    }

    #[test]
    fn entities_decoded_in_content() {
        let out = strip_tags("<p>a &amp; b &lt; c</p>");
        assert!(out.contains("a & b < c"));
    }

    #[test]
    fn consecutive_blanks_collapsed() {
        let out = collapse_whitespace("a\n\n\n\nb".to_string());
        assert!(!out.contains("\n\n\n"));
    }

    #[test]
    fn self_closing_br() {
        let out = strip_tags("line1<br/>line2");
        assert!(out.contains("line1"));
        assert!(out.contains("line2"));
    }
}
