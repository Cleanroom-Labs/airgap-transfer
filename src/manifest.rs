/// JSON manifest for tracking transfer state and chunk metadata.
///
/// The manifest file (`airgap-transfer-manifest.json`) is the single source
/// of truth for an in-progress or completed transfer.  It records every
/// chunk's filename, size, checksum, and completion status so that operations
/// can be resumed after interruption.
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{AirgapError, Result};

/// Well-known filename written alongside chunks.
pub const MANIFEST_FILENAME: &str = "airgap-transfer-manifest.json";

/// Current schema version.
const MANIFEST_VERSION: &str = "1.0";

// ── Core types ──────────────────────────────────────────────────────────

/// Top-level manifest describing a pack or unpack operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    pub version: String,
    pub operation: Operation,
    pub source_path: String,
    pub total_size_bytes: u64,
    pub chunk_size_bytes: u64,
    pub hash_algorithm: String,
    pub chunk_count: usize,
    pub chunks: Vec<ChunkEntry>,
    /// Checksum of all source file contents (computed during pack, verified
    /// after unpack).  `None` for manifests created before this field existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_checksum: Option<String>,
    pub created_utc: DateTime<Utc>,
    pub last_updated_utc: DateTime<Utc>,
}

/// Metadata for a single chunk file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChunkEntry {
    pub index: usize,
    pub filename: String,
    pub size_bytes: u64,
    pub checksum: String,
    pub status: ChunkStatus,
}

/// Completion status of an individual chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChunkStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Which high-level operation the manifest describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Pack,
    Unpack,
}

// ── Constructors & persistence ──────────────────────────────────────────

impl Manifest {
    /// Create a new manifest for a pack operation.
    ///
    /// Chunks are pre-populated with `Pending` status and expected filenames.
    pub fn new_pack(
        source_path: &str,
        total_size_bytes: u64,
        chunk_size_bytes: u64,
        hash_algorithm: &str,
    ) -> Self {
        let chunk_count = if total_size_bytes == 0 {
            1
        } else {
            total_size_bytes.div_ceil(chunk_size_bytes) as usize
        };

        let now = Utc::now();
        let chunks = (0..chunk_count)
            .map(|i| ChunkEntry {
                index: i,
                filename: format!("chunk_{:03}.tar", i),
                size_bytes: 0,
                checksum: String::new(),
                status: ChunkStatus::Pending,
            })
            .collect();

        Manifest {
            version: MANIFEST_VERSION.to_string(),
            operation: Operation::Pack,
            source_path: source_path.to_string(),
            total_size_bytes,
            chunk_size_bytes,
            hash_algorithm: hash_algorithm.to_string(),
            chunk_count,
            chunks,
            source_checksum: None,
            created_utc: now,
            last_updated_utc: now,
        }
    }

    /// Load a manifest from a JSON file.
    pub fn load(path: &Path) -> Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let manifest: Manifest =
            serde_json::from_str(&data).map_err(|e| AirgapError::ManifestInvalid(e.to_string()))?;
        Ok(manifest)
    }

    /// Save the manifest as pretty-printed JSON.
    pub fn save(&mut self, path: &Path) -> Result<()> {
        self.last_updated_utc = Utc::now();
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Update a chunk's status, size, and checksum after processing.
    pub fn update_chunk(
        &mut self,
        index: usize,
        status: ChunkStatus,
        size_bytes: u64,
        checksum: &str,
    ) {
        if let Some(chunk) = self.chunks.get_mut(index) {
            chunk.status = status;
            chunk.size_bytes = size_bytes;
            chunk.checksum = checksum.to_string();
        }
    }

    /// Find the index of the first chunk that is not completed.
    ///
    /// Returns `None` if all chunks are completed (nothing to resume).
    pub fn first_incomplete_chunk(&self) -> Option<usize> {
        self.chunks
            .iter()
            .position(|c| c.status != ChunkStatus::Completed)
    }

    /// Check whether this manifest is compatible with a new pack operation,
    /// allowing resume of a previous interrupted pack.
    pub fn is_compatible_pack(
        &self,
        source_path: &str,
        chunk_size_bytes: u64,
        hash_algorithm: &str,
    ) -> bool {
        self.operation == Operation::Pack
            && self.source_path == source_path
            && self.chunk_size_bytes == chunk_size_bytes
            && self.hash_algorithm == hash_algorithm
    }
}

// ── Display ─────────────────────────────────────────────────────────────

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Pack => write!(f, "pack"),
            Operation::Unpack => write!(f, "unpack"),
        }
    }
}

impl std::fmt::Display for ChunkStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChunkStatus::Pending => write!(f, "pending"),
            ChunkStatus::InProgress => write!(f, "in_progress"),
            ChunkStatus::Completed => write!(f, "completed"),
            ChunkStatus::Failed => write!(f, "failed"),
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// TC-PCK-005: Manifest creation with correct structure.
    #[test]
    fn new_pack_manifest() {
        let m = Manifest::new_pack("/data/bigfile.iso", 3_000_000_000, 1_000_000_000, "sha256");

        assert_eq!(m.version, "1.0");
        assert_eq!(m.operation, Operation::Pack);
        assert_eq!(m.source_path, "/data/bigfile.iso");
        assert_eq!(m.total_size_bytes, 3_000_000_000);
        assert_eq!(m.chunk_size_bytes, 1_000_000_000);
        assert_eq!(m.hash_algorithm, "sha256");
        assert_eq!(m.chunk_count, 3);
        assert_eq!(m.chunks.len(), 3);

        // Chunks should be named chunk_000.tar through chunk_002.tar
        assert_eq!(m.chunks[0].filename, "chunk_000.tar");
        assert_eq!(m.chunks[1].filename, "chunk_001.tar");
        assert_eq!(m.chunks[2].filename, "chunk_002.tar");

        // All pending initially
        for chunk in &m.chunks {
            assert_eq!(chunk.status, ChunkStatus::Pending);
            assert_eq!(chunk.size_bytes, 0);
            assert!(chunk.checksum.is_empty());
        }
    }

    /// Chunk count rounds up correctly.
    #[test]
    fn chunk_count_rounds_up() {
        // 2.5 chunks worth of data → 3 chunks
        let m = Manifest::new_pack("/src", 2_500, 1_000, "sha256");
        assert_eq!(m.chunk_count, 3);
        assert_eq!(m.chunks.len(), 3);
    }

    /// Zero-byte source still creates one chunk.
    #[test]
    fn zero_byte_source() {
        let m = Manifest::new_pack("/empty", 0, 1_000_000, "sha256");
        assert_eq!(m.chunk_count, 1);
    }

    /// TC-STA-001: Manifest serialization round-trip.
    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(MANIFEST_FILENAME);

        let mut original = Manifest::new_pack("/data/test", 5_000, 2_000, "sha256");
        original.update_chunk(0, ChunkStatus::Completed, 2_000, "sha256:aaa");
        original.save(&path).unwrap();

        let loaded = Manifest::load(&path).unwrap();
        // Timestamps may differ slightly due to save() updating last_updated_utc,
        // so compare fields individually.
        assert_eq!(loaded.version, original.version);
        assert_eq!(loaded.operation, original.operation);
        assert_eq!(loaded.source_path, original.source_path);
        assert_eq!(loaded.chunk_count, original.chunk_count);
        assert_eq!(loaded.chunks[0].status, ChunkStatus::Completed);
        assert_eq!(loaded.chunks[0].checksum, "sha256:aaa");
    }

    /// TC-STA-002: Chunk status tracking via update_chunk.
    #[test]
    fn update_chunk_status() {
        let mut m = Manifest::new_pack("/src", 10_000, 5_000, "sha256");

        // Initially pending
        assert_eq!(m.chunks[0].status, ChunkStatus::Pending);

        // Mark in-progress
        m.update_chunk(0, ChunkStatus::InProgress, 0, "");
        assert_eq!(m.chunks[0].status, ChunkStatus::InProgress);

        // Mark completed with checksum
        m.update_chunk(0, ChunkStatus::Completed, 5_000, "sha256:abc123");
        assert_eq!(m.chunks[0].status, ChunkStatus::Completed);
        assert_eq!(m.chunks[0].size_bytes, 5_000);
        assert_eq!(m.chunks[0].checksum, "sha256:abc123");

        // Second chunk still pending
        assert_eq!(m.chunks[1].status, ChunkStatus::Pending);
    }

    /// JSON output matches expected schema structure.
    #[test]
    fn json_schema_structure() {
        let m = Manifest::new_pack("/test", 1_000, 1_000, "sha256");
        let json = serde_json::to_string_pretty(&m).unwrap();

        // Verify key fields appear in the JSON
        assert!(json.contains("\"version\": \"1.0\""));
        assert!(json.contains("\"operation\": \"pack\""));
        assert!(json.contains("\"hash_algorithm\": \"sha256\""));
        assert!(json.contains("\"chunk_000.tar\""));
        assert!(json.contains("\"pending\""));
    }

    /// Loading a non-existent file returns an I/O error.
    #[test]
    fn load_missing_file() {
        let result = Manifest::load(Path::new("/nonexistent/manifest.json"));
        assert!(result.is_err());
    }

    /// Loading invalid JSON returns a ManifestInvalid error.
    #[test]
    fn load_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.json");
        std::fs::write(&path, "not valid json {{{").unwrap();

        let result = Manifest::load(&path);
        assert!(result.is_err());
    }
}
