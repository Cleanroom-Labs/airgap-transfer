/// Pluggable hash verification for chunk integrity.
///
/// Provides a trait-based backend so new algorithms can be added without
/// modifying existing code.  SHA-256 is the default and only v1.0 backend.
use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::{AirgapError, Result};

// ── Traits ──────────────────────────────────────────────────────────────

/// Factory for creating streaming hash writers.
///
/// Each algorithm (SHA-256, future BLAKE3, etc.) implements this trait
/// so the caller can select the algorithm at runtime via CLI flags.
pub trait HashAlgorithm: Send + Sync {
    /// Short lowercase name used in manifests and CLI (e.g. `"sha256"`).
    fn name(&self) -> &str;

    /// Create a fresh, zero-state hasher ready to accept data.
    fn create_writer(&self) -> Box<dyn HashWriter>;
}

/// Streaming hash accumulator.
///
/// Feed data via [`update`](HashWriter::update), then call
/// [`finalize`](HashWriter::finalize) to obtain the prefixed digest string.
pub trait HashWriter {
    /// Feed bytes into the running hash state.
    fn update(&mut self, data: &[u8]);

    /// Consume the writer and return `"algorithm:hex_digest"`.
    fn finalize(self: Box<Self>) -> String;
}

// ── SHA-256 implementation ──────────────────────────────────────────────

/// SHA-256 hash algorithm backend (default).
pub struct Sha256Algorithm;

impl HashAlgorithm for Sha256Algorithm {
    fn name(&self) -> &str {
        "sha256"
    }

    fn create_writer(&self) -> Box<dyn HashWriter> {
        Box::new(Sha256Writer {
            hasher: Sha256::new(),
        })
    }
}

struct Sha256Writer {
    hasher: Sha256,
}

impl HashWriter for Sha256Writer {
    fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    fn finalize(self: Box<Self>) -> String {
        let digest = self.hasher.finalize();
        format!("sha256:{:x}", digest)
    }
}

// ── Public helpers ──────────────────────────────────────────────────────

/// Look up a [`HashAlgorithm`] by its CLI / manifest name.
///
/// Returns `Err(UnsupportedAlgorithm)` for unknown names.
pub fn algorithm_from_name(name: &str) -> Result<Box<dyn HashAlgorithm>> {
    match name {
        "sha256" => Ok(Box::new(Sha256Algorithm)),
        other => Err(AirgapError::UnsupportedAlgorithm(other.to_string())),
    }
}

/// Compute the checksum of an entire file using the given algorithm.
///
/// Returns the prefixed digest string (e.g. `"sha256:abcdef…"`).
pub fn compute_checksum(path: &Path, algorithm: &dyn HashAlgorithm) -> Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut writer = algorithm.create_writer();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        writer.update(&buf[..n]);
    }
    Ok(writer.finalize())
}

/// Verify that a file's checksum matches an expected value.
///
/// Returns `Ok(true)` on match, `Ok(false)` on mismatch.
pub fn verify_checksum(path: &Path, expected: &str, algorithm: &dyn HashAlgorithm) -> Result<bool> {
    let actual = compute_checksum(path, algorithm)?;
    Ok(actual == expected)
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// TC-INT-001: Generate SHA-256 checksum, verify matches known value.
    #[test]
    fn sha256_known_value() {
        let mut writer = Sha256Algorithm.create_writer();
        writer.update(b"hello world");
        let digest = writer.finalize();
        // SHA-256("hello world") is a well-known value
        assert_eq!(
            digest,
            "sha256:b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    /// TC-CRA-002: Default hash algorithm is SHA-256.
    #[test]
    fn default_algorithm_is_sha256() {
        let algo = algorithm_from_name("sha256").unwrap();
        assert_eq!(algo.name(), "sha256");
    }

    /// TC-CRA-006: Invalid algorithm name is rejected.
    #[test]
    fn invalid_algorithm_rejected() {
        let result = algorithm_from_name("md5");
        assert!(result.is_err());
        match result {
            Err(e) => assert!(
                e.to_string().contains("md5"),
                "error should mention the bad algorithm"
            ),
            Ok(_) => panic!("expected error for unsupported algorithm"),
        }
    }

    /// TC-CRA-005: Pluggable hash backend interface works via trait object.
    #[test]
    fn trait_object_dispatch() {
        let algo: Box<dyn HashAlgorithm> = Box::new(Sha256Algorithm);
        let mut writer = algo.create_writer();
        writer.update(b"test");
        let digest = writer.finalize();
        assert!(digest.starts_with("sha256:"));
    }

    /// TC-INT-001 (file variant): Compute checksum of a real file.
    #[test]
    fn compute_file_checksum() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        std::fs::write(&path, b"hello world").unwrap();

        let checksum = compute_checksum(&path, &Sha256Algorithm).unwrap();
        assert_eq!(
            checksum,
            "sha256:b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    /// TC-INT-002: Verify checksum returns true on match.
    #[test]
    fn verify_checksum_match() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        std::fs::write(&path, b"hello world").unwrap();

        let expected = "sha256:b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert!(verify_checksum(&path, expected, &Sha256Algorithm).unwrap());
    }

    /// TC-INT-003: Detect corrupted data (checksum mismatch).
    #[test]
    fn verify_checksum_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");
        std::fs::write(&path, b"hello world").unwrap();

        // Flip last char of the expected digest
        let wrong = "sha256:b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde0";
        assert!(!verify_checksum(&path, wrong, &Sha256Algorithm).unwrap());
    }

    /// TC-INT-004: Verify integrity after content modification.
    #[test]
    fn detect_modified_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.bin");

        // Write original content and capture checksum
        std::fs::write(&path, b"original content").unwrap();
        let original_checksum = compute_checksum(&path, &Sha256Algorithm).unwrap();

        // Modify the file
        let mut f = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
        f.write_all(b"modified content").unwrap();
        drop(f);

        // Checksum should no longer match
        assert!(!verify_checksum(&path, &original_checksum, &Sha256Algorithm).unwrap());
    }

    /// TC-CRA-003 / TC-CRA-004: Algorithm name appears in digest prefix.
    #[test]
    fn digest_prefix_matches_algorithm_name() {
        let algo = algorithm_from_name("sha256").unwrap();
        let mut writer = algo.create_writer();
        writer.update(b"data");
        let digest = writer.finalize();
        assert!(digest.starts_with(&format!("{}:", algo.name())));
    }

    /// Incremental update produces same result as single update.
    #[test]
    fn incremental_hashing() {
        let algo = Sha256Algorithm;

        let mut w1 = algo.create_writer();
        w1.update(b"hello world");
        let digest1 = w1.finalize();

        let mut w2 = algo.create_writer();
        w2.update(b"hello ");
        w2.update(b"world");
        let digest2 = w2.finalize();

        assert_eq!(digest1, digest2);
    }
}
