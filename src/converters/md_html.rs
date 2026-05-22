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
