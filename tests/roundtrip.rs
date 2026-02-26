/// Integration tests: end-to-end pack → list → unpack → verify roundtrip.
use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn cmd() -> Command {
    Command::cargo_bin("airgap-transfer").unwrap()
}

/// Helper: create a source file with known content.
fn create_source_file(dir: &std::path::Path, name: &str, content: &[u8]) {
    fs::write(dir.join(name), content).unwrap();
}

/// Pack a single file, list it, unpack it, compare byte-for-byte.
#[test]
fn single_file_roundtrip() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();
    let out_dir = tempdir().unwrap();

    let content = b"Hello, air-gapped world! This is test data for roundtrip.";
    create_source_file(src_dir.path(), "hello.txt", content);

    // Pack
    cmd()
        .args([
            "pack",
            src_dir.path().join("hello.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
            "--chunk-size",
            "1MB",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pack complete"));

    // List
    cmd()
        .args(["list", chunks_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("present"));

    // List --verify
    cmd()
        .args(["list", chunks_dir.path().to_str().unwrap(), "--verify"])
        .assert()
        .success()
        .stdout(predicate::str::contains("verified"));

    // Unpack
    cmd()
        .args([
            "unpack",
            chunks_dir.path().to_str().unwrap(),
            out_dir.path().to_str().unwrap(),
            "--force",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Unpack complete"));

    // Verify byte-for-byte match
    let output = fs::read(out_dir.path().join("hello.txt")).unwrap();
    assert_eq!(output, content);
}

/// Find a file by name anywhere under a root directory.
fn find_file(root: &std::path::Path, name: &str) -> Vec<u8> {
    fn walk(dir: &std::path::Path, name: &str) -> Option<Vec<u8>> {
        for entry in fs::read_dir(dir).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = walk(&path, name) {
                    return Some(found);
                }
            } else if path.file_name().and_then(|n| n.to_str()) == Some(name) {
                return Some(fs::read(&path).unwrap());
            }
        }
        None
    }
    walk(root, name).unwrap_or_else(|| panic!("file {name} not found under {}", root.display()))
}

/// Pack a directory with multiple files, unpack, compare all.
#[test]
fn directory_roundtrip() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();
    let out_dir = tempdir().unwrap();

    create_source_file(src_dir.path(), "a.txt", b"file a content");
    create_source_file(src_dir.path(), "b.bin", &[0u8; 1024]);
    fs::create_dir_all(src_dir.path().join("subdir")).unwrap();
    create_source_file(src_dir.path(), "subdir/c.txt", b"nested file");

    // Pack directory
    cmd()
        .args([
            "pack",
            src_dir.path().to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
            "--chunk-size",
            "1MB",
        ])
        .assert()
        .success();

    // Unpack
    cmd()
        .args([
            "unpack",
            chunks_dir.path().to_str().unwrap(),
            out_dir.path().to_str().unwrap(),
            "--force",
        ])
        .assert()
        .success();

    // Verify all files (tar preserves source dir name as prefix, so search by filename)
    assert_eq!(find_file(out_dir.path(), "a.txt"), b"file a content");
    assert_eq!(find_file(out_dir.path(), "b.bin"), vec![0u8; 1024]);
    assert_eq!(find_file(out_dir.path(), "c.txt"), b"nested file");
}

/// Pack with --dry-run should produce no files.
#[test]
fn dry_run_writes_nothing() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();

    create_source_file(src_dir.path(), "data.txt", b"test data");

    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run"));

    // No chunk files should exist
    let entries: Vec<_> = fs::read_dir(chunks_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(entries.is_empty(), "dry-run should not write any files");
}

/// Pack with --no-verify still produces valid chunks that can be unpacked.
#[test]
fn no_verify_still_valid() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();
    let out_dir = tempdir().unwrap();

    let content = b"no-verify test data";
    create_source_file(src_dir.path(), "nv.txt", content);

    // Pack with --no-verify
    cmd()
        .args([
            "pack",
            src_dir.path().join("nv.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
            "--no-verify",
        ])
        .assert()
        .success();

    // Unpack (with default verification) should still work
    cmd()
        .args([
            "unpack",
            chunks_dir.path().to_str().unwrap(),
            out_dir.path().to_str().unwrap(),
            "--force",
        ])
        .assert()
        .success();

    assert_eq!(fs::read(out_dir.path().join("nv.txt")).unwrap(), content);
}

/// Multi-chunk roundtrip: source larger than chunk size produces multiple chunks.
#[test]
fn multi_chunk_roundtrip() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();
    let out_dir = tempdir().unwrap();

    // Create ~3KB file, use 1KB chunks → should produce multiple chunks
    let content: Vec<u8> = (0..3000).map(|i| (i % 256) as u8).collect();
    create_source_file(src_dir.path(), "big.bin", &content);

    cmd()
        .args([
            "pack",
            src_dir.path().join("big.bin").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
            "--chunk-size",
            "1024",
        ])
        .assert()
        .success();

    // Should have multiple chunk files
    let chunk_count = fs::read_dir(chunks_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map_or(false, |n| n.starts_with("chunk_"))
        })
        .count();
    assert!(
        chunk_count > 1,
        "expected multiple chunks, got {chunk_count}"
    );

    // List should show all present
    cmd()
        .args(["list", chunks_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "{}/{} present",
            chunk_count, chunk_count
        )));

    // Unpack and verify
    cmd()
        .args([
            "unpack",
            chunks_dir.path().to_str().unwrap(),
            out_dir.path().to_str().unwrap(),
            "--force",
        ])
        .assert()
        .success();

    assert_eq!(fs::read(out_dir.path().join("big.bin")).unwrap(), content);
}
