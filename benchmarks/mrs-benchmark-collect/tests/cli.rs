//! End-to-end tests that drive the built binary over stdin.

use std::io::Write;
use std::process::{Command, Stdio};

use mrs_benchmark_collect::HEADER;

const BIN: &str = env!("CARGO_BIN_EXE_mrs-benchmark-collect");

fn run_with_stdin(input: &str) -> std::process::Output {
    let mut child = Command::new(BIN)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn binary");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(input.as_bytes())
        .expect("write stdin");
    child.wait_with_output().expect("wait")
}

#[test]
fn filters_strips_sorts_and_headers() {
    // Fake rows — independent of the real column format.
    let out = run_with_stdin("noise\nBENCH b,2\nBENCH a,1\nmore noise\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        format!("{HEADER}\na,1\nb,2\n")
    );
}

#[test]
fn empty_input_exits_non_zero() {
    let out = run_with_stdin("just logs, nothing to collect\n");
    assert!(!out.status.success());
}
