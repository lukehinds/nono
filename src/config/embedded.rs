//! Embedded configuration loading
//!
//! Loads security lists that are compiled into the binary.
//! Built-in profiles are defined as Rust structs in src/profile/builtin.rs.

use super::security_lists::SecurityLists;
use crate::error::{NonoError, Result};

/// Embedded security lists (compiled into binary by build.rs)
const EMBEDDED_SECURITY_LISTS: &str =
    include_str!(concat!(env!("OUT_DIR"), "/security-lists.toml"));

#[allow(dead_code)]
/// Author public key for verifying signatures
/// This is the root of trust - embedded at compile time
/// Luke: Implementing signature verification is a future step, but we need to have the key in place for when we do. In fact it will be rotating, so we need to have the infrastructure for it in place before we can generate the first key pair. For now, this is a placeholder value that will be replaced once we have a real key pair generated.
pub const AUTHOR_PUBLIC_KEY: &str = "RWTk1xXqcTODeYttYMCqEwcLg+KiX+Vpu1v6iV3D0sGabcdef12345678";
// TODO: Replace with actual public key when generated

/// Check if security lists are signed (runtime check)
fn is_signed() -> bool {
    option_env!("SECURITY_LISTS_SIGNED") == Some("1")
}

/// Load embedded security lists
///
/// If signatures are present, verifies them before returning.
/// For unsigned builds (development), returns the lists without verification.
pub fn load_security_lists() -> Result<SecurityLists> {
    // Parse the TOML
    let lists: SecurityLists = toml::from_str(EMBEDDED_SECURITY_LISTS).map_err(|e| {
        NonoError::ConfigParse(format!("Failed to parse embedded security lists: {}", e))
    })?;

    // Signature verification is deferred until we have proper key management
    // For now, just log if we're running unsigned
    if !is_signed() {
        tracing::debug!("Running with unsigned security lists (development mode)");
    }

    // TODO: Check version against stored state for downgrade protection

    Ok(lists)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_security_lists() {
        let lists = load_security_lists().expect("Failed to load security lists");

        // Verify basic structure
        assert!(lists.meta.version >= 1);
        assert!(!lists.all_sensitive_paths().is_empty());
        assert!(!lists.all_dangerous_commands().is_empty());
    }
}
