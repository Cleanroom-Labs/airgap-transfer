/// Integration tests: error scenarios and safety checks.
use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn cmd() -> Command {
    Command::cargo_bin("airgap-transfer").unwrap()
}

/// Pack with non-existent source produces an error.
#[test]
fn pack_missing_source() {
    let chunks_dir = tempdir().unwrap();

    cmd()
        .args([
            "pack",
            "/nonexistent/path/to/source",
            chunks_dir.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

/// Unpack with missing chunk files produces an error.
#[test]
fn unpack_missing_chunks() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();
    let out_dir = tempdir().unwrap();

    // Pack a file first
    fs::write(src_dir.path().join("data.txt"), b"test").unwrap();
    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Remove the chunk file
    fs::remove_file(chunks_dir.path().join("chunk_000.tar")).unwrap();

    // Unpack should fail
    cmd()
        .args([
            "unpack",
            chunks_dir.path().to_str().unwrap(),
            out_dir.path().to_str().unwrap(),
            "--force",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

/// Unpack with corrupted chunk produces a checksum error.
#[test]
fn unpack_corrupted_chunk() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();
    let out_dir = tempdir().unwrap();

    fs::write(src_dir.path().join("data.txt"), b"test data").unwrap();
    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Corrupt the chunk
    fs::write(chunks_dir.path().join("chunk_000.tar"), b"corrupted!").unwrap();

    // Unpack should fail with checksum error
    cmd()
        .args([
            "unpack",
            chunks_dir.path().to_str().unwrap(),
            out_dir.path().to_str().unwrap(),
            "--force",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("checksum"));
}

/// Pack over existing manifest without --force produces an error.
#[test]
fn pack_overwrite_protection() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();

    fs::write(src_dir.path().join("data.txt"), b"test").unwrap();

    // First pack succeeds
    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Second pack without --force should fail
    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--force"));
}

/// Pack over existing manifest with --force succeeds.
#[test]
fn pack_force_overwrite() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();

    fs::write(src_dir.path().join("data.txt"), b"test").unwrap();

    // First pack
    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Second pack with --force succeeds
    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
            "--force",
        ])
        .assert()
        .success();
}

/// Unpack into non-empty directory without --force produces an error.
#[test]
fn unpack_nonempty_dest_protection() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();
    let out_dir = tempdir().unwrap();

    fs::write(src_dir.path().join("data.txt"), b"test").unwrap();

    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Put a file in the output dir
    fs::write(out_dir.path().join("existing.txt"), b"blocker").unwrap();

    // Unpack without --force should fail
    cmd()
        .args([
            "unpack",
            chunks_dir.path().to_str().unwrap(),
            out_dir.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--force"));
}

/// Unpack with --no-verify skips checksum validation.
#[test]
fn unpack_no_verify_skips_checksums() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();
    let out_dir = tempdir().unwrap();

    fs::write(src_dir.path().join("data.txt"), b"test data").unwrap();
    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Unpack with --no-verify should not print "Verifying"
    cmd()
        .args([
            "unpack",
            chunks_dir.path().to_str().unwrap(),
            out_dir.path().to_str().unwrap(),
            "--force",
            "--no-verify",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Verifying").not());
}

/// Unpack with --keep-chunks preserves chunk files.
#[test]
fn unpack_keep_chunks() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();
    let out_dir = tempdir().unwrap();

    fs::write(src_dir.path().join("data.txt"), b"keep me").unwrap();
    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    cmd()
        .args([
            "unpack",
            chunks_dir.path().to_str().unwrap(),
            out_dir.path().to_str().unwrap(),
            "--force",
            "--keep-chunks",
        ])
        .assert()
        .success();

    // Chunk file and manifest should still exist
    assert!(chunks_dir.path().join("chunk_000.tar").exists());
    assert!(
        chunks_dir
            .path()
            .join("airgap-transfer-manifest.json")
            .exists()
    );
}
