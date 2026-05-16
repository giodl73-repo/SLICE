use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::Value;

#[derive(Debug, Parser)]
#[command(name = "slice")]
#[command(about = "Evaluate SLICE expressions over structured artifacts.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Evaluate an expression over JSON or newline-delimited JSON input.
    Eval {
        /// SLICE expression, for example: metadata.tags has 'context'
        #[arg(long)]
        expr: String,
        /// JSON or JSONL input file.
        #[arg(long)]
        input: PathBuf,
        /// Treat the input as newline-delimited JSON.
        #[arg(long)]
        jsonl: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Eval { expr, input, jsonl } => run_eval(&expr, &input, jsonl),
    }
}

fn run_eval(expr: &str, input: &PathBuf, jsonl: bool) -> Result<()> {
    let expr = slice_core::parse(expr).context("failed to parse SLICE expression")?;
    let content = fs::read_to_string(input)
        .with_context(|| format!("failed to read input {}", input.display()))?;

    if jsonl {
        for line in content.lines().filter(|line| !line.trim().is_empty()) {
            let value: Value = serde_json::from_str(line).context("failed to parse JSONL row")?;
            if expr.matches(&value) {
                println!("{}", serde_json::to_string(&value)?);
            }
        }
        return Ok(());
    }

    let value: Value = serde_json::from_str(&content).context("failed to parse JSON input")?;
    match value {
        Value::Array(items) => {
            for item in items {
                if expr.matches(&item) {
                    println!("{}", serde_json::to_string(&item)?);
                }
            }
        }
        item if expr.matches(&item) => {
            println!("{}", serde_json::to_string(&item)?);
        }
        _ => {}
    }

    Ok(())
}
