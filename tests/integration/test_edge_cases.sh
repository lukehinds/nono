#!/bin/bash
# Edge Case Tests
# Tests symlinks, path variations, environment variables, and other edge cases

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_helpers.sh"

echo ""
echo -e "${BLUE}=== Edge Case Tests ===${NC}"

verify_nono_binary

# Create test fixtures
TMPDIR=$(setup_test_dir)
trap 'cleanup_test_dir "$TMPDIR"' EXIT

echo ""
echo "Test directory: $TMPDIR"
echo ""

# =============================================================================
# Symlink Tests
# =============================================================================

echo "--- Symlink Tests ---"

# Note: On macOS, TMPDIR (/var/folders) is a system-accessible path,
# so symlink tests within TMPDIR won't show denial behavior.
# We test symlinks that work, and symlink escapes to sensitive paths.

# Setup: Create directories and symlinks
mkdir -p "$TMPDIR/real_allowed"
echo "allowed content" > "$TMPDIR/real_allowed/data.txt"
ln -s "$TMPDIR/real_allowed" "$TMPDIR/symlink_to_allowed"

# Access via symlink to allowed directory
expect_success "access file via symlink to allowed directory" \
    "$NONO_BIN" run --allow "$TMPDIR/real_allowed" -- cat "$TMPDIR/symlink_to_allowed/data.txt"

# Symlink escape to sensitive path should fail
# Create a symlink in allowed dir pointing to ~/.ssh (a sensitive path)
mkdir -p "$TMPDIR/allowed_with_escape"
echo "safe" > "$TMPDIR/allowed_with_escape/safe.txt"

if [[ -d ~/.ssh ]]; then
    ln -s ~/.ssh "$TMPDIR/allowed_with_escape/ssh_escape"
    expect_failure "symlink escape to sensitive path blocked" \
        "$NONO_BIN" run --allow "$TMPDIR/allowed_with_escape" -- ls "$TMPDIR/allowed_with_escape/ssh_escape/"
else
    skip_test "symlink escape to sensitive path" "~/.ssh not found"
fi

# File symlink within allowed directory
echo "linked file content" > "$TMPDIR/real_file.txt"
ln -s "$TMPDIR/real_file.txt" "$TMPDIR/file_symlink.txt"

expect_success "file symlink to allowed file works" \
    "$NONO_BIN" run --allow "$TMPDIR" -- cat "$TMPDIR/file_symlink.txt"

# Symlink chain (symlink to symlink to file)
ln -s "$TMPDIR/file_symlink.txt" "$TMPDIR/chain_symlink.txt"

expect_success "symlink chain works within allowed paths" \
    "$NONO_BIN" run --allow "$TMPDIR" -- cat "$TMPDIR/chain_symlink.txt"

# =============================================================================
# Path Variations
# =============================================================================

echo ""
echo "--- Path Variations ---"

# Create subdirectory for path tests
mkdir -p "$TMPDIR/subdir/nested"
echo "nested content" > "$TMPDIR/subdir/nested/file.txt"

# Relative path grant
ORIGINAL_DIR=$(pwd)
cd "$TMPDIR"

expect_success "relative path grant (./subdir)" \
    "$NONO_BIN" run --allow ./subdir -- cat ./subdir/nested/file.txt

cd "$ORIGINAL_DIR"

# Path with .. (parent references)
cd "$TMPDIR/subdir"

expect_success "path with .. references" \
    "$NONO_BIN" run --allow ../subdir -- cat ../subdir/nested/file.txt

cd "$ORIGINAL_DIR"

# Paths with spaces
mkdir -p "$TMPDIR/path with spaces/nested dir"
echo "spaced content" > "$TMPDIR/path with spaces/nested dir/file.txt"

expect_success "path with spaces" \
    "$NONO_BIN" run --allow "$TMPDIR/path with spaces" -- cat "$TMPDIR/path with spaces/nested dir/file.txt"

# Paths with special characters (but safe ones)
mkdir -p "$TMPDIR/path-with-dashes_and_underscores"
echo "special" > "$TMPDIR/path-with-dashes_and_underscores/file.txt"

expect_success "path with dashes and underscores" \
    "$NONO_BIN" run --allow "$TMPDIR/path-with-dashes_and_underscores" -- cat "$TMPDIR/path-with-dashes_and_underscores/file.txt"

# =============================================================================
# Environment Variables
# =============================================================================

echo ""
echo "--- Environment Variables ---"

expect_output_contains "NONO_ACTIVE is set to 1" "NONO_ACTIVE=1" \
    "$NONO_BIN" run --allow "$TMPDIR" -- env

expect_output_contains "NONO_ALLOWED contains granted path" "$TMPDIR" \
    "$NONO_BIN" run --allow "$TMPDIR" -- sh -c 'echo $NONO_ALLOWED'

expect_output_contains "NONO_NET shows 'allowed' by default" "allowed" \
    "$NONO_BIN" run --allow "$TMPDIR" -- sh -c 'echo $NONO_NET'

expect_output_contains "NONO_NET shows 'blocked' with --net-block" "blocked" \
    "$NONO_BIN" run --net-block --allow "$TMPDIR" -- sh -c 'echo $NONO_NET'

# NONO_BLOCKED should contain sensitive paths
expect_output_contains "NONO_BLOCKED contains sensitive paths" ".ssh" \
    "$NONO_BIN" run --allow "$TMPDIR" -- sh -c 'echo $NONO_BLOCKED'

# NONO_HELP should provide guidance
expect_output_contains "NONO_HELP is set" "nono" \
    "$NONO_BIN" run --allow "$TMPDIR" -- sh -c 'echo $NONO_HELP'

# =============================================================================
# Non-existent Paths
# =============================================================================

echo ""
echo "--- Non-existent Paths ---"

expect_failure "grant non-existent directory fails at startup" \
    "$NONO_BIN" run --allow /nonexistent/path/that/does/not/exist/anywhere -- echo "should not run"

expect_failure "grant non-existent file fails at startup" \
    "$NONO_BIN" run --read-file /nonexistent/file.txt -- echo "should not run"

# Reading a file that doesn't exist (but directory is allowed) should give normal "not found" error
expect_failure "read non-existent file in allowed dir gives file error" \
    "$NONO_BIN" run --allow "$TMPDIR" -- cat "$TMPDIR/this_file_does_not_exist.txt"

# =============================================================================
# Dry Run Mode
# =============================================================================

echo ""
echo "--- Dry Run Mode ---"

# Use echo instead of rm since rm is blocked even in dry-run
expect_success "dry-run shows sandbox info" \
    "$NONO_BIN" run --dry-run --allow "$TMPDIR" -- echo "test"

expect_output_contains "dry-run shows granted paths" "$TMPDIR" \
    "$NONO_BIN" run --dry-run --allow "$TMPDIR" -- echo "test"

# Verify dry-run doesn't create files
expect_success "dry-run with touch doesn't create file" \
    "$NONO_BIN" run --dry-run --allow "$TMPDIR" -- touch "$TMPDIR/should_not_exist.txt"

if [[ ! -f "$TMPDIR/should_not_exist.txt" ]]; then
    echo -e "  ${GREEN}PASS${NC}: dry-run did not execute command"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "  ${RED}FAIL${NC}: dry-run executed command!"
    rm -f "$TMPDIR/should_not_exist.txt"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
TESTS_RUN=$((TESTS_RUN + 1))

# =============================================================================
# Profile Workdir (for variable expansion)
# =============================================================================

echo ""
echo "--- Profile Workdir ---"

# Note: --workdir is for $WORKDIR expansion in profiles, not for setting cwd
# It's tested here to ensure the flag doesn't cause errors
expect_success "--workdir flag accepted (for profile variable expansion)" \
    "$NONO_BIN" run --allow "$TMPDIR" --workdir "$TMPDIR" -- echo "workdir test"

# =============================================================================
# Multiple Permission Types
# =============================================================================

echo ""
echo "--- Multiple Permission Types ---"

mkdir -p "$TMPDIR/mixed_read" "$TMPDIR/mixed_write"
echo "can read" > "$TMPDIR/mixed_read/file.txt"

# Read-only and write-only directories together
expect_success "read from read-only, write to write-only" \
    "$NONO_BIN" run --read "$TMPDIR/mixed_read" --write "$TMPDIR/mixed_write" --allow /tmp -- \
    sh -c "cat '$TMPDIR/mixed_read/file.txt' && echo 'written' > '$TMPDIR/mixed_write/output.txt'"

# Verify write worked
if [[ -f "$TMPDIR/mixed_write/output.txt" ]]; then
    echo -e "  ${GREEN}PASS${NC}: write to write-only directory succeeded"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "  ${RED}FAIL${NC}: write to write-only directory failed"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
TESTS_RUN=$((TESTS_RUN + 1))

# =============================================================================
# Summary
# =============================================================================

print_summary
