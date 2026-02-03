<div align="center">

# nono

**A secure, kernel-enforced capability sandbox for AI agents**

<a href="https://discord.gg/pPcjYzGvbS">
  <img src="https://img.shields.io/badge/Chat-Join%20Discord-7289da?style=for-the-badge&logo=discord&logoColor=white" alt="Join Discord"/>
</a>

<p>
  <a href="https://opensource.org/licenses/Apache-2.0">
    <img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License"/>
  </a>
  <a href="https://github.com/lukehinds/nono/actions/workflows/ci.yml">
    <img src="https://github.com/lukehinds/nono/actions/workflows/ci.yml/badge.svg" alt="CI Status"/>
  </a>
  <a href="https://discord.gg/pPcjYzGvbS">
    <img src="https://img.shields.io/discord/1384081906773131274?color=7289da&label=Discord&logo=discord&logoColor=white" alt="Discord"/>
  </a>
</p>

</div>

> [!WARNING]
> This is an early alpha release that has not undergone comprehensive security auditing or peer review. Some of the core policy files are still undergoing changes. Although care and attention has been made and the author has a long background in security, there are no guarantees regarding maturity or stability. Not recommended for production environments. Please do raise bugs, if you see something wrong, you're probably right. 

**nono** is a secure, kernel-enforced capability shell for running untrusted AI agents and processes. Unlike policy-based sandboxes that intercept and filter operations, nono leverages OS security primitives (Landlock on Linux, Seatbelt on macOS) to create an environment where unauthorized operations are structurally impossible.

## Quick Start

### Homebrew (macOS)

```bash
brew tap lukehinds/nono 
brew install nono
```

### Prebuilt Binaries
Download the latest release from the [Releases](https://github.com/lukehinds/nono/releases) page.

### Build from Source

```bash
git clone https://github.com/lukehinds/nono.git
cd nono
cargo build --release
```

## Get Started!


### Claude Code

Run Claude Code in a sandboxed environment:

```bash
nono run --profile claude-code --allow . claude
```

This command:
- Grants read+write access to your current directory

#### Creating an Alias

Choose ONE of the following options:

**Option 1: Simple (limited) alias** (if you only need current directory access)

```bash
alias sclaude='nono run --profile claude-code --allow . claude'
```

Usage: `sclaude`

> Note: This alias does NOT support additional flags like `--allow /tmp`. If you need to grant access to additional paths, use Option 2 instead.

**Option 2: Flexible function** (if you need to grant additional paths)

```bash
sclaude() {
    nono run --profile claude-code --allow . "$@" -- claude
}
```

Usage:
```bash
sclaude                           # Current directory only
sclaude --allow /tmp              # Current directory + /tmp
sclaude --read ~/Documents        # Current directory + read-only ~/Documents
```

#### Verifying Your Setup

After adding the alias/function, reload your shell configuration:

```bash
source ~/.zshrc  # or ~/.bashrc
```

Test that it works:

```bash
# Check that nono grants the expected permissions
sclaude --dry-run

# Verify Claude version (ensure consistent version)
which claude
```

## Features

- **No escape hatch** - Once inside nono, there is no mechanism to bypass restrictions
- **Agent agnostic** - Works with any AI agent (Claude, GPT, opencode, openclaw) or any process
- **OS-level enforcement** - Kernel denies unauthorized operations
- **Destructive command blocking** - Blocks dangerous commands like `rm`, `dd`, `chmod` by default
- **Cross-platform** - Linux (Landlock) and macOS (Seatbelt)

## Usage

```bash
# Allow read+write to current directory
nono run --allow . -- command

# Separate read and write permissions
nono run --read ./src --write ./output -- cargo build

# Multiple paths
nono run --allow ./project-a --allow ./project-b -- command

# Block network access
nono run --allow . --net-block -- command

# Dry run (show what would be sandboxed)
nono run --allow . --dry-run -- command

# Check why a path would be blocked
nono why ~/.ssh/id_rsa
```

## Command Blocking

nono blocks dangerous commands by default to prevent AI agents from accidentally (or maliciously) causing harm. This provides defense-in-depth beyond filesystem restrictions.

### Blocked Commands

The following categories of commands are blocked by default:

| Category | Commands |
|----------|----------|
| File destruction | `rm`, `rmdir`, `shred`, `srm` |
| Disk operations | `dd`, `mkfs`, `fdisk`, `parted`, `wipefs` |
| Permission changes | `chmod`, `chown`, `chgrp`, `chattr` |
| System modification | `shutdown`, `reboot`, `halt`, `systemctl` |
| Package managers | `apt`, `brew`, `pip`, `yum`, `pacman` |
| File operations | `mv`, `cp`, `truncate` |
| Privilege escalation | `sudo`, `su`, `doas`, `pkexec` |
| Network exfiltration | `scp`, `rsync`, `sftp`, `ftp` |

### Overriding Command Blocks

```bash
# Allow a specific blocked command (use with caution)
nono run --allow . --allow-command rm -- rm ./temp-file.txt

# Block an additional command
nono run --allow . --block-command my-dangerous-tool -- my-script.sh
```

### Kernel-Level Protection

Even if a command is allowed via `--allow-command`, nono applies kernel-level protection that blocks:

- **File deletion** - `unlink`/`rmdir` syscalls are blocked
- **File truncation** - Cannot zero out files via truncation

This means even if `rm` is allowed, the actual deletion is blocked by the kernel:

```bash
$ nono run --allow /tmp/test --allow-command rm -- rm /tmp/test/file.txt
rm: /tmp/test/file.txt: Operation not permitted
```

## How It Works

```
┌─────────────────────────────────────────────────┐
│  Terminal                                       │
│                                                 │
│  $ nono run --allow ./project -- agent          │
│                                                 │
│  ┌───────────────────────────────────────────┐  │
│  │  nono (applies sandbox, then exec)        │  │
│  │                                           │  │
│  │  ┌─────────────────────────────────────┐  │  │
│  │  │  Agent (sandboxed)            │  │  │
│  │  │  - Can read/write ./project         │  │  │
│  │  │  - Cannot access ~/.ssh, ~/.aws...  │  │  │
│  │  │  - Network: allowed (or blocked)    │  │  │
│  │  └─────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

## Platform Support

| Platform | Mechanism | Kernel | Status |
|----------|-----------|--------|--------|
| macOS | Seatbelt | 10.5+ | Filesystem + Network |
| Linux | Landlock | 5.13+ | Filesystem |
| Linux | Landlock | 6.7+ | Filesystem + Network (TCP) |
| Windows | - | - | Not yet supported |

## Roadmap

### Planned Features

| Feature | Description |
|---------|-------------|
| **Advisory API** | Allow agents to preemptively check permissions before attempting operations, avoiding trial-and-error failures |
| **Signed Policy Files** | Policy files signed and attestable via [Sigstore Rekor](https://rekor.sigstore.dev/), with embedded DSSE signed payloads. Users can craft and sign their own default policies |
| **Interactive Permission Mode** | `nono run --interactive` spawns a supervisor that prompts when blocked operations are attempted |
| **Network Filtering** | Fine-grained network controls (e.g. allowlist/denylist hosts, ports, protocols) |
| **Time-Limited Permissions** | `nono run --allow /tmp:5m -- agent` grants temporary access that expires automatically |
| **Learning Mode** | `nono learn -- command` traces syscalls and generates a minimal capability profile |
| **Ephemeral Mode** | `nono run --ephemeral` creates a copy-on-write overlay filesystem where writes are isolated, enabling full undo |
| **Audit Logging** | `nono run --audit-log ./session.jsonl -- command` logs all sandbox-relevant operations for post-hoc analysis and replay |
| **Windows Support** | Implement a Windows version using Job Objects and Windows Sandbox |


## Security Model

nono follows a capability-based security model with defense-in-depth:

1. **Command validation** - Dangerous commands (rm, dd, chmod, etc.) are blocked before execution
2. **Sandbox applied** - OS-level restrictions are applied (irreversible)
3. **Kernel enforcement** - Even allowed paths cannot have files deleted or truncated
4. **Command executed** - The command runs with only granted capabilities
5. **All children inherit** - Subprocesses also run under restrictions

### Defense Layers

| Layer | Protection | Bypass |
|-------|------------|--------|
| Command blocklist | Blocks known-dangerous binaries | `--allow-command` |
| Kernel (delete) | Blocks unlink/rmdir syscalls | None |
| Kernel (truncate) | Blocks file truncation | None |
| Filesystem sandbox | Restricts path access | Explicit `--allow` |
| Network sandbox | Blocks network access | Remove `--net-block` |

## License

Apache-2.0
