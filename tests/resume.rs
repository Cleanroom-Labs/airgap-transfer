/// Integration tests: resume capability for pack and unpack.
use std::fs;

use predicates::prelude::*;
use tempfile::tempdir;

mod common;
use common::cmd;

/// Simulate interrupted pack by packing, deleting a chunk, then resuming.
#[test]
fn pack_resume_after_partial() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();

    // Create source data large enough for multiple chunks
    let content: Vec<u8> = (0..3000).map(|i| (i % 256) as u8).collect();
    fs::write(src_dir.path().join("big.bin"), &content).unwrap();

    // First pack with small chunk size
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

    // Count original chunks
    let manifest_path = chunks_dir.path().join("airgap-transfer-manifest.json");
    assert!(manifest_path.exists());

    let original_manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    let chunk_count = original_manifest["chunk_count"].as_u64().unwrap();
    assert!(chunk_count >= 2, "need multiple chunks for resume test");

    // Delete the last chunk to simulate interruption
    let last_chunk = format!("chunk_{:03}.tar", chunk_count - 1);
    fs::remove_file(chunks_dir.path().join(&last_chunk)).unwrap();

    // Manually mark last chunk as pending in manifest to simulate interruption
    let mut manifest_data = original_manifest.clone();
    let chunks = manifest_data["chunks"].as_array_mut().unwrap();
    let last = chunks.last_mut().unwrap();
    last["status"] = serde_json::json!("pending");
    last["checksum"] = serde_json::json!("");
    last["size_bytes"] = serde_json::json!(0);
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest_data).unwrap(),
    )
    .unwrap();

    // Resume should succeed
    cmd()
        .args([
            "pack",
            src_dir.path().join("big.bin").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
            "--chunk-size",
            "1024",
            "--resume",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Resuming"));

    // All chunks should now exist
    for i in 0..chunk_count {
        assert!(
            chunks_dir
                .path()
                .join(format!("chunk_{:03}.tar", i))
                .exists(),
            "chunk_{:03}.tar should exist after resume",
            i
        );
    }

    // Verify the result unpacks correctly
    let out_dir = tempdir().unwrap();
    cmd()
        .args([
            "unpack",
            chunks_dir.path().to_str().unwrap(),
            out_dir.path().to_str().unwrap(),
            "--force",
        ])
        .assert()
        .success();
}

/// Pack --resume on completed pack should be a no-op.
#[test]
fn pack_resume_already_complete() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();

    fs::write(src_dir.path().join("data.txt"), b"test data").unwrap();

    // Pack fully
    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Resume should report nothing to do
    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
            "--resume",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("already completed"));
}

/// Pack --resume with incompatible manifest should error.
#[test]
fn pack_resume_incompatible_manifest() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();

    fs::write(src_dir.path().join("data.txt"), b"test").unwrap();

    // Pack with default chunk size
    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Try resume with different chunk size
    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
            "--chunk-size",
            "512",
            "--resume",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not compatible"));
}

/// Unpack --resume into non-empty directory succeeds.
#[test]
fn unpack_resume_nonempty_dest() {
    let src_dir = tempdir().unwrap();
    let chunks_dir = tempdir().unwrap();
    let out_dir = tempdir().unwrap();

    let content = b"resume unpack test";
    fs::write(src_dir.path().join("data.txt"), content).unwrap();

    // Pack
    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Put a file in output dir
    fs::write(out_dir.path().join("existing.txt"), b"old data").unwrap();

    // Regular unpack should fail
    cmd()
        .args([
            "unpack",
            chunks_dir.path().to_str().unwrap(),
            out_dir.path().to_str().unwrap(),
        ])
        .assert()
        .failure();

    // Unpack --resume should succeed
    cmd()
        .args([
            "unpack",
            chunks_dir.path().to_str().unwrap(),
            out_dir.path().to_str().unwrap(),
            "--resume",
            "--keep-chunks",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Resuming"));

    // Verify extracted file
    assert_eq!(fs::read(out_dir.path().join("data.txt")).unwrap(), content);
}

/// Pack error message suggests both --force and --resume.
#[test]
fn pack_suggests_resume_and_force() {
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

    // Second pack without flags should mention both options
    cmd()
        .args([
            "pack",
            src_dir.path().join("data.txt").to_str().unwrap(),
            chunks_dir.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--force"))
        .stderr(predicate::str::contains("--resume"));
}
