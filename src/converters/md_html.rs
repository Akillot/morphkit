use anyhow::Result;
use pulldown_cmark::{html, Options, Parser};
use std::path::Path;

pub fn md_to_html(input: &Path, output: &Path) -> Result<()> {
    let content = std::fs::read_to_string(input)?;

    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(&content, opts);

    let mut body = String::new();
    html::push_html(&mut body, parser);

    let title = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Document");

    let document = format!(
        "<!DOCTYPE html>\n\
         <html lang=\"en\">\n\
         <head>\n\
         <meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>{title}</title>\n\
         <style>\n\
           body {{ font-family: system-ui, sans-serif; max-width: 800px; margin: 2rem auto; padding: 0 1rem; line-height: 1.6; }}\n\
           pre {{ background: #f4f4f4; padding: 1rem; border-radius: 4px; overflow-x: auto; }}\n\
           code {{ font-family: monospace; }}\n\
         </style>\n\
         </head>\n\
         <body>\n\
         {body}\
         </body>\n\
         </html>\n"
    );

    std::fs::write(output, document)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static ID: AtomicU64 = AtomicU64::new(0);

    fn run(stem: &str, md: &str) -> String {
        let id = ID.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir();
        let inp = dir.join(format!("{}_{}.md", stem, id));
        let out = dir.join(format!("{}_{}.html", stem, id));
        std::fs::write(&inp, md).unwrap();
        md_to_html(&inp, &out).unwrap();
        let r = std::fs::read_to_string(&out).unwrap();
        let _ = std::fs::remove_file(&inp);
        let _ = std::fs::remove_file(&out);
        r
    }

    #[test]
    fn valid_html5_skeleton() {
        let html = run("doc", "# Hello\n");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html"));
        assert!(html.contains("</html>"));
        assert!(html.contains("<body>") || html.contains("<body\n"));
        assert!(html.contains("</body>"));
        assert!(html.contains("charset=\"utf-8\"") || html.contains("charset=utf-8"));
    }

    #[test]
    fn heading_rendered_as_h1() {
        let html = run("doc", "# My Heading\n");
        assert!(html.contains("<h1>My Heading</h1>"));
    }

    #[test]
    fn table_extension_enabled() {
        let md = "| a | b |\n| --- | --- |\n| 1 | 2 |\n";
        let html = run("doc", md);
        assert!(html.contains("<table"));
        assert!(html.contains("<td>"));
    }

    #[test]
    fn title_taken_from_filename_stem() {
        let id = ID.fetch_add(1, Ordering::SeqCst);
        let stem = format!("my_report_{}", id);
        let dir = std::env::temp_dir();
        let inp = dir.join(format!("{}.md", stem));
        let out = dir.join(format!("{}.html", stem));
        std::fs::write(&inp, "text").unwrap();
        md_to_html(&inp, &out).unwrap();
        let html = std::fs::read_to_string(&out).unwrap();
        let _ = std::fs::remove_file(&inp);
        let _ = std::fs::remove_file(&out);
        assert!(html.contains(&stem));
    }

    #[test]
    fn strikethrough_extension_enabled() {
        let html = run("doc", "~~deleted~~\n");
        assert!(html.contains("<del>deleted</del>"));
    }
}
