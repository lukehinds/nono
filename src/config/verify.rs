//! Signature verification for security-critical configuration
//!
//! Uses minisign format for compatibility with existing tooling.
//! See: https://jedisct1.github.io/minisign/

#![allow(dead_code)]

use crate::error::{NonoError, Result};
use minisign_verify::{PublicKey, Signature};

/// Verify a minisign signature
///
/// # Arguments
/// * `data` - The data that was signed
/// * `signature` - The minisign signature (full .minisig file content)
/// * `public_key` - The public key in minisign format (starts with RW)
///
/// # Returns
/// * `Ok(())` if signature is valid
/// * `Err(NonoError::SignatureInvalid)` if verification fails
pub fn verify_minisign(data: &[u8], signature: &str, public_key: &str) -> Result<()> {
    // Parse the public key
    let pk = PublicKey::from_base64(public_key).map_err(|e| NonoError::SignatureInvalid {
        reason: format!("Invalid public key: {}", e),
    })?;

    // Parse the signature
    let sig = Signature::decode(signature).map_err(|e| NonoError::SignatureInvalid {
        reason: format!("Invalid signature format: {}", e),
    })?;

    // Verify
    pk.verify(data, &sig, false)
        .map_err(|e| NonoError::SignatureInvalid {
            reason: format!("Signature verification failed: {}", e),
        })?;

    Ok(())
}

/// Verify a minisign signature from file paths
#[allow(dead_code)]
pub fn verify_minisign_files(
    data_path: &std::path::Path,
    sig_path: &std::path::Path,
    public_key: &str,
) -> Result<()> {
    let data = std::fs::read(data_path).map_err(|e| NonoError::ConfigRead {
        path: data_path.to_path_buf(),
        source: e,
    })?;

    let signature = std::fs::read_to_string(sig_path).map_err(|e| NonoError::ConfigRead {
        path: sig_path.to_path_buf(),
        source: e,
    })?;

    verify_minisign(&data, &signature, public_key)
}

/// Extract trusted comment from a minisign signature
///
/// Trusted comments are verified as part of the signature and can contain
/// metadata like version numbers and timestamps.
#[allow(dead_code)]
pub fn extract_trusted_comment(signature: &str) -> Option<String> {
    // Minisign format:
    // Line 1: untrusted comment
    // Line 2: base64 signature
    // Line 3: trusted comment (starts with "trusted comment: ")
    // Line 4: base64 signature of trusted comment

    let lines: Vec<&str> = signature.lines().collect();
    if lines.len() >= 3 {
        let trusted_line = lines[2];
        if let Some(comment) = trusted_line.strip_prefix("trusted comment: ") {
            return Some(comment.to_string());
        }
    }
    None
}

/// Parse version and timestamp from a trusted comment
///
/// Expected format: "timestamp:1705312200 version:3"
#[allow(dead_code)]
pub fn parse_trusted_comment(comment: &str) -> (Option<i64>, Option<u64>) {
    let mut timestamp = None;
    let mut version = None;

    for part in comment.split_whitespace() {
        if let Some(ts) = part.strip_prefix("timestamp:") {
            timestamp = ts.parse().ok();
        } else if let Some(v) = part.strip_prefix("version:") {
            version = v.parse().ok();
        }
    }

    (timestamp, version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_trusted_comment() {
        let sig = r#"untrusted comment: nono security-lists v1
RWSomeBase64SignatureHere==
trusted comment: timestamp:1705312200 version:3
SomeBase64TrustedSigHere=="#;

        let comment = extract_trusted_comment(sig);
        assert_eq!(comment, Some("timestamp:1705312200 version:3".to_string()));
    }

    #[test]
    fn test_parse_trusted_comment() {
        let comment = "timestamp:1705312200 version:3";
        let (ts, ver) = parse_trusted_comment(comment);

        assert_eq!(ts, Some(1705312200));
        assert_eq!(ver, Some(3));
    }

    #[test]
    fn test_parse_trusted_comment_partial() {
        let comment = "version:5";
        let (ts, ver) = parse_trusted_comment(comment);

        assert_eq!(ts, None);
        assert_eq!(ver, Some(5));
    }
}
