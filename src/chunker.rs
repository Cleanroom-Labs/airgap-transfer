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

/// Convenience wrapper — pack from the start with no per-chunk callback.
///
/// Used by tests and simple callers that don't need resume or USB-swap
/// prompting.
#[cfg(test)]
pub fn pack_to_chunks(
    source: &Path,
    dest: &Path,
    chunk_size: u64,
    algorithm: &dyn HashAlgorithm,
    manifest: &mut Manifest,
    progress: &TransferProgress,
) -> Result<()> {
    pack_to_chunks_with_callback(
        source,
        dest,
        chunk_size,
        algorithm,
        manifest,
        progress,
        0,
        |_, _| Ok(()),
    )
}

/// Pack source path into tar chunks at the destination directory.
///
/// Each chunk is written as `chunk_XXX.tar`.  The manifest is updated with
/// the actual size and checksum of each chunk after it is written.
///
/// The source can be a single file or a directory tree.
///
/// When `resume_from` is greater than zero, chunks before that index are
/// simulated (file sizes are tracked to maintain correct chunk boundaries)
/// but no data is written to disk, allowing an interrupted pack to resume.
///
/// The `on_chunk_start` callback is invoked before each chunk (at or after
/// `resume_from`) begins writing.  It receives the chunk index and
/// destination path, and can be used to check available space, prompt for
/// USB swapping, or save the manifest.
#[allow(clippy::too_many_arguments)]
pub fn pack_to_chunks_with_callback(
    source: &Path,
    dest: &Path,
    chunk_size: u64,
    algorithm: &dyn HashAlgorithm,
    manifest: &mut Manifest,
    progress: &TransferProgress,
    resume_from: usize,
    mut on_chunk_start: impl FnMut(usize, &Path) -> Result<()>,
) -> Result<()> {
    fs::create_dir_all(dest)?;

    // Collect the list of files to pack (single file or directory walk).
    let entries = collect_entries(source)?;

    // Track how many bytes have been written to the current chunk.
    let mut current_chunk_index: usize = 0;
    let mut current_chunk_bytes: u64 = 0;
    let skipping = resume_from > 0;

    // Only open file/hasher when we're writing (not skipping)
    let mut hasher: Option<Box<dyn crate::verifier::HashWriter>> = None;
    let mut chunk_file: Option<fs::File> = None;

    if current_chunk_index >= resume_from {
        on_chunk_start(current_chunk_index, dest)?;
        // Delete partial chunk file if present
        let chunk_path = dest.join(&manifest.chunks[current_chunk_index].filename);
        let _ = fs::remove_file(&chunk_path);
        chunk_file = Some(fs::File::create(&chunk_path)?);
        hasher = Some(algorithm.create_writer());
        manifest.update_chunk(current_chunk_index, ChunkStatus::InProgress, 0, "");
    }

    for entry_path in &entries {
        let relative = entry_path
            .strip_prefix(source.parent().unwrap_or(source))
            .unwrap_or(entry_path);

        let metadata = fs::metadata(entry_path)?;
        let file_size = metadata.len();

        if current_chunk_index < resume_from {
            // Simulate: track sizes without writing
            let header_size = 512u64; // tar header is always 512 bytes
            current_chunk_bytes += header_size;
            current_chunk_bytes += file_size;
            let remainder = (file_size % 512) as u64;
            if remainder > 0 {
                current_chunk_bytes += 512 - remainder;
            }
        } else {
            // Write tar header
            let header_bytes = build_tar_header(relative, file_size)?;
            write_to_chunk(
                &header_bytes,
                chunk_file.as_mut().unwrap(),
                hasher.as_mut().unwrap(),
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
                    chunk_file.as_mut().unwrap(),
                    hasher.as_mut().unwrap(),
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
                    chunk_file.as_mut().unwrap(),
                    hasher.as_mut().unwrap(),
                    &mut current_chunk_bytes,
                )?;
            }
        }

        // Check if we should start a new chunk (if we've exceeded the target size
        // and there are more files to write)
        if current_chunk_bytes >= chunk_size && current_chunk_index + 1 < manifest.chunk_count {
            if current_chunk_index >= resume_from {
                // Finalize current chunk with two 512-byte zero blocks (tar EOF)
                let eof_block = [0u8; 1024];
                write_to_chunk(
                    &eof_block,
                    chunk_file.as_mut().unwrap(),
                    hasher.as_mut().unwrap(),
                    &mut current_chunk_bytes,
                )?;

                let checksum = hasher.take().unwrap().finalize();
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
            } else if skipping {
                progress.verbose_message(&format!(
                    "  chunk_{:03}.tar: skipped (already completed)",
                    current_chunk_index
                ));
            }

            // Start next chunk
            current_chunk_index += 1;
            current_chunk_bytes = 0;

            if current_chunk_index >= resume_from {
                on_chunk_start(current_chunk_index, dest)?;
                // Delete partial chunk file if present
                let next_path = dest.join(&manifest.chunks[current_chunk_index].filename);
                let _ = fs::remove_file(&next_path);
                chunk_file = Some(fs::File::create(&next_path)?);
                hasher = Some(algorithm.create_writer());
                manifest.update_chunk(current_chunk_index, ChunkStatus::InProgress, 0, "");
            }
        }
    }

    // Finalize the last chunk
    if current_chunk_index >= resume_from {
        let eof_block = [0u8; 1024];
        write_to_chunk(
            &eof_block,
            chunk_file.as_mut().unwrap(),
            hasher.as_mut().unwrap(),
            &mut current_chunk_bytes,
        )?;
        drop(chunk_file);

        let checksum = hasher.take().unwrap().finalize();
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
    }

    // If we created fewer chunks than initially estimated, truncate the manifest
    manifest.chunk_count = current_chunk_index + 1;
    manifest.chunks.truncate(manifest.chunk_count);

    Ok(())
}

/// Extract tar chunks back to a destination directory.
///
/// Iterates chunks in manifest order, opening each as a tar archive and
/// extracting all entries to `dest`.  The `tar` crate handles directory
/// creation and path joining.
pub fn unpack_from_chunks(
    source_dir: &Path,
    dest: &Path,
    manifest: &Manifest,
    progress: &TransferProgress,
) -> Result<()> {
    fs::create_dir_all(dest)?;

    for chunk in &manifest.chunks {
        let chunk_path = source_dir.join(&chunk.filename);
        let file = fs::File::open(&chunk_path)
            .map_err(|e| AirgapError::ChunkMissing(format!("{}: {e}", chunk.filename)))?;

        let mut archive = tar::Archive::new(file);
        for entry in archive.entries()? {
            let mut entry = entry?;
            entry.unpack_in(dest)?;
        }

        progress.advance(chunk.size_bytes);
        progress.verbose_message(&format!("  extracted {}", chunk.filename));
    }

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

/// Compute a checksum over all source file contents in deterministic order.
///
/// Files are enumerated in the same sorted order used by packing, and their
/// raw content is fed into a single running hash.  This produces a
/// whole-source fingerprint that can be compared after unpacking to verify
/// that the reconstructed output is bit-identical to the original.
pub fn compute_source_checksum(
    source: &Path,
    algorithm: &dyn crate::verifier::HashAlgorithm,
) -> Result<String> {
    let entries = collect_entries(source)?;
    let mut writer = algorithm.create_writer();
    let mut buf = [0u8; BUF_SIZE];
    for entry_path in &entries {
        let mut file = fs::File::open(entry_path)?;
        loop {
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            writer.update(&buf[..n]);
        }
    }
    Ok(writer.finalize())
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

    // ── Unpack tests ──────────────────────────────────────────────────

    /// Helper: pack a source into chunks, save manifest, return (chunk_dir, manifest).
    fn pack_roundtrip(
        src_dir: &Path,
        source: &Path,
        chunk_size: u64,
    ) -> (tempfile::TempDir, Manifest) {
        let dest_dir = tempfile::tempdir().unwrap();
        let total = calculate_total_size(source).unwrap();
        let mut manifest =
            Manifest::new_pack(source.to_str().unwrap(), total, chunk_size, "sha256");
        let progress = TransferProgress::hidden();
        pack_to_chunks(
            source,
            dest_dir.path(),
            chunk_size,
            &Sha256Algorithm,
            &mut manifest,
            &progress,
        )
        .unwrap();
        let _ = src_dir; // keep alive
        (dest_dir, manifest)
    }

    /// TC-UNP-001: Pack then unpack a directory, verify files match originals.
    #[test]
    fn unpack_roundtrip_directory() {
        let src_dir = tempfile::tempdir().unwrap();
        make_test_file(src_dir.path(), "a.txt", b"File A content");
        make_test_file(src_dir.path(), "b.txt", b"File B content here");
        make_test_file(src_dir.path(), "sub/c.txt", b"Nested file C");

        let (chunk_dir, manifest) = pack_roundtrip(src_dir.path(), src_dir.path(), 1_000_000);

        // Unpack to a new directory
        let unpack_dir = tempfile::tempdir().unwrap();
        let progress = TransferProgress::hidden();
        unpack_from_chunks(chunk_dir.path(), unpack_dir.path(), &manifest, &progress).unwrap();

        // Verify file contents match. The tar entries include the source dir name
        // as a path prefix, so we need to find the files relative to the unpack root.
        let find_file = |name: &str| -> Vec<u8> {
            for entry in walkdir(unpack_dir.path()) {
                if entry.ends_with(name) {
                    return fs::read(&entry).unwrap();
                }
            }
            panic!("file {name} not found in unpacked output");
        };

        assert_eq!(find_file("a.txt"), b"File A content");
        assert_eq!(find_file("b.txt"), b"File B content here");
        assert_eq!(find_file("c.txt"), b"Nested file C");
    }

    /// TC-UNP-001 (single file variant): Pack and unpack a single file.
    #[test]
    fn unpack_roundtrip_single_file() {
        let src_dir = tempfile::tempdir().unwrap();
        let content = b"Hello from the air-gapped world!";
        make_test_file(src_dir.path(), "hello.txt", content);

        let source = src_dir.path().join("hello.txt");
        let (chunk_dir, manifest) = pack_roundtrip(src_dir.path(), &source, 1_000_000);

        let unpack_dir = tempfile::tempdir().unwrap();
        let progress = TransferProgress::hidden();
        unpack_from_chunks(chunk_dir.path(), unpack_dir.path(), &manifest, &progress).unwrap();

        let find_file = |name: &str| -> Vec<u8> {
            for entry in walkdir(unpack_dir.path()) {
                if entry.ends_with(name) {
                    return fs::read(&entry).unwrap();
                }
            }
            panic!("file {name} not found");
        };

        assert_eq!(find_file("hello.txt"), content);
    }

    /// TC-UNP-004: Missing chunk file returns ChunkMissing error.
    #[test]
    fn unpack_missing_chunk_errors() {
        let src_dir = tempfile::tempdir().unwrap();
        make_test_file(src_dir.path(), "data.txt", b"test");

        let source = src_dir.path().join("data.txt");
        let (chunk_dir, manifest) = pack_roundtrip(src_dir.path(), &source, 1_000_000);

        // Delete the chunk file
        fs::remove_file(chunk_dir.path().join("chunk_000.tar")).unwrap();

        let unpack_dir = tempfile::tempdir().unwrap();
        let progress = TransferProgress::hidden();
        let result = unpack_from_chunks(chunk_dir.path(), unpack_dir.path(), &manifest, &progress);
        assert!(result.is_err());
    }

    /// TC-UNP-001 (multi-chunk): Pack with small chunks, unpack, verify content.
    #[test]
    fn unpack_multi_chunk_roundtrip() {
        let src_dir = tempfile::tempdir().unwrap();
        let data = vec![0xABu8; 4096];
        make_test_file(src_dir.path(), "big.bin", &data);

        let source = src_dir.path().join("big.bin");
        let (chunk_dir, manifest) = pack_roundtrip(src_dir.path(), &source, 1024);
        assert!(manifest.chunk_count >= 2, "should have multiple chunks");

        let unpack_dir = tempfile::tempdir().unwrap();
        let progress = TransferProgress::hidden();
        unpack_from_chunks(chunk_dir.path(), unpack_dir.path(), &manifest, &progress).unwrap();

        let find_file = |name: &str| -> Vec<u8> {
            for entry in walkdir(unpack_dir.path()) {
                if entry.ends_with(name) {
                    return fs::read(&entry).unwrap();
                }
            }
            panic!("file {name} not found");
        };

        assert_eq!(find_file("big.bin"), data);
    }

    /// Helper: recursively collect all file paths under a directory.
    fn walkdir(dir: &Path) -> Vec<std::path::PathBuf> {
        let mut files = Vec::new();
        walk_dir_test(dir, &mut files);
        files
    }

    fn walk_dir_test(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk_dir_test(&path, files);
                } else {
                    files.push(path);
                }
            }
        }
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
