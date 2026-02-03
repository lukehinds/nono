use crate::capability::{CapabilitySet, FsAccess};
use crate::config;
use crate::error::{NonoError, Result};
use landlock::{
    Access, AccessFs, AccessNet, BitFlags, PathBeneath, PathFd, Ruleset, RulesetAttr,
    RulesetCreatedAttr, ABI,
};
use std::path::Path;
use tracing::{debug, info, warn};

/// The target ABI version we support (highest we know about)
const TARGET_ABI: ABI = ABI::V5;

/// Check if Landlock is supported on this system
pub fn is_supported() -> bool {
    // Try to create a minimal ruleset to check if Landlock is available
    Ruleset::default()
        .handle_access(AccessFs::from_all(TARGET_ABI))
        .and_then(|r| r.create())
        .is_ok()
}

/// Get information about Landlock support
pub fn support_info() -> String {
    // Try to create a ruleset and check the status
    match Ruleset::default()
        .handle_access(AccessFs::from_all(TARGET_ABI))
        .and_then(|r| r.create())
    {
        Ok(_) => format!("Landlock available (targeting ABI v{:?})", TARGET_ABI),
        Err(_) => {
            "Landlock not available. Requires Linux kernel 5.13+ with Landlock enabled.".to_string()
        }
    }
}

/// Convert FsAccess to Landlock AccessFs flags
/// Note: RemoveFile, RemoveDir, and Truncate are intentionally excluded
/// to prevent destructive operations even in allowed directories.
/// This is a defense-in-depth measure against accidental or malicious deletion.
fn access_to_landlock(access: FsAccess, _abi: ABI) -> BitFlags<AccessFs> {
    match access {
        FsAccess::Read => AccessFs::ReadFile | AccessFs::ReadDir | AccessFs::Execute,
        FsAccess::Write => {
            // Write access allows creating and modifying files, but NOT deleting or truncating.
            // This prevents destructive operations like `rm -rf` or truncating files to zero bytes.
            // Excluded operations:
            //   - RemoveFile: unlink syscall (file deletion)
            //   - RemoveDir: rmdir syscall (directory deletion)
            //   - Truncate: truncate/ftruncate syscalls (can zero out files)
            AccessFs::WriteFile
                | AccessFs::MakeChar
                | AccessFs::MakeDir
                | AccessFs::MakeReg
                | AccessFs::MakeSock
                | AccessFs::MakeFifo
                | AccessFs::MakeBlock
                | AccessFs::MakeSym
        }
        FsAccess::ReadWrite => {
            access_to_landlock(FsAccess::Read, _abi) | access_to_landlock(FsAccess::Write, _abi)
        }
    }
}

/// Apply Landlock sandbox with the given capabilities
pub fn apply(caps: &CapabilitySet) -> Result<()> {
    info!("Using Landlock ABI {:?}", TARGET_ABI);

    // Determine which access rights to handle based on ABI
    let handled_fs = AccessFs::from_all(TARGET_ABI);

    debug!("Handling filesystem access: {:?}", handled_fs);

    // Create the ruleset (Ruleset::default() auto-probes kernel support)
    // Start with filesystem access
    let ruleset_builder = Ruleset::default()
        .handle_access(handled_fs)
        .map_err(|e| NonoError::SandboxInit(format!("Failed to handle fs access: {}", e)))?;

    // Add network access handling if blocking network (ABI V4+ required)
    let ruleset_builder = if caps.net_block {
        let handled_net = AccessNet::from_all(TARGET_ABI);
        if !handled_net.is_empty() {
            debug!("Handling network access (blocking): {:?}", handled_net);
            ruleset_builder.handle_access(handled_net).map_err(|e| {
                NonoError::SandboxInit(format!("Failed to handle net access: {}", e))
            })?
        } else {
            warn!("Network blocking requested but kernel ABI doesn't support it (requires V4+)");
            ruleset_builder
        }
    } else {
        ruleset_builder
    };

    let mut ruleset = ruleset_builder
        .create()
        .map_err(|e| NonoError::SandboxInit(format!("Failed to create ruleset: {}", e)))?;

    // Add read+execute access to system paths needed for executables to run
    // These paths are loaded from the embedded security-lists.toml
    let read_access = access_to_landlock(FsAccess::Read, TARGET_ABI);
    let system_paths = config::get_system_read_paths();
    for path_str in &system_paths {
        let path = Path::new(path_str);
        if path.exists() {
            match PathFd::new(path) {
                Ok(path_fd) => {
                    debug!("Adding system read rule: {}", path_str);
                    // Some paths (device nodes, special filesystems) may fail with EBADFD
                    // Skip them gracefully - they're optional for most programs
                    match ruleset.add_rule(PathBeneath::new(path_fd, read_access)) {
                        Ok(r) => ruleset = r,
                        Err(e) => {
                            debug!("Skipping system path {} (cannot add rule: {})", path_str, e);
                        }
                    }
                }
                Err(e) => {
                    debug!("Skipping system path {} (cannot open: {})", path_str, e);
                }
            }
        } else {
            debug!("Skipping system path {} (does not exist)", path_str);
        }
    }

    // Add read access to current working directory
    // Many programs need cwd access to function properly
    if let Ok(cwd) = std::env::current_dir() {
        if cwd.exists() {
            match PathFd::new(&cwd) {
                Ok(path_fd) => {
                    debug!("Adding cwd read rule: {}", cwd.display());
                    match ruleset.add_rule(PathBeneath::new(path_fd, read_access)) {
                        Ok(r) => ruleset = r,
                        Err(e) => {
                            debug!("Skipping cwd {} (cannot add rule: {})", cwd.display(), e);
                        }
                    }
                }
                Err(e) => {
                    debug!("Skipping cwd {} (cannot open: {})", cwd.display(), e);
                }
            }
        }
    }

    // Add rules for each user-specified filesystem capability
    // These MUST succeed - user explicitly requested these capabilities
    // Failing silently would violate the principle of least surprise and fail-secure design
    for cap in &caps.fs {
        let access = access_to_landlock(cap.access, TARGET_ABI);

        debug!(
            "Adding rule: {} with access {:?}",
            cap.resolved.display(),
            access
        );

        let path_fd = PathFd::new(&cap.resolved)?;
        ruleset = ruleset
            .add_rule(PathBeneath::new(path_fd, access))
            .map_err(|e| {
                NonoError::SandboxInit(format!(
                    "Cannot add Landlock rule for {}: {} (filesystem may not support Landlock)",
                    cap.resolved.display(),
                    e
                ))
            })?;
    }

    // Apply the ruleset - THIS IS IRREVERSIBLE
    let status = ruleset
        .restrict_self()
        .map_err(|e| NonoError::SandboxInit(format!("Failed to restrict self: {}", e)))?;

    match status.ruleset {
        landlock::RulesetStatus::FullyEnforced => {
            info!("Landlock sandbox fully enforced");
        }
        landlock::RulesetStatus::PartiallyEnforced => {
            // This is normal - the kernel supports a subset of features we requested.
            // The sandbox is still active and enforcing restrictions.
            debug!("Landlock sandbox enforced in best-effort mode");
        }
        landlock::RulesetStatus::NotEnforced => {
            return Err(NonoError::SandboxInit(
                "Landlock sandbox was not enforced".to_string(),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported() {
        // This test will pass or fail depending on kernel version
        // Just verify it doesn't panic
        let _ = is_supported();
    }

    #[test]
    fn test_support_info() {
        let info = support_info();
        assert!(!info.is_empty());
    }

    #[test]
    fn test_access_conversion() {
        let abi = ABI::V3;

        let read = access_to_landlock(FsAccess::Read, abi);
        assert!(read.contains(AccessFs::ReadFile));
        assert!(!read.contains(AccessFs::WriteFile));

        let write = access_to_landlock(FsAccess::Write, abi);
        assert!(write.contains(AccessFs::WriteFile));
        assert!(!write.contains(AccessFs::ReadFile));
        // Verify destructive operations are NOT included
        assert!(!write.contains(AccessFs::RemoveFile));
        assert!(!write.contains(AccessFs::RemoveDir));

        let rw = access_to_landlock(FsAccess::ReadWrite, abi);
        assert!(rw.contains(AccessFs::ReadFile));
        assert!(rw.contains(AccessFs::WriteFile));
        // Verify destructive operations are NOT included in ReadWrite either
        assert!(!rw.contains(AccessFs::RemoveFile));
        assert!(!rw.contains(AccessFs::RemoveDir));
    }
}
