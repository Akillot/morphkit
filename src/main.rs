use anyhow::{bail, Context, Result};
use clap::Parser;
use std::path::Path;

mod converters;
mod detect;
mod pipeline;

#[derive(Parser)]
#[command(name = "conkit")]
#[command(version)]
#[command(about = "Universal file converter — zero bullshit setup")]
#[command(override_usage = "conkit <file> to <format> [-o <output>]")]
struct Args {
    input: String,
    to: String,
    format: String,
    #[arg(short, long, value_name = "file")]
    output: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.to != "to" {
        bail!("usage: conkit <file> to <format>\nexample: conkit data.json to csv");
    }

    let input_path = Path::new(&args.input);
    if !input_path.exists() {
        bail!("file not found: {}", args.input);
    }

    let from = detect::from_path(input_path)
        .with_context(|| format!("cannot detect format of '{}'", args.input))?;

    let to = detect::from_ext(&args.format)
        .with_context(|| format!("unknown output format: '{}'", args.format))?;

    pipeline::run(input_path, from, to, args.output.as_deref())
}
