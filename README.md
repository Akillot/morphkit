# morphkit

Zero-config file format converter for the terminal.

```
$ morphkit data.json csv
  ✓  data.csv

$ morphkit report.md html
  ✓  report.html

$ morphkit contract.docx pdf
  ✓  contract.pdf
```

## Formats

|          | json | csv | md | html | txt | pdf |
| -------- | :--: | :-: | :-: | :--: | :-: | :-: |
| **json** |  —   |  ✓  |  ✓  |      |     |     |
| **csv**  |  ✓   |  —  |  ✓  |      |     |     |
| **md**   |      |     |  —  |  ✓   |  ✓  |  ✓  |
| **html** |      |     |     |  —   |  ✓  |  ✓  |
| **txt**  |      |     |  ✓  |      |  —  |  ✓  |
| **docx** |      |     |  ✓  |  ✓   |  ✓  |  ✓  |

## Install

Requires Rust. Clone and build with Cargo.

```
git clone https://github.com/your-username/morphkit
cd morphkit
cargo build --release
```

The binary lands at `target/release/morphkit`. PDF and DOCX conversions require Pandoc — see https://pandoc.org/installing.html.

## Usage

```
morphkit <file> <format>
morphkit <file> <format> -o <output>
```

Output is named after the input with the new extension. Pass `-o` to override.

## AI Bootstrap Prompt

> Copy and paste into Claude, Cursor, Codex, or GPT:

```text
You are working on morphkit — a command-line file format converter written in Rust.

Stack: Rust 2021 edition, clap (derive), anyhow, serde_json (preserve_order),
       csv, pulldown-cmark, indicatif
Entry point: src/main.rs
Format detection: src/detect.rs
Conversion dispatch: src/pipeline.rs
Converters: src/converters/ (one file per format pair)
Run: cargo run -- <input-file> <output-format>
Build: cargo build --release → target/release/morphkit

CLI: morphkit <file> <format> [-o <output>]
Format is inferred from the input file extension.
Output format is a plain string: json, csv, md, html, txt, pdf, docx.

UX: pipeline.rs shows an indicatif spinner while converting, then prints
    "  ✓  <output-file>" on success using ANSI green (\x1b[32m).

Converters:
- html_txt.rs — slice-based scanner, no regex. Skips <script>/<style> by
  searching for closing tags literally. Decodes HTML entities. Handles
  <!-- comments and <!DOCTYPE>.
- csv_md.rs, json_md.rs — Markdown table output. Pipe and newline characters
  in cells are escaped (| → \|, newline → <br>).
- json_csv.rs — bidirectional. JSON→CSV preserves key insertion order via
  preserve_order. CSV→JSON coerces field types (int, float, bool, null).
- txt_md.rs — txt_to_md detects setext headings (=== and ---).
  md_to_txt uses pulldown-cmark events to preserve heading prefixes
  and list markers with nesting via indentation.
- md_html.rs — pulldown-cmark HTML output wrapped in a full HTML5 document
  with inline CSS. Strikethrough, tables, footnotes, and task lists enabled.
- pdf.rs — delegates to Pandoc via std::process::Command. Handles PDF output
  from Markdown, HTML, TXT, and all DOCX conversions (Pandoc infers formats
  from file extensions).

Non-obvious:
- serde_json must have features = ["preserve_order"] in Cargo.toml — without
  it JSON object keys sort alphabetically and CSV column order breaks.
- pdf.rs is format-agnostic: it just calls pandoc <input> -o <output> and
  lets Pandoc resolve formats from extensions — it covers docx→pdf,
  docx→md, docx→html, docx→txt as well.
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
