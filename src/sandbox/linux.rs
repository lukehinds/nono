use crate::capability::{CapabilitySet, FsAccess, FsCapability};
use crate::error::{NonoError, Result};
use landlock::{
    Access, AccessFs, PathBeneath, PathFd, Ruleset, RulesetAttr, RulesetCreatedAttr, ABI,
};
use tracing::{debug, info, warn};

/// Detect the best available Landlock ABI version
fn detect_abi() -> Option<ABI> {
    // Try ABIs from newest to oldest
    for abi in [ABI::V5, ABI::V4, ABI::V3, ABI::V2, ABI::V1] {
        if landlock::is_available(abi) {
            return Some(abi);
        }
    }
    None
}

/// Check if Landlock is supported on this system
pub fn is_supported() -> bool {
    detect_abi().is_some()
}

/// Get information about Landlock support
pub fn support_info() -> String {
    match detect_abi() {
        Some(abi) => {
            let version = match abi {
                ABI::V1 => "1 (kernel 5.13+)",
                ABI::V2 => "2 (kernel 5.19+)",
                ABI::V3 => "3 (kernel 6.2+)",
                ABI::V4 => "4 (kernel 6.7+, includes TCP)",
                ABI::V5 => "5 (kernel 6.10+)",
                _ => "unknown",
            };
            format!("Landlock ABI v{} available", version)
        }
        None => {
            "Landlock not available. Requires Linux kernel 5.13+ with Landlock enabled.".to_string()
        }
    }
}

/// Convert FsAccess to Landlock AccessFs flags
fn access_to_landlock(access: FsAccess, abi: ABI) -> AccessFs {
    match access {
        FsAccess::Read => AccessFs::ReadFile | AccessFs::ReadDir | AccessFs::Execute,
        FsAccess::Write => {
            let mut flags = AccessFs::WriteFile
                | AccessFs::RemoveFile
                | AccessFs::RemoveDir
                | AccessFs::MakeChar
                | AccessFs::MakeDir
                | AccessFs::MakeReg
                | AccessFs::MakeSock
                | AccessFs::MakeFifo
                | AccessFs::MakeBlock
                | AccessFs::MakeSym;

            // Add truncate if available (ABI v3+)
            if abi >= ABI::V3 {
                flags |= AccessFs::Truncate;
            }

            flags
        }
        FsAccess::ReadWrite => {
            access_to_landlock(FsAccess::Read, abi) | access_to_landlock(FsAccess::Write, abi)
        }
    }
}

/// Apply Landlock sandbox with the given capabilities
pub fn apply(caps: &CapabilitySet) -> Result<()> {
    let abi = detect_abi().ok_or_else(|| {
        NonoError::SandboxInit(
            "Landlock not available. Requires Linux kernel 5.13+ with Landlock enabled."
                .to_string(),
        )
    })?;

    info!("Using Landlock ABI {:?}", abi);

    // Determine which access rights to handle based on ABI
    let handled_fs = AccessFs::from_all(abi);

    debug!("Handling filesystem access: {:?}", handled_fs);

    // Create the ruleset
    let mut ruleset = Ruleset::default()
        .handle_access(handled_fs)
        .map_err(|e| NonoError::SandboxInit(format!("Failed to create ruleset: {}", e)))?
        .create()
        .map_err(NonoError::LandlockCreate)?;

    // Add rules for each filesystem capability
    for cap in &caps.fs {
        let access = access_to_landlock(cap.access, abi);

        debug!(
            "Adding rule: {} with access {:?}",
            cap.resolved.display(),
            access
        );

        let path_fd = PathFd::new(&cap.resolved)?;
        ruleset = ruleset.add_rule(PathBeneath::new(path_fd, access))?;
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
            warn!("Landlock sandbox only partially enforced (kernel may lack some features)");
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
    fn test_detect_abi() {
        // This test will pass or fail depending on kernel version
        // Just verify it doesn't panic
        let _ = detect_abi();
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

        let rw = access_to_landlock(FsAccess::ReadWrite, abi);
        assert!(rw.contains(AccessFs::ReadFile));
        assert!(rw.contains(AccessFs::WriteFile));
    }
}
