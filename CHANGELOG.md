# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Security
- **Fixed macOS Seatbelt allowing all file reads** - Removed blanket `(allow file-read*)` that made `--allow` ineffective. Now uses explicit allow-listing matching Linux Landlock behavior.
- Implemented "Allow Discovery, Deny Content" strategy for sensitive paths:
  - `file-read-metadata` allowed (stat, existence checks)
  - `file-read-data` denied (actual content reads blocked)
- Narrowed `system*` permissions to only `system-socket`, `system-fsctl`, `system-info`
- Added `file-map-executable` permission (required for dyld)
- Expanded sensitive path protection:
  - Browser data (Chrome, Firefox, Safari, Edge, Arc, Brave)
  - macOS private data (Messages, Mail, Cookies, MobileSync)
  - Password managers (Keychains, 1Password, pass)
  - Shell configs and history files

### Changed
- Expanded `security-lists.toml` with comprehensive macOS system paths (dyld, ssl, locale, terminfo, system, user_library)
- Improved Seatbelt documentation with technical clarifications:
  - Added note about private `sandbox_init()` API stability
  - Clarified network filtering capabilities (IP filtering possible, domain filtering not)
  - Explained APFS firmlinks and path resolution on macOS 10.15+
  - Added common debugging issues table

## [0.1.0] - 2025-02-01

### Added
- Filesystem sandboxing with Landlock (Linux) and Seatbelt (macOS)
- Network blocking capability (`--net-block`)
- Profile system for reusable configurations
- `nono run` command for executing sandboxed processes
- `nono why` command for debugging path access
- `nono setup` command for system configuration
- Sensitive path protection (SSH keys, credentials, shell configs)
- Silent mode (`--silent`) for scripting
- Dry-run mode for testing configurations

### Security
- Blocks access to ~/.ssh, ~/.aws, ~/.gnupg, and other sensitive paths by default
- Uses OS-enforced sandboxing (no escape hatch)
- All child processes inherit sandbox restrictions

[Unreleased]: https://github.com/lukehinds/nono/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/lukehinds/nono/releases/tag/v0.1.0
