//! CLI: merge benchmark `BENCH ` result lines into a single headed CSV.

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use mrs_benchmark_collect::{collect, write_rows, CollectError};

/// Merge benchmark `BENCH ` result lines into a single headed CSV.
#[derive(Parser)]
#[command(name = "mrs-benchmark-collect", version, about)]
struct Cli {
    /// Log files to read. If none are given, read from stdin.
    files: Vec<PathBuf>,

    /// Write the CSV here instead of stdout.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), CollectError> {
    let sources: Vec<Box<dyn BufRead>> = if cli.files.is_empty() {
        vec![Box::new(BufReader::new(io::stdin()))]
    } else {
        let mut sources = Vec::with_capacity(cli.files.len());
        for path in &cli.files {
            let file = File::open(path).map_err(|e| {
                io::Error::new(e.kind(), format!("opening {}: {e}", path.display()))
            })?;
            sources.push(Box::new(BufReader::new(file)) as Box<dyn BufRead>);
        }
        sources
    };

    let rows = collect(sources)?;

    match cli.output {
        Some(path) => write_rows(&rows, File::create(&path)?)?,
        None => write_rows(&rows, io::stdout().lock())?,
    }
    Ok(())
}
