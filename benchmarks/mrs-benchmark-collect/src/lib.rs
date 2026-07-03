//! Merge benchmark `BENCH ` result lines from one or more logs into a single
//! headed CSV. Rows are treated as opaque text — no per-column parsing — so the
//! benchmark output format can change without touching this crate. The column
//! names live in exactly one place: [`HEADER`].

use std::fmt;
use std::io::{BufRead, Write};

/// Lines carrying a result row start with this prefix (note the trailing space).
pub const BENCH_PREFIX: &str = "BENCH ";

/// CSV header prepended to the output. This is the only place the columns are
/// named; if the benchmark output format changes, edit this line.
pub const HEADER: &str = "test,library,platform,input_index,reps,min_duration,unit";

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

/// Read the `BENCH ` rows from each source, strip the prefix, and return them
/// sorted. Non-`BENCH ` lines are ignored. Errors if no rows are found at all.
pub fn collect(sources: Vec<Box<dyn BufRead>>) -> Result<Vec<String>, CollectError> {
    let mut rows = Vec::new();
    for reader in sources {
        for line in reader.lines() {
            if let Some(row) = line?.strip_prefix(BENCH_PREFIX) {
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
}
