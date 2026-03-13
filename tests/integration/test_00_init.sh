#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(mktemp -d /tmp/ai-teamlead-init-XXXXXX)"
git init -q "$REPO_ROOT"
git -C "$REPO_ROOT" remote add origin git@github.com:dapi/example.git

AI_TEAMLEAD_BIN="/test/bin/ai-teamlead"

OUTPUT="$(
    cd "$REPO_ROOT"
    "$AI_TEAMLEAD_BIN" init
)"

SETTINGS_FILE="$REPO_ROOT/.ai-teamlead/settings.yml"
README_FILE="$REPO_ROOT/.ai-teamlead/README.md"
FLOW_FILE="$REPO_ROOT/.ai-teamlead/flows/issue-analysis-flow.md"
RUNTIME_DIR="$REPO_ROOT/.git/.ai-teamlead"

assert_file_exists "$SETTINGS_FILE" "init created settings.yml"
assert_file_exists "$README_FILE" "init created .ai-teamlead README"
assert_file_exists "$FLOW_FILE" "init created issue-analysis-flow.md"

if [[ -d "$RUNTIME_DIR" ]]; then
    echo "  FAIL: init must not create runtime directory"
    ((FAIL++)) || true
else
    echo "  PASS: init does not create runtime directory"
    ((PASS++)) || true
fi

if [[ "$OUTPUT" == *"created: $SETTINGS_FILE"* ]] && [[ "$OUTPUT" == *"created: $README_FILE"* ]] && [[ "$OUTPUT" == *"created: $FLOW_FILE"* ]]; then
    echo "  PASS: init reports created files"
    ((PASS++)) || true
else
    echo "  FAIL: init reports created files"
    ((FAIL++)) || true
fi

SECOND_OUTPUT="$(
    cd "$REPO_ROOT"
    "$AI_TEAMLEAD_BIN" init
)"

if [[ "$SECOND_OUTPUT" == *"skipped: $SETTINGS_FILE"* ]] && [[ "$SECOND_OUTPUT" == *"skipped: $README_FILE"* ]] && [[ "$SECOND_OUTPUT" == *"skipped: $FLOW_FILE"* ]]; then
    echo "  PASS: init is idempotent"
    ((PASS++)) || true
else
    echo "  FAIL: init is idempotent"
    ((FAIL++)) || true
fi
