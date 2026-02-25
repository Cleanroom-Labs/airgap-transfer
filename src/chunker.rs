/// Streaming chunk creation for pack operations.
///
/// Splits source files into `chunk_XXX.tar` archives.  Each chunk is a
/// self-contained tar file containing as many source entries as fit within
/// the configured chunk size.  Hashes are computed inline during writes
/// so there is no second pass over the data.
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use crate::error::{AirgapError, Result};
use crate::manifest::{ChunkStatus, Manifest};
use crate::progress::TransferProgress;
use crate::verifier::HashAlgorithm;

/// Read buffer size (8 KiB) — keeps memory usage well under the 100 MB budget.
const BUF_SIZE: usize = 8192;

/// Pack source path into tar chunks at the destination directory.
///
/// Each chunk is written as `chunk_XXX.tar`.  The manifest is updated with
/// the actual size and checksum of each chunk after it is written.
///
/// The source can be a single file or a directory tree.
pub fn pack_to_chunks(
    source: &Path,
    dest: &Path,
    chunk_size: u64,
    algorithm: &dyn HashAlgorithm,
    manifest: &mut Manifest,
    progress: &TransferProgress,
) -> Result<()> {
    fs::create_dir_all(dest)?;

    // Collect the list of files to pack (single file or directory walk).
    let entries = collect_entries(source)?;

    // Track how many bytes have been written to the current chunk.
    let mut current_chunk_index: usize = 0;
    let mut current_chunk_bytes: u64 = 0;
    let mut hasher = algorithm.create_writer();
    let chunk_path = dest.join(&manifest.chunks[current_chunk_index].filename);
    let mut chunk_file = fs::File::create(&chunk_path)?;

    manifest.update_chunk(current_chunk_index, ChunkStatus::InProgress, 0, "");

    for entry_path in &entries {
        let relative = entry_path
            .strip_prefix(source.parent().unwrap_or(source))
            .unwrap_or(entry_path);

        let metadata = fs::metadata(entry_path)?;
        let file_size = metadata.len();

        // Write tar header
        let header_bytes = build_tar_header(relative, file_size)?;
        write_to_chunk(
            &header_bytes,
            &mut chunk_file,
            &mut hasher,
            &mut current_chunk_bytes,
        )?;
        progress.advance(header_bytes.len() as u64);

        // Stream file content in BUF_SIZE blocks
        let mut source_file = fs::File::open(entry_path)?;
        let mut buf = [0u8; BUF_SIZE];
        loop {
            let n = source_file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            write_to_chunk(
                &buf[..n],
                &mut chunk_file,
                &mut hasher,
                &mut current_chunk_bytes,
            )?;
            progress.advance(n as u64);
        }

        // Tar entries are padded to 512-byte boundaries
        let remainder = (file_size % 512) as usize;
        if remainder > 0 {
            let padding = vec![0u8; 512 - remainder];
            write_to_chunk(
                &padding,
                &mut chunk_file,
                &mut hasher,
                &mut current_chunk_bytes,
            )?;
        }

        // Check if we should start a new chunk (if we've exceeded the target size
        // and there are more files to write)
        if current_chunk_bytes >= chunk_size && current_chunk_index + 1 < manifest.chunk_count {
            // Finalize current chunk with two 512-byte zero blocks (tar EOF)
            let eof_block = [0u8; 1024];
            write_to_chunk(
                &eof_block,
                &mut chunk_file,
                &mut hasher,
                &mut current_chunk_bytes,
            )?;

            let checksum = hasher.finalize();
            manifest.update_chunk(
                current_chunk_index,
                ChunkStatus::Completed,
                current_chunk_bytes,
                &checksum,
            );

            progress.verbose_message(&format!(
                "  chunk_{:03}.tar: {} bytes, {}",
                current_chunk_index, current_chunk_bytes, checksum
            ));

            // Start next chunk
            current_chunk_index += 1;
            current_chunk_bytes = 0;
            hasher = algorithm.create_writer();
            let next_path = dest.join(&manifest.chunks[current_chunk_index].filename);
            chunk_file = fs::File::create(&next_path)?;
            manifest.update_chunk(current_chunk_index, ChunkStatus::InProgress, 0, "");
        }
    }

    // Finalize the last chunk with tar EOF
    let eof_block = [0u8; 1024];
    write_to_chunk(
        &eof_block,
        &mut chunk_file,
        &mut hasher,
        &mut current_chunk_bytes,
    )?;
    drop(chunk_file);

    let checksum = hasher.finalize();
    manifest.update_chunk(
        current_chunk_index,
        ChunkStatus::Completed,
        current_chunk_bytes,
        &checksum,
    );
    progress.verbose_message(&format!(
        "  chunk_{:03}.tar: {} bytes, {}",
        current_chunk_index, current_chunk_bytes, checksum
    ));

    // If we created fewer chunks than initially estimated, truncate the manifest
    manifest.chunk_count = current_chunk_index + 1;
    manifest.chunks.truncate(manifest.chunk_count);

    Ok(())
}

/// Calculate total size of all files to be packed (for progress display).
pub fn calculate_total_size(source: &Path) -> Result<u64> {
    let entries = collect_entries(source)?;
    let mut total = 0u64;
    for entry in &entries {
        total += fs::metadata(entry)?.len();
    }
    Ok(total)
}

// ── Internal helpers ────────────────────────────────────────────────────

/// Write data to the current chunk file and the running hasher.
fn write_to_chunk(
    data: &[u8],
    file: &mut fs::File,
    hasher: &mut Box<dyn crate::verifier::HashWriter>,
    bytes_written: &mut u64,
) -> Result<()> {
    file.write_all(data)?;
    hasher.update(data);
    *bytes_written += data.len() as u64;
    Ok(())
}

/// Collect all regular files under `source` (or just `source` if it's a file).
fn collect_entries(source: &Path) -> Result<Vec<std::path::PathBuf>> {
    if !source.exists() {
        return Err(AirgapError::InvalidPath(format!(
            "source path does not exist: {}",
            source.display()
        )));
    }

    let mut entries = Vec::new();
    if source.is_file() {
        entries.push(source.to_path_buf());
    } else if source.is_dir() {
        walk_dir(source, &mut entries)?;
        entries.sort(); // deterministic ordering
    }
    Ok(entries)
}

/// Recursively walk a directory, collecting regular file paths.
fn walk_dir(dir: &Path, entries: &mut Vec<std::path::PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_dir(&path, entries)?;
        } else if path.is_file() {
            entries.push(path);
        }
    }
    Ok(())
}

/// Build a POSIX tar header for a file entry.
///
/// Returns a 512-byte header block.
fn build_tar_header(relative_path: &Path, file_size: u64) -> Result<Vec<u8>> {
    let mut header = tar::Header::new_gnu();
    header.set_path(relative_path).map_err(|e| {
        AirgapError::InvalidPath(format!(
            "path too long for tar: {}: {e}",
            relative_path.display()
        ))
    })?;
    header.set_size(file_size);
    header.set_mode(0o644);
    header.set_mtime(0);
    header.set_entry_type(tar::EntryType::Regular);
    header.set_cksum();

    Ok(header.as_bytes().to_vec())
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Manifest;
    use crate::verifier::Sha256Algorithm;

    fn make_test_file(dir: &Path, name: &str, content: &[u8]) {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
    }

    /// TC-PCK-001: Pack a single file into chunks, verify chunk creation.
    #[test]
    fn pack_single_file() {
        let src_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();

        make_test_file(src_dir.path(), "hello.txt", b"Hello, air-gapped world!");

        let source = src_dir.path().join("hello.txt");
        let total = calculate_total_size(&source).unwrap();
        let mut manifest = Manifest::new_pack(
            source.to_str().unwrap(),
            total,
            1_000_000, // 1 MB chunk — way bigger than our test file
            "sha256",
        );

        let progress = TransferProgress::hidden();
        pack_to_chunks(
            &source,
            dest_dir.path(),
            1_000_000,
            &Sha256Algorithm,
            &mut manifest,
            &progress,
        )
        .unwrap();

        // Should create chunk_000.tar
        assert!(dest_dir.path().join("chunk_000.tar").exists());
        assert_eq!(manifest.chunk_count, 1);
        assert_eq!(manifest.chunks[0].status, ChunkStatus::Completed);
        assert!(manifest.chunks[0].checksum.starts_with("sha256:"));
        assert!(manifest.chunks[0].size_bytes > 0);
    }

    /// TC-PCK-002: Pack a directory with multiple files.
    #[test]
    fn pack_directory() {
        let src_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();

        make_test_file(src_dir.path(), "a.txt", b"File A content");
        make_test_file(src_dir.path(), "b.txt", b"File B content here");
        make_test_file(src_dir.path(), "sub/c.txt", b"Nested file C");

        let total = calculate_total_size(src_dir.path()).unwrap();
        let mut manifest =
            Manifest::new_pack(src_dir.path().to_str().unwrap(), total, 1_000_000, "sha256");

        let progress = TransferProgress::hidden();
        pack_to_chunks(
            src_dir.path(),
            dest_dir.path(),
            1_000_000,
            &Sha256Algorithm,
            &mut manifest,
            &progress,
        )
        .unwrap();

        assert!(dest_dir.path().join("chunk_000.tar").exists());
        assert_eq!(manifest.chunks[0].status, ChunkStatus::Completed);
    }

    /// TC-PCK-007: Chunk size flag produces correct number of chunks.
    #[test]
    fn small_chunk_size_splits_data() {
        let src_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();

        // Create a file large enough to span multiple chunks at 1024 byte chunk size
        let data = vec![0x42u8; 4096];
        make_test_file(src_dir.path(), "big.bin", &data);

        let source = src_dir.path().join("big.bin");
        let total = calculate_total_size(&source).unwrap();
        let chunk_size = 1024u64;
        let mut manifest =
            Manifest::new_pack(source.to_str().unwrap(), total, chunk_size, "sha256");

        let progress = TransferProgress::hidden();
        pack_to_chunks(
            &source,
            dest_dir.path(),
            chunk_size,
            &Sha256Algorithm,
            &mut manifest,
            &progress,
        )
        .unwrap();

        // With a 4096-byte file and 1024-byte chunk target, we expect multiple chunks.
        // The tar header adds 512 bytes, so the first chunk will exceed 1024 bytes
        // during the first file write, triggering a split.
        assert!(
            manifest.chunk_count >= 2,
            "expected multiple chunks, got {}",
            manifest.chunk_count
        );

        // All completed
        for chunk in &manifest.chunks {
            assert_eq!(chunk.status, ChunkStatus::Completed);
            assert!(chunk.checksum.starts_with("sha256:"));
        }
    }

    /// Chunks are valid tar files that can be read back.
    #[test]
    fn chunks_are_valid_tar() {
        let src_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();

        let content = b"This is the test content for tar validation";
        make_test_file(src_dir.path(), "test.txt", content);

        let source = src_dir.path().join("test.txt");
        let total = calculate_total_size(&source).unwrap();
        let mut manifest = Manifest::new_pack(source.to_str().unwrap(), total, 1_000_000, "sha256");

        let progress = TransferProgress::hidden();
        pack_to_chunks(
            &source,
            dest_dir.path(),
            1_000_000,
            &Sha256Algorithm,
            &mut manifest,
            &progress,
        )
        .unwrap();

        // Read back the tar and verify the entry
        let chunk_path = dest_dir.path().join("chunk_000.tar");
        let file = fs::File::open(&chunk_path).unwrap();
        let mut archive = tar::Archive::new(file);
        let entries: Vec<_> = archive.entries().unwrap().collect();
        assert!(!entries.is_empty(), "tar should have at least one entry");
    }

    /// Checksum in manifest matches independently computed checksum.
    #[test]
    fn manifest_checksum_matches_file() {
        let src_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();

        make_test_file(src_dir.path(), "data.bin", b"checksum test data");

        let source = src_dir.path().join("data.bin");
        let total = calculate_total_size(&source).unwrap();
        let mut manifest = Manifest::new_pack(source.to_str().unwrap(), total, 1_000_000, "sha256");

        let progress = TransferProgress::hidden();
        pack_to_chunks(
            &source,
            dest_dir.path(),
            1_000_000,
            &Sha256Algorithm,
            &mut manifest,
            &progress,
        )
        .unwrap();

        // Independently compute the checksum of the written chunk
        let chunk_path = dest_dir.path().join("chunk_000.tar");
        let independent = crate::verifier::compute_checksum(&chunk_path, &Sha256Algorithm).unwrap();
        assert_eq!(manifest.chunks[0].checksum, independent);
    }

    /// Source path that doesn't exist returns an error.
    #[test]
    fn nonexistent_source_errors() {
        let dest_dir = tempfile::tempdir().unwrap();
        let bad_source = Path::new("/nonexistent/path/to/file.txt");
        let mut manifest = Manifest::new_pack("/nonexistent", 100, 100, "sha256");
        let progress = TransferProgress::hidden();

        let result = pack_to_chunks(
            bad_source,
            dest_dir.path(),
            100,
            &Sha256Algorithm,
            &mut manifest,
            &progress,
        );
        assert!(result.is_err());
    }

    /// TC-PCK-006: No temporary files created during streaming.
    #[test]
    fn no_temp_files() {
        let src_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();

        make_test_file(src_dir.path(), "file.txt", b"stream test");

        let source = src_dir.path().join("file.txt");
        let total = calculate_total_size(&source).unwrap();
        let mut manifest = Manifest::new_pack(source.to_str().unwrap(), total, 1_000_000, "sha256");

        let progress = TransferProgress::hidden();
        pack_to_chunks(
            &source,
            dest_dir.path(),
            1_000_000,
            &Sha256Algorithm,
            &mut manifest,
            &progress,
        )
        .unwrap();

        // Only chunk files should exist in dest — no .tmp or other intermediates
        let files: Vec<_> = fs::read_dir(dest_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();

        for f in &files {
            assert!(
                f.starts_with("chunk_") && f.ends_with(".tar"),
                "unexpected file in dest: {f}"
            );
        }
    }
}
