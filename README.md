# morphkit Overview

**morphkit** is a command-line file converter written in Rust. It converts between JSON, CSV, Markdown, HTML, plain text, and PDF with a single command and no configuration.

## Supported Conversions

| From | To |
| --- | --- |
| JSON | CSV, Markdown |
| CSV | JSON, Markdown |
| Markdown | HTML, PDF, plain text |
| HTML | plain text, PDF |
| Plain text | Markdown, PDF |

## Installation

Requires Rust. Clone the repository and build with Cargo.

```
git clone https://github.com/your-username/morphkit
cd morphkit
cargo build --release
```

The binary will be at `target/release/morphkit`. PDF output requires Pandoc — see https://pandoc.org/installing.html.

## Usage

```
morphkit <file> <format>
morphkit data.json csv
morphkit report.md html
morphkit table.csv json -o output.json
```

The output file is named after the input with the new extension. Use `-o` to specify a custom path.

## Implementation Notes

The HTML-to-text converter scans the source string with a slice-based state machine rather than a regex pass. It handles comments, `<!DOCTYPE>`, and `<script>`/`<style>` blocks correctly by searching for closing tags literally rather than tokenizing tag by tag. HTML entities are decoded before output.

JSON-to-CSV and JSON-to-Markdown preserve the original key order from the source file via `serde_json`'s `preserve_order` feature. CSV fields containing pipe characters or newlines are escaped before rendering into Markdown tables.

PDF conversion delegates to Pandoc rather than implementing a renderer from scratch.

## AI Bootstrap Prompt

> Copy and paste into Claude, Cursor, Codex, or GPT:

```text
You are working on morphkit — a command-line file converter written in Rust.

Stack: Rust 2021 edition, clap (derive), anyhow, serde_json (preserve_order), csv, pulldown-cmark
Entry point: src/main.rs
Format detection: src/detect.rs
Conversion dispatch: src/pipeline.rs
Converters: src/converters/ (one file per format pair)
Run: cargo run -- <input-file> <output-format>
Build: cargo build --release → target/release/morphkit

CLI: morphkit <file> <format> [-o <output>]
Format is inferred from the input file extension.
Output format is a plain string: json, csv, md, html, txt, pdf.

Converters:
- html_txt.rs — slice-based scanner, no regex. Skips <script>/<style> by
  searching for closing tags literally. Decodes HTML entities. Handles
  <!-- comments and <!DOCTYPE>.
- csv_md.rs, json_md.rs — Markdown table output. Pipe and newline characters
  in cells are escaped (| → \|, newline → <br>).
- json_csv.rs — bidirectional. JSON→CSV preserves key insertion order via
  preserve_order. CSV→JSON coerces field types (int, float, bool, null).
- txt_md.rs — txt_to_md detects setext headings (=== and ---).
  md_to_txt uses pulldown-cmark events to preserve heading prefixes (#, ##, …)
  and list markers (- for unordered, N. for ordered) with nesting via indentation.
- md_html.rs — pulldown-cmark HTML output wrapped in a full HTML5 document
  with inline CSS. Strikethrough, tables, footnotes, and task lists enabled.
- pdf.rs — delegates to Pandoc via std::process::Command.

Non-obvious:
- serde_json must have features = ["preserve_order"] in Cargo.toml — without it
  JSON object keys sort alphabetically and CSV column order breaks.
- pdf.rs has no fallback: if pandoc is not in PATH, the error message includes
  the install URL https://pandoc.org/installing.html.
- Every converter module has a #[cfg(test)] block. Internal helpers are tested
  directly (same module = access to private fns). File-level functions use a
  temp-file runner with an AtomicU64 counter for unique filenames across
  parallel tests.

Known gaps: csv_to_json loads the entire file into memory; no streaming.
All paths assume UTF-8 — Windows-1252 CSV from Excel will fail.
No HTML→Markdown or CSV→HTML conversion paths exist yet.
```

## License

MIT
