# morphkit

Zero-config converter: docs and data to LLM-ready Markdown

```
$ morph data.json csv
  ✓  data.csv

$ morph report.md html
  ✓  report.html

$ morph contract.docx pdf
  ✓  contract.pdf
```

## Formats

|           | json | csv | md | html | txt | pdf |
| --------- | :--: | :-: | :-: | :--: | :-: | :-: |
| **json**  |  —   |  ✓  |  ✓  |      |     |     |
| **csv**   |  ✓   |  —  |  ✓  |      |     |     |
| **md**    |      |     |  —  |  ✓   |  ✓  |  ✓  |
| **html**  |      |     |     |  —   |  ✓  |  ✓  |
| **txt**   |      |     |  ✓  |      |  —  |  ✓  |
| **docx**  |      |     |  ✓  |  ✓   |  ✓  |  ✓  |
| **xlsx**  |  ✓   |  ✓  |  ✓  |      |     |     |

## Install

Requires Rust. Clone and build with Cargo.

```
git clone https://github.com/your-username/morphkit
cd morphkit
cargo build --release
```

The binary lands at `target/release/morph`. PDF and DOCX conversions require Pandoc — see https://pandoc.org/installing.html.

## Usage

```
morph <file> <format>
morph <file> <format> -o <output>
```

Output is named after the input with the new extension. Pass `-o` to override.

## License

MIT
