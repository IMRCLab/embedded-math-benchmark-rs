//! Merge benchmark `BENCH ` result lines from one or more logs into a single
//! headed CSV. Rows are treated as opaque text — no per-column parsing — so the
//! benchmark output format can change without touching this crate. The column
//! names live in exactly one place: [`HEADER`].

use std::collections::BTreeMap;
use std::fmt;
use std::io::{BufRead, Write};
use std::path::Path;
use std::process::Command;

/// Marker that tags a result row in the logs (note the trailing space). It need
/// not start the line — a log source may prefix lines with a timestamp — so
/// `collect` searches for it anywhere in the line.
pub const BENCH_PREFIX: &str = "BENCH ";

/// CSV header prepended to the output. This is the only place the columns are
/// named; if the benchmark output format changes, edit this line.
pub const HEADER: &str = mrs_benchmark_core::CSV_HEADER;

/// Rust crates whose resolved version we look up in `Cargo.lock`. Names match
/// both the Cargo package name and the `library` field these benchmarks report.
pub const TRACKED_RUST_LIBRARIES: &[&str] = &["glam", "libm", "micromath", "nalgebra"];

/// Path (repo-root-relative) to the `crazyflie-firmware` git submodule, whose
/// pinned commit stands in for a version number (it tracks `master`, no semver tags).
pub const CRAZYFLIE_FW_SUBMODULE_PATH: &str = "benchmarks/vendor/crazyflie-firmware";

/// Anything that can go wrong while collecting rows.
#[derive(Debug)]
pub enum CollectError {
    /// No `BENCH ` rows were found across all inputs.
    Empty,
    Io(std::io::Error),
}

impl fmt::Display for CollectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CollectError::Empty => f.write_str("no `BENCH ` result rows found in input"),
            CollectError::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for CollectError {}

impl From<std::io::Error> for CollectError {
    fn from(e: std::io::Error) -> Self {
        CollectError::Io(e)
    }
}

/// Extracts the pinned commit SHA (shortened to 7 hex chars) from one line of
/// `git ls-tree HEAD -- <path>` output, e.g.
/// `"160000 commit f45ff8fdce4e73e5ce88ec06eee76309d2fa9a39\tbenchmarks/vendor/crazyflie-firmware\n"`.
fn parse_ls_tree_sha(output: &str) -> Option<String> {
    let sha = output.split_whitespace().nth(2)?;
    (sha.len() >= 7).then(|| sha[..7].to_string())
}

/// Reads a git submodule's pinned commit via `git ls-tree` -- works even when the
/// submodule itself isn't checked out, since the pin lives in the parent repo's
/// own tree object.
fn submodule_commit(repo_root: &Path, submodule_path: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["ls-tree", "HEAD", "--", submodule_path])
        .current_dir(repo_root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_ls_tree_sha(&String::from_utf8(output.stdout).ok()?)
}

/// Best-effort snapshot of each math library's version: the four Rust crates
/// (from `benchmarks/Cargo.lock`) plus `crazyflie-fw`'s pinned commit. Never
/// fails -- a missing/unreadable source just leaves that one library out of the
/// returned map, with a human-readable line in the returned warnings instead.
pub fn library_versions(repo_root: &Path) -> (BTreeMap<String, String>, Vec<String>) {
    let mut warnings = Vec::new();
    let lockfile_path = repo_root.join("benchmarks/Cargo.lock");
    let mut versions = match cargo_lock::Lockfile::load(&lockfile_path) {
        Ok(lockfile) => lockfile
            .packages
            .into_iter()
            .filter(|pkg| TRACKED_RUST_LIBRARIES.contains(&pkg.name.as_str()))
            .fold(BTreeMap::new(), |mut versions, pkg| {
                versions
                    .entry(pkg.name.to_string())
                    .or_insert_with(|| pkg.version.to_string());
                versions
            }),
        Err(e) => {
            warnings.push(format!("could not read {}: {e}", lockfile_path.display()));
            BTreeMap::new()
        }
    };

    match submodule_commit(repo_root, CRAZYFLIE_FW_SUBMODULE_PATH) {
        Some(sha) => {
            versions.insert("crazyflie-fw".to_string(), sha);
        }
        None => warnings.push(format!(
            "could not resolve the pinned commit for {CRAZYFLIE_FW_SUBMODULE_PATH}"
        )),
    }

    (versions, warnings)
}

/// Read the `BENCH ` rows from each source and return them sorted.
///
/// The `BENCH ` marker is matched anywhere in a line, not only at the start:
/// probe-rs / RTT may prefix each line with a host timestamp or log level. Lines
/// without the marker are ignored. Errors if no rows are found at all.
pub fn collect(sources: Vec<Box<dyn BufRead>>) -> Result<Vec<String>, CollectError> {
    let mut rows = Vec::new();
    for reader in sources {
        for line in reader.lines() {
            let line = line?;
            // Keep whatever follows the marker, trimming any trailing CR left by
            // CRLF-terminated logs.
            if let Some(pos) = line.find(BENCH_PREFIX) {
                let row = line[pos + BENCH_PREFIX.len()..].trim_end();
                rows.push(row.to_string());
            }
        }
    }
    if rows.is_empty() {
        return Err(CollectError::Empty);
    }
    rows.sort();
    Ok(rows)
}

/// Write the header followed by the rows, each `\n`-terminated.
pub fn write_rows<W: Write>(rows: &[String], mut out: W) -> Result<(), CollectError> {
    writeln!(out, "{HEADER}")?;
    for row in rows {
        writeln!(out, "{row}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // Fake rows on purpose — the tool doesn't parse columns, so these tests
    // never need editing when the benchmark output format changes.
    fn src(data: &str) -> Box<dyn BufRead> {
        Box::new(Cursor::new(data.as_bytes().to_vec()))
    }

    #[test]
    fn keeps_bench_rows_strips_prefix_and_sorts() {
        let rows = collect(vec![
            src("log noise\nBENCH b,2\nmore noise\n"),
            src("BENCH a,1\n"),
        ])
        .unwrap();
        assert_eq!(rows, vec!["a,1".to_string(), "b,2".to_string()]);
    }

    #[test]
    fn finds_bench_marker_after_a_log_prefix_and_trims_cr() {
        // probe-rs / RTT can prefix each line with a timestamp; rows may be CRLF-terminated.
        let rows = collect(vec![src("2026-07-03T12:00:00Z INFO rtt: BENCH a,1\r\n")]).unwrap();
        assert_eq!(rows, vec!["a,1".to_string()]);
    }

    #[test]
    fn errors_when_no_bench_rows() {
        assert!(matches!(
            collect(vec![src("just logs\nnothing here\n")]),
            Err(CollectError::Empty)
        ));
    }

    #[test]
    fn write_rows_prepends_header() {
        let mut buf = Vec::new();
        write_rows(&["a,1".to_string(), "b,2".to_string()], &mut buf).unwrap();
        assert_eq!(
            String::from_utf8(buf).unwrap(),
            format!("{HEADER}\na,1\nb,2\n")
        );
    }

    #[test]
    fn parse_ls_tree_sha_shortens_to_seven_chars() {
        let output = "160000 commit f45ff8fdce4e73e5ce88ec06eee76309d2fa9a39\tbenchmarks/vendor/crazyflie-firmware\n";
        assert_eq!(parse_ls_tree_sha(output), Some("f45ff8f".to_string()));
    }

    #[test]
    fn parse_ls_tree_sha_returns_none_for_empty_output() {
        assert_eq!(parse_ls_tree_sha(""), None);
    }

    #[test]
    fn library_versions_reads_lockfile_and_warns_when_not_a_git_repo() {
        let dir = std::env::temp_dir().join("mrs-benchmark-collect-test-library-versions");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("benchmarks")).unwrap();
        std::fs::write(
            dir.join("benchmarks/Cargo.lock"),
            "[[package]]\nname = \"glam\"\nversion = \"0.28.0\"\n",
        )
        .unwrap();

        let (versions, warnings) = library_versions(&dir);

        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(versions.get("glam"), Some(&"0.28.0".to_string()));
        assert_eq!(versions.get("crazyflie-fw"), None);
        assert!(
            !warnings.is_empty(),
            "expected a warning about the missing git repo"
        );
    }
}
