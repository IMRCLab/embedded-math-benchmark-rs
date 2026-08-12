//! CLI: merge benchmark `BENCH ` result lines into a single headed CSV.

use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use mrs_benchmark_collect::{collect, library_versions, write_rows, CollectError};

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
        Some(path) => {
            write_rows(&rows, File::create(&path)?)?;
            write_library_versions_sidecar(&path);
        }
        None => write_rows(&rows, io::stdout().lock())?,
    }
    Ok(())
}

/// Writes `library_versions.json` next to `output_path`, best-effort: a lookup
/// failure only prints a warning, it never fails the `collect` command -- this
/// is metadata for the report, not part of the `results.csv` contract.
fn write_library_versions_sidecar(output_path: &Path) {
    let Some(dir) = output_path.parent() else {
        return;
    };
    let (versions, warnings) = library_versions(&repo_root());
    for warning in &warnings {
        eprintln!("warning: {warning}");
    }
    let json = serde_json::to_string_pretty(&versions)
        .expect("serializing a BTreeMap<String, String> cannot fail");
    if let Err(e) = fs::write(dir.join("library_versions.json"), json) {
        eprintln!("warning: could not write library_versions.json: {e}");
    }
}

/// Repo root, anchored at compile time via `CARGO_MANIFEST_DIR` rather than the
/// process's runtime cwd: `mrs-benchmark-collect` lives at
/// `<repo>/benchmarks/mrs-benchmark-collect`, two levels below the repo root.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("mrs-benchmark-collect has a parent directory")
        .parent()
        .expect("benchmarks/ has a parent directory")
        .to_path_buf()
}
