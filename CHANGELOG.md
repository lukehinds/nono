# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2] - 2026-02-03

### Changed

- Merge pull request #25 from lukehinds/usr-local-fix

Usr local fix
- Rem broken image
- Add /nix
- Merge pull request #26 from lukehinds/bug-19

Add /nix
- Deny process-info for other processes to prevent info leaks

The Seatbelt profile used (allow process*) could allow sandboxed
processes to inspect other processes via ps aux, top, etc. This could
leak secrets from command-line arguments of other programs running as
the same user.

Replace blanket process* with targeted permissions:
- process-exec* and process-fork for execution
- process-info* (target self) for dyld/code signing
- process-info* (target others) DENIED to block inspection

Signed-off-by: Luke Hinds <lukehinds@gmail.com>
- Assertions to include \n for exact rule matching
- Merge pull request #28 from lukehinds/process-hardening

Deny process-info for other processes to prevent info leaks
- Revise warning for early alpha release

Updated warning message to clarify the alpha release status and encourage bug reporting.
- Add integration test suite and development documentation

Introduce comprehensive shell-based integration tests that build nono
and verify sandbox behavior from the user's perspective. Tests cover:

- Filesystem access control (read/write permissions, single file grants)
- Sensitive path protection (~/.ssh, ~/.aws, ~/.gnupg, etc.)
- System path protection (/usr/bin, /etc, /System/Library)
- Binary execution (shells, text tools, language runtimes)
- Network blocking (--net-block flag)
- Dangerous command blocking (rm, chmod, sudo, pip, etc.)
- Edge cases (symlinks, path variations, env vars, dry-run mode)

Tests gracefully handle platform differences (macOS TMPDIR behavior,
Homebrew-installed runtimes) with skip functionality.

Also adds Development section to documentation with contributor guide
and detailed testing documentation.

New files:
- tests/lib/test_helpers.sh
- tests/run_integration_tests.sh
- tests/integration/test_*.sh (7 test suites)
- docs/development/index.mdx
- docs/development/testing.mdx

Modified:
- .github/workflows/ci.yml (add integration job)
- docs/docs.json (add Development navigation)

Signed-off-by: Luke Hinds <lukehinds@gmail.com>
- Fix Linux Landlock sandbox for Ubuntu 24.04 and address review comments

The integration tests were failing on ubuntu-latest because the Landlock
sandbox was missing paths required for program execution on modern
Debian/Ubuntu systems.

Linux Landlock fixes:
- Add multiarch library paths (/lib/x86_64-linux-gnu, /usr/lib/x86_64-linux-gnu,
  and aarch64 variants) required by dynamically linked binaries
- Add /tmp and /var/tmp to system read paths for programs that need temp access
- Automatically grant read access to current working directory, as many
  programs need cwd access to function properly

Review comment fixes (gemini-code-assist):
- Replace manual test blocks with run_test helper function in:
  - test_commands.sh (2 instances)
  - test_fs_access.sh (1 instance)
  - test_edge_cases.sh (2 instances)

Signed-off-by: Luke Hinds <lukehinds@gmail.com>
- CI was caching the target directory

Signed-off-by: Luke Hinds <lukehinds@gmail.com>
- Narrow out ubuntu tests that fail with tmp ops
- Skip ubuntu intehration tests involving tmp ops
- Fixed the ownership bug

- restructured to use early continue and ? propagation so ruleset

Signed-off-by: Luke Hinds <lukehinds@gmail.com>
- Remove Linux, lets get seperate profiles and then return
- Merge pull request #30 from lukehinds/integration-tests

Add integration test suite and development documentation
- Add Roadmap
- Update with explicit sec guidance
- Refactor for more idiomatic rust
- Merge pull request #42 from lukehinds/process-harde

fix(security): prevent path collision bypass in macOS sandbox
- Bump actions/checkout from 4 to 6

Bumps [actions/checkout](https://github.com/actions/checkout) from 4 to 6.
- [Release notes](https://github.com/actions/checkout/releases)
- [Commits](https://github.com/actions/checkout/compare/v4...v6)

---
updated-dependencies:
- dependency-name: actions/checkout
  dependency-version: '6'
  dependency-type: direct:production
  update-type: version-update:semver-major
...

Signed-off-by: dependabot[bot] <support@github.com>
- Merge pull request #33 from lukehinds/dependabot/github_actions/actions/checkout-6

Bump actions/checkout from 4 to 6
- Bump actions/download-artifact from 4 to 7

Bumps [actions/download-artifact](https://github.com/actions/download-artifact) from 4 to 7.
- [Release notes](https://github.com/actions/download-artifact/releases)
- [Commits](https://github.com/actions/download-artifact/compare/v4...v7)

---
updated-dependencies:
- dependency-name: actions/download-artifact
  dependency-version: '7'
  dependency-type: direct:production
  update-type: version-update:semver-major
...

Signed-off-by: dependabot[bot] <support@github.com>
- Merge pull request #34 from lukehinds/dependabot/github_actions/actions/download-artifact-7

Bump actions/download-artifact from 4 to 7
- Silent Failure on Config Load Error

The get_sensitive_paths function has a silent failure which
could be used to conceal an attack

Signed-off-by: Luke Hinds <lukehinds@gmail.com>
- Formatting
- Merge pull request #43 from lukehinds/silent-fail

Silent Failure on Config Load Error

### Fixed

- Prevent path collision bypass in macOS sandbox

## [Unreleased]

### Security
- **Fixed macOS Seatbelt allowing all file reads** - Removed blanket `(allow file-read*)` that made `--allow` ineffective. Now uses explicit allow-listing matching Linux Landlock behavior.
- Implemented "Allow Discovery, Deny Content" strategy for sensitive paths:
  - `file-read-metadata` allowed (stat, existence checks)
  - `file-read-data` denied (actual content reads blocked)
- Narrowed `system*` permissions to only `system-socket`, `system-fsctl`, `system-info`
- Narrowed `sysctl*` to `sysctl-read` (blocks kernel parameter writes)
- Fixed sensitive path override logic to require explicit grants (granting `~` no longer bypasses `~/.ssh` protection)
- Fixed Seatbelt rule ordering so user-granted paths can delete files while global unlink is blocked
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
