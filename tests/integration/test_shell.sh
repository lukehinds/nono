#!/bin/bash
# Interactive Shell Tests
# Verifies nono shell runs commands inside the sandbox and enforces restrictions

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_helpers.sh"

echo ""
echo -e "${BLUE}=== Shell Tests ===${NC}"

verify_nono_binary

PROJECT_ROOT="$(get_project_root)"

BASE_DIR="$(mktemp -d "$PROJECT_ROOT/target/nono-shell-test-XXXX")"
trap 'cleanup_test_dir "$BASE_DIR"' EXIT

ALLOWED_DIR="$BASE_DIR/allowed"
DENIED_DIR="$BASE_DIR/denied"

mkdir -p "$ALLOWED_DIR" "$DENIED_DIR"
echo "ALLOWED_OK" > "$ALLOWED_DIR/ok.txt"
echo "DENIED_SECRET" > "$DENIED_DIR/secret.txt"

if is_macos; then
    EXPECT_DENIED="Operation not permitted"
else
    EXPECT_DENIED="Permission denied"
fi

echo ""
echo "Test directory: $BASE_DIR"
echo ""

# Shell should read allowed path
expect_output_contains "shell can read allowed file" "ALLOWED_OK" \
    bash -c "cat <<'EOF' | \"$NONO_BIN\" shell --allow \"$ALLOWED_DIR\" --shell /bin/sh
cat \"$ALLOWED_DIR/ok.txt\"
exit
EOF"

# Shell should not read denied path
expect_output_contains "shell blocks denied file" "$EXPECT_DENIED" \
    bash -c "cat <<'EOF' | \"$NONO_BIN\" shell --allow \"$ALLOWED_DIR\" --shell /bin/sh
cat \"$DENIED_DIR/secret.txt\"
exit
EOF"

# =============================================================================
# Summary
# =============================================================================

print_summary
