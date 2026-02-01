# Examples

Common usage patterns and recipes for nono.

## AI Coding Agents

### Claude Code

Run Claude Code with access limited to your project:

```bash
nono run --allow . -- claude
```

Allow Claude to read your global config:

```bash
nono run --allow . --read-file ~/.claude/config.json -- claude
```

### OpenClaw

Run OpenClaw gateway with nono sandbox:

```bash
nono run --profile openclaw -- openclaw gateway
```

Or manually specify permissions:

```bash
nono run --allow ~/.openclaw -- openclaw gateway
```

### Generic AI Agent

```bash
nono run --allow ./workspace -- my-ai-agent
```

## Checking Path Access

### Why is a path blocked?

```bash
# Check a sensitive path
nono why ~/.ssh/id_rsa
# Output: BLOCKED: ~/.ssh/id_rsa is a sensitive path

# Check with suggestions
nono why -s ~/.aws
# Output includes: nono run --read ~/.aws -- <command>
```

### Check a project directory

```bash
nono why -s ./my-project
# Shows what flags would grant access
```

## Build Tools

### Cargo (Rust)

```bash
# Full build with all access
nono run --allow . -- cargo build

# Read source, write only to target
nono run --read ./src --read ./Cargo.toml --read ./Cargo.lock --allow ./target -- cargo build
```

### npm/Node.js

```bash
# Install dependencies (requires network, allowed by default)
nono run --allow . -- npm install

# Run build (offline)
nono run --allow . --net-block -- npm run build

# Run tests
nono run --allow . -- npm test
```

### Make

```bash
nono run --allow . -- make
```

## Network Operations

### curl/wget

```bash
# Download a file (network allowed by default)
nono run --write ./downloads -- curl -o ./downloads/file.tar.gz https://example.com/file.tar.gz

# API request
nono run --allow . -- curl -X POST https://api.example.com/data
```

### Git Operations

```bash
# Clone (network allowed by default)
nono run --allow ./repos -- git clone https://github.com/user/repo.git

# Local operations
nono run --allow . -- git status
nono run --allow . -- git commit -m "message"

# Push/pull (network allowed by default)
nono run --allow . -- git push
```

## Multi-Directory Access

### Separate Source and Output

```bash
nono run --read ./src --allow ./dist -- webpack build
```

### Multiple Projects

```bash
nono run --allow ./project-a --allow ./project-b -- my-tool
```

### Shared Dependencies

```bash
nono run --allow . --read ~/.local/share/my-tool -- my-tool
```

## Debugging and Testing

### Dry Run

Preview what access would be granted:

```bash
nono run --allow . --read /etc --dry-run -- my-agent
```

### Verbose Output

```bash
# Maximum verbosity
nono run -vvv --allow . -- command
```

### Testing Sandbox Enforcement

```bash
# Should succeed - writing to allowed path
nono run --allow . -- sh -c "echo test > ./allowed.txt"

# Should fail - writing outside allowed path
nono run --allow . -- sh -c "echo test > /tmp/blocked.txt"

# Should succeed - network allowed by default
nono run --allow . -- curl https://example.com

# Should fail - network blocked with --net-block
nono run --allow . --net-block -- curl https://example.com
```

## Shell Scripts

### Running a Script

```bash
nono run --allow . -- ./my-script.sh
```

### Inline Commands

```bash
nono run --allow . -- sh -c "echo hello && ls -la"
```

## Configuration Files

### Read-Only Config

```bash
nono run --allow . --read-file ~/.config/myapp/config.toml -- myapp
```

### Multiple Config Files

```bash
nono run --allow . \
  --read-file ~/.gitconfig \
  --read-file ~/.npmrc \
  -- my-tool
```

## Using Profiles

### Built-in Profiles

```bash
# Claude Code profile
nono run --profile claude-code -- claude

# OpenClaw profile
nono run --profile openclaw -- openclaw gateway

# Cargo build profile
nono run --profile cargo-build -- cargo build
```

### Profile with Extra Permissions

```bash
nono run --profile claude-code --read /tmp/extra -- claude
```

### Profile with Custom Workdir

```bash
nono run --profile claude-code --workdir ./my-project -- claude
```

## Real-World Scenarios

### Code Review Agent

An agent that reads code and writes review comments:

```bash
nono run \
  --read ./src \
  --read ./tests \
  --write ./reviews \
  -- code-review-agent
```

### Documentation Generator

An agent that reads source and generates docs:

```bash
nono run \
  --read ./src \
  --allow ./docs \
  -- doc-generator
```

### Data Processing Pipeline

```bash
nono run \
  --read ./input \
  --write ./output \
  --read-file ./config.yaml \
  -- data-processor
```

### Offline Build Environment

```bash
nono run \
  --allow . \
  --net-block \
  -- make release
```
