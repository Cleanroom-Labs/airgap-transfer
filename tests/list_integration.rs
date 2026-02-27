/// Integration tests: list command behavior.
use std::fs;

use predicates::prelude::*;
use tempfile::tempdir;

mod common;
use common::cmd;

/// Spec: TC-LST-001
/// List after pack shows all chunks present.
#[test]
fn list_all_present() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();

    fs::write(src_dir.path().join("data.txt"), b"list test data").unwrap();
    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    cmd()
        .args(["list", chunks_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("1/1 present"))
        .stdout(predicate::str::contains("All 1 chunks present"));
}

/// Spec: TC-LST-005
/// List with --verify reports OK for intact chunks.
#[test]
fn list_verify_ok() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();

    fs::write(src_dir.path().join("data.txt"), b"verify test").unwrap();
    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    cmd()
        .args(["list", chunks_dir.path().to_str().unwrap(), "--verify"])
        .assert()
        .success()
        .stdout(predicate::str::contains("OK"))
        .stdout(predicate::str::contains("verified"));
}

/// Spec: TC-LST-003
/// List after removing a chunk shows MISSING.
#[test]
fn list_missing_chunk() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();

    fs::write(src_dir.path().join("data.txt"), b"missing test").unwrap();
    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Remove chunk
    fs::remove_file(chunks_dir.path().join("chunk_000.tar")).unwrap();

    cmd()
        .args(["list", chunks_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("MISSING"))
        .stdout(predicate::str::contains("0/1 present"))
        .stdout(predicate::str::contains("missing"));
}

/// Spec: TC-LST-005
/// List with --verify on corrupted chunk shows CORRUPT.
#[test]
fn list_verify_corrupt() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();

    fs::write(src_dir.path().join("data.txt"), b"corrupt test").unwrap();
    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Corrupt the chunk
    fs::write(chunks_dir.path().join("chunk_000.tar"), b"bad data").unwrap();

    cmd()
        .args(["list", chunks_dir.path().to_str().unwrap(), "--verify"])
        .assert()
        .success()
        .stdout(predicate::str::contains("CORRUPT"))
        .stdout(predicate::str::contains("corrupted"));
}

/// Spec: TC-TRANSFER-ERR-002
/// List without a manifest produces an error.
#[test]
fn list_no_manifest() {
    let empty_dir = tempdir().unwrap();

    cmd()
        .args(["list", empty_dir.path().to_str().unwrap()])
        .assert()
        .failure();
}
