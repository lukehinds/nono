# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `nono shell` for starting an interactive shell inside the sandbox

## [0.2.7] - 2026-02-04

### Changed

- Set release to un-draft

## [0.2.6] - 2026-02-04

### Changed

- Remove signing for now and mark placeholder
- Add roadmap and re-org client docs
- Update links to documentation in README.md
- Revise README for installation and build instructions

Updated installation instructions and removed outdated sections.
- Add MacOS installation instructions to README

Added installation instructions for MacOS to README.
- Refactor CWD handling into profile-driven [workdir] config and remove cargo-build profile

Introduce a [workdir] section in profiles to declare working directory
access requirements (read, readwrite, none). This replaces the previous
$WORKDIR variable approach and enables automatic CWD inclusion with a
y/N confirmation prompt. Add --allow-cwd flag to skip the prompt.

Remove the cargo-build built-in profile and all references to it.
Update client documentation to reflect simplified usage commands.

Signed-off-by: Luke Hinds <lukehinds@gmail.com>
- Fix CWD prompt decline to continue gracefully instead of aborting

Declining the CWD sharing prompt now logs an info message and continues
execution without CWD access, instead of returning a fatal error. This
fixes a regression where commands that don't need filesystem access
(e.g., nono run -- curl ...) would fail if the user declined the prompt.

Remove the now-unused UserDeclinedCwd error variant. Fix confusing
installation docs sentence that referenced the Usage guide for
installation details.

Signed-off-by: Luke Hinds <lukehinds@gmail.com>
- Merge pull request #55 from lukehinds/profiles-update

Refactor CWD handling into profile-driven [workdir] config and remove…
- Fix documentation to match current code state

Audit all docs against source code and fix 17 discrepancies:

- landlock.mdx: Remove RemoveDir from Write access, add Refer
- seatbelt.mdx: Replace blanket (allow process*) with actual rules
- README.md: Fix nono why syntax, correct kernel protection claims
  about file deletion scope, fix defense layers table
- security/index.mdx: Fix typo ~/.gitcredentials -> ~/.git-credentials
- claude-code.mdx: Fix ~/.claude.json access level to read+write
- flags.mdx: Add missing --allow-command, --block-command, --allow-cwd
- profiles.mdx: Add [workdir] section, fix built-in profile examples,
  add $TMPDIR/$UID to env vars, fix filesystem field format
- development/index.mdx: Add missing run subcommand to cargo run
  example, add -D clippy::unwrap_used to clippy command
- vs-containers.mdx: Fix broken internal links
- secrets.mdx: Fix broken link to profiles page
- data/profiles/opencode.toml: Sync paths with builtin.rs
- data/profiles/claude-code.toml: Replace silently-ignored
  [filesystem.files] subsection with flat allow_file field
- data/profiles/openclaw.toml: Same [filesystem.files] fix

Signed-off-by: Luke Hinds <lukehinds@gmail.com>
- Merge pull request #56 from lukehinds/docs-refresh

Fix documentation to match current code state
- Some tests require new --allow-cwd
- Patch gofmt
- Merge pull request #57 from lukehinds/integration-test-fix

Some tests require new --allow-cwd
- Merge pull request #58 from zemberdotnet/fix-readme-dev-guide-link

docs: fix broken Development Guide link in README

### Documentation

- Fix broken Development Guide link in README

## [0.2.5] - 2026-02-03

### Changed

- Fix atomic writes by adding required Landlock permissions

Applications using atomic write patterns (write to .tmp → rename to target)
were failing with EACCES: permission denied, even when the directory was
in the --allow list.

Root cause: The FsAccess::Write mapping was missing three Landlock flags
required for rename() operations:
- LANDLOCK_ACCESS_FS_REFER: required for rename/hard link operations
- LANDLOCK_ACCESS_FS_REMOVE_FILE: required for rename source removal
- LANDLOCK_ACCESS_FS_TRUNCATE: common write operation (ABI v3+)

Per the Landlock kernel docs, rename() requires REMOVE_FILE permission
on the source's parent directory, treating it as "removing" the source.

Security analysis: When users grant --write to a directory, they expect
sandboxed processes to create, modify, AND delete files within that path.
The previous overly-defensive approach broke legitimate use cases while
providing minimal additional security benefit.

Changes:
- Add RemoveFile, Refer, and Truncate to FsAccess::Write mapping
- Update CLI documentation to clarify what write permissions include
- Update tests to verify new permissions are granted
- Still exclude RemoveDir for defense-in-depth (directory deletion)

This fixes atomic writes for Node.js, Python, and other tools that use
this standard pattern for safe configuration updates.

Signed-off-by: Luke Hinds <lukehinds@gmail.com>
- Remove mistaken agents file
- Merge pull request #51 from lukehinds/linux-fsaccess

Fix atomic writes by adding required Landlock permissions

## [0.2.4] - 2026-02-03

### Changed

- Fix release workflow: add tag_name parameter
- Fix crates.io publish with --allow-dirty flag
- Bump actions/cache from 4.3.0 to 5.0.3

Bumps [actions/cache](https://github.com/actions/cache) from 4.3.0 to 5.0.3.
- [Release notes](https://github.com/actions/cache/releases)
- [Changelog](https://github.com/actions/cache/blob/main/RELEASES.md)
- [Commits](https://github.com/actions/cache/compare/0057852bfaa89a56745cba8c7296529d2fc39830...cdf6c1fa76f9f475f3d7449005a359c84ca0f306)

---
updated-dependencies:
- dependency-name: actions/cache
  dependency-version: 5.0.3
  dependency-type: direct:production
  update-type: version-update:semver-major
...

Signed-off-by: dependabot[bot] <support@github.com>
- Merge pull request #36 from lukehinds/dependabot/github_actions/actions/cache-5.0.3

Bump actions/cache from 4.3.0 to 5.0.3
- Replace environment variable introspection with advisory API

This replaces the NONO_* environment variables (NONO_ACTIVE, NONO_ALLOWED,
NONO_BLOCKED, NONO_NET, NONO_HELP, NONO_CONTEXT) with a structured query
API that enables AI agents to pre-check if operations will be allowed
before attempting them. Instead of parsing environment variables,
sandboxed processes now call `nono why --self` to get a programmatic
JSON response explaining why an operation is allowed or denied.

The old `nono why <path>` command is reimplemented with a more powerful
interface that supports filesystem path queries (`--path` and `--op`),
network queries (`--host` and `--port`), JSON output for programmatic
use, and the ability to query current sandbox state from inside the
sandbox with `--self`. Capability context can be provided via the
existing flags (--allow, --read, --write, --profile, etc.) to check
what would be allowed under different configurations.

The sandbox state is now written to a temp file and passed via
NONO_CAP_FILE, which allows `nono why --self` to reconstruct the
capability set from inside the sandbox and answer questions about
what operations are allowed.

Signed-off-by: Luke Hinds <lukehinds@gmail.com>
- Merge pull request #45 from lukehinds/advisory-api

Replace environment variable introspection with advisory API
- Add Claude Instructions
- Add Claude Instructions
- Update release script with cargo build

## [0.2.3] - 2026-02-03

### Changed

- Add workflow_dispatch to manually trigger releases

## [0.2.3] - 2026-02-03

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
