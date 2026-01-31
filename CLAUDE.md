# nono - Development Guide

## Project Overview

nono ("the opposite of YOLO") is a capability-based shell for running untrusted AI agents with OS-enforced isolation. It uses Landlock (Linux) and Seatbelt (macOS) to create sandboxes where unauthorized operations are structurally impossible.

## Architecture

```
src/
├── main.rs           # Entry point, CLI handling, command execution
├── cli.rs            # Clap argument definitions
├── error.rs          # Error types (NonoError)
├── capability.rs     # Capability model (FsCapability, CapabilitySet)
└── sandbox/
    ├── mod.rs        # Platform dispatch (apply, is_supported)
    ├── linux.rs      # Landlock implementation
    └── macos.rs      # Seatbelt implementation
```

## Build & Test

After evevry session at the end of completing a task, run the following commands to ensure correctness:

```bash
# Build
cargo build

# Run tests
cargo test

# Run with verbose logging
RUST_LOG=debug cargo run -- --allow . -- echo "test"

# Dry run (show capabilities without applying sandbox)
cargo run -- --allow . --dry-run -- command
```

## Lint and Format

After evevry session at the end of completing a task, run the following commands to ensure correctness:

```bash
# Lint code
cargo clippy -- -D warnings -D clippy::unwrap_used

# Format code
cargo fmt -- --check
```

## Key Design Decisions

1. **No escape hatch**: Once sandbox is applied via `restrict_self()` (Landlock) or `sandbox_init()` (Seatbelt), there is no API to expand permissions.

2. **exec() model**: After applying the sandbox, nono uses `exec()` to replace itself with the target command. This means the command inherits all restrictions.

3. **Phase 1 read permissions**: Currently allows all file reads on macOS to ensure executables work. Write restrictions are enforced. This will be tightened in Phase 2.

4. **Capability resolution**: All paths are canonicalized at grant time to prevent symlink escapes.

## Platform-Specific Notes

### macOS (Seatbelt)
- Uses `sandbox_init()` FFI with raw profile strings
- Profile is Scheme-like DSL: `(allow file-read* (subpath "/path"))`
- Network denied by default with `(deny network*)`
- System paths (/usr, /bin, /System, etc.) allowed for executables

### Linux (Landlock)
- Uses landlock crate for safe Rust bindings
- Detects highest available ABI (v1-v5)
- ABI v4+ includes TCP network filtering
- Older kernels need seccomp fallback for network (Phase 2)

## Implementation Status

- [x] Phase 1: Filesystem sandbox (MVP)
- [ ] Phase 2: Network isolation (per-host filtering)
- [ ] Phase 3: Runtime capability expansion (supervisor model)
- [ ] Phase 4: Polish and release

## Adding New Capabilities

To add a new capability type:

1. Add variant to `Capability` enum in `capability.rs`
2. Update `CapabilitySet::from_args()` to parse from CLI
3. Update `sandbox/linux.rs` to apply via Landlock
4. Update `sandbox/macos.rs` to generate Seatbelt profile rules

## Testing Sandbox Enforcement

```bash
# Should succeed (write to allowed path)
nono --allow . -- sh -c "echo test > ./allowed.txt"

# Should fail (write to disallowed path)
nono --allow . -- sh -c "echo test > /tmp/outside.txt"

# Should fail (network blocked)
nono --allow . -- curl https://example.com
```

## Coding Standards
- Error Handling: Use NonoError for all errors; propagation via ? only.
- Unwrap Policy: Strictly forbid .unwrap() and .expect(); use clippy::unwrap_used to enforce.
- Unsafe Code: Restrict unsafe to FFI; must be wrapped in safe APIs with // SAFETY: docs.
- Path Security: Validate and canonicalize all paths before applying capabilities.
- Arithmetic: Use checked_, saturating_, or overflowing_ methods for security-critical math.
- Memory: Use the zeroize crate for sensitive data (keys/passwords) in memory.
- Dependencies: Mandatory cargo-audit and cargo-deny checks in CI.
- Testing: Write unit tests for all new capability types and sandbox logic.
- Attributes: Apply #[must_use] to all functions returning critical Results.

## Security Considerations
- Principle of Least Privilege: Only grant necessary capabilities.
- Defense in Depth: Combine OS-level sandboxing with application-level checks.

## References

- [SPEC.md](./SPEC.md) - Full specification and threat model
- [IMPLEMENTATION.md](./IMPLEMENTATION.md) - Detailed implementation plan
- [Landlock docs](https://landlock.io/)
- [macOS Sandbox Guide](https://developer.apple.com/library/archive/documentation/Security/Conceptual/AppSandboxDesignGuide/)
