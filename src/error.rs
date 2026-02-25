/// Error types for AirGap Transfer operations.
///
/// All errors are surfaced to the user with clear, actionable messages.
/// The `Result` type alias is used throughout the crate for consistency.
use std::path::PathBuf;

/// Convenience type alias used throughout the crate.
pub type Result<T> = std::result::Result<T, AirgapError>;

/// Top-level error type for all AirGap Transfer operations.
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)] // Variants used in later phases (Unpack, List).
pub enum AirgapError {
    /// Wraps I/O errors from file and USB operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization failures.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Checksum mismatch during verification.
    #[error("checksum mismatch for {path}: expected {expected}, got {actual}")]
    Checksum {
        path: PathBuf,
        expected: String,
        actual: String,
    },

    /// Manifest file is missing or structurally invalid.
    #[error("invalid manifest: {0}")]
    ManifestInvalid(String),

    /// A required chunk file is missing from the chunk directory.
    #[error("missing chunk: {0}")]
    ChunkMissing(String),

    /// Destination does not have enough space for the operation.
    #[error("insufficient space: need {needed} bytes, only {available} available")]
    InsufficientSpace { needed: u64, available: u64 },

    /// A supplied path is invalid or inaccessible.
    #[error("invalid path: {0}")]
    InvalidPath(String),

    /// The user cancelled the operation (e.g. declined an overwrite prompt).
    #[error("operation cancelled by user")]
    UserAbort,

    /// An unsupported hash algorithm was requested.
    #[error("unsupported hash algorithm: {0}")]
    UnsupportedAlgorithm(String),
}
