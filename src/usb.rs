/// USB drive detection and filesystem utilities.
///
/// Provides platform-specific detection of removable media and a
/// cross-platform `sync_filesystem` call to ensure data is flushed
/// before USB removal.
use std::path::Path;

use crate::error::Result;

/// Information about a mounted drive (USB or otherwise).
#[derive(Debug, Clone)]
#[allow(dead_code)] // Used in later phases for USB drive listing.
pub struct DriveInfo {
    pub mount_point: std::path::PathBuf,
    pub total_bytes: u64,
    pub available_bytes: u64,
}

/// Query the available space at a filesystem path.
///
/// Works for any path — USB drives, local disks, network mounts, etc.
/// This is the primary capacity check used by the pack command.
#[allow(clippy::unnecessary_cast)] // f_bavail/f_frsize types differ across platforms
pub fn get_available_space(path: &Path) -> Result<u64> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let c_path =
            std::ffi::CString::new(path.as_os_str().as_bytes()).map_err(std::io::Error::other)?;
        unsafe {
            let mut stat: libc::statvfs = std::mem::zeroed();
            if libc::statvfs(c_path.as_ptr(), &mut stat) == 0 {
                Ok(stat.f_bavail as u64 * stat.f_frsize as u64)
            } else {
                Err(std::io::Error::last_os_error().into())
            }
        }
    }

    #[cfg(not(unix))]
    {
        // Fallback: assume unlimited space (Windows support in future phase)
        let _ = path;
        Ok(u64::MAX)
    }
}

/// Flush filesystem buffers to ensure data reaches physical media.
///
/// Call this before prompting the user to remove a USB drive.
pub fn sync_filesystem() -> Result<()> {
    #[cfg(unix)]
    {
        unsafe {
            libc::sync();
        }
    }
    Ok(())
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Spec: TC-PCK-003
    /// Query available space on a temp directory.
    #[test]
    fn available_space_is_nonzero() {
        let dir = tempfile::tempdir().unwrap();
        let space = get_available_space(dir.path()).unwrap();
        assert!(space > 0, "available space should be > 0");
    }

    /// Spec: TC-SAF-003
    /// sync_filesystem completes without error.
    #[test]
    fn sync_does_not_panic() {
        sync_filesystem().unwrap();
    }
}
