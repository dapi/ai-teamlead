#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(mktemp -d /tmp/ai-teamlead-zellij-XXXXXX)"
create_test_repo "$REPO_ROOT"

AI_TEAMLEAD_BIN="/test/bin/ai-teamlead"

(
    cd "$REPO_ROOT"
    "$AI_TEAMLEAD_BIN" internal launch-zellij-fixture 42
)

ISSUE_INDEX="$REPO_ROOT/.git/ai-teamlead/issues/42.json"
if ! wait_for_file "$ISSUE_INDEX"; then
    echo "  FAIL: issue index file created"
    ((FAIL++)) || true
    return 0
fi
assert_file_exists "$ISSUE_INDEX" "issue index file created"

SESSION_UUID="$(jq -r '.session_uuid' "$ISSUE_INDEX")"
SESSION_MANIFEST="$REPO_ROOT/.git/ai-teamlead/sessions/$SESSION_UUID/session.json"
LAYOUT_FILE="$REPO_ROOT/.git/ai-teamlead/sessions/$SESSION_UUID/launch-layout.kdl"
ENTRYPOINT_FILE="$REPO_ROOT/.git/ai-teamlead/sessions/$SESSION_UUID/launch-agent.sh"
CAPTURE_LOG="$REPO_ROOT/.git/ai-teamlead/sessions/$SESSION_UUID/capture.log"

if ! wait_for_file "$SESSION_MANIFEST"; then
    echo "  FAIL: session manifest created"
    ((FAIL++)) || true
    return 0
fi

assert_file_exists "$SESSION_MANIFEST" "session manifest created"
assert_file_exists "$LAYOUT_FILE" "launcher layout created"
assert_file_exists "$ENTRYPOINT_FILE" "launcher entrypoint created"

TAB_ID="$(wait_for_json_field_not_value "$SESSION_MANIFEST" '.zellij.tab_id' 'pending' 30 || true)"
PANE_ID="$(wait_for_json_field_not_value "$SESSION_MANIFEST" '.zellij.pane_id' 'pending' 30 || true)"
SESSION_ID="$(jq -r '.zellij.session_id' "$SESSION_MANIFEST")"

if [[ -z "$TAB_ID" || -z "$PANE_ID" ]] && [[ -f "$CAPTURE_LOG" ]]; then
    echo "  INFO: capture.log"
    sed -n '1,200p' "$CAPTURE_LOG"
fi

assert_eq "$SESSION_ID" "ai-teamlead-test" "session_id captured from configured session name"
assert_ne "$TAB_ID" "" "tab_id captured from zellij"
assert_ne "$PANE_ID" "" "pane_id captured from zellij"
assert_session_alive "ai-teamlead-test" "zellij session is alive after launcher run"
