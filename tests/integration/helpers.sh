#!/usr/bin/env bash
set -euo pipefail

PASS=0
FAIL=0
ZELLIJ_TEST_TIMEOUT="${ZELLIJ_TEST_TIMEOUT:-20}"

assert_eq() {
    local actual="$1" expected="$2" msg="$3"
    if [[ "$actual" == "$expected" ]]; then
        echo "  PASS: $msg"
        ((PASS++)) || true
    else
        echo "  FAIL: $msg"
        echo "    expected: '$expected'"
        echo "    actual:   '$actual'"
        ((FAIL++)) || true
    fi
}

assert_ne() {
    local actual="$1" expected="$2" msg="$3"
    if [[ "$actual" != "$expected" ]]; then
        echo "  PASS: $msg"
        ((PASS++)) || true
    else
        echo "  FAIL: $msg"
        echo "    unexpected: '$actual'"
        ((FAIL++)) || true
    fi
}

assert_file_exists() {
    local path="$1" msg="$2"
    if [[ -f "$path" ]]; then
        echo "  PASS: $msg"
        ((PASS++)) || true
    else
        echo "  FAIL: $msg"
        echo "    missing file: $path"
        ((FAIL++)) || true
    fi
}

assert_session_alive() {
    local session_name="$1" msg="$2"
    if zellij list-sessions --short 2>/dev/null | grep -Fxq "$session_name"; then
        echo "  PASS: $msg"
        ((PASS++)) || true
    else
        echo "  FAIL: $msg"
        ((FAIL++)) || true
    fi
}

wait_for_file() {
    local path="$1"
    local timeout_seconds="${2:-$ZELLIJ_TEST_TIMEOUT}"
    local deadline=$((SECONDS + timeout_seconds))
    while (( SECONDS < deadline )); do
        if [[ -f "$path" ]]; then
            return 0
        fi
        sleep 0.2
    done
    return 1
}

wait_for_json_field_not_value() {
    local path="$1" field="$2" bad_value="$3"
    local timeout_seconds="${4:-$ZELLIJ_TEST_TIMEOUT}"
    local deadline=$((SECONDS + timeout_seconds))
    while (( SECONDS < deadline )); do
        if [[ -f "$path" ]]; then
            local value
            value=$(jq -r "$field" "$path" 2>/dev/null || true)
            if [[ -n "$value" && "$value" != "null" && "$value" != "$bad_value" ]]; then
                echo "$value"
                return 0
            fi
        fi
        sleep 0.2
    done
    return 1
}

create_test_repo() {
    local repo_root="$1"
    mkdir -p "$repo_root/.ai-teamlead"
    git init -q "$repo_root"
    git -C "$repo_root" remote add origin git@github.com:dapi/teamlead.git
    cat > "$repo_root/.ai-teamlead/settings.yml" <<'EOF'
github:
  project_id: "PVT_test_project"

issue_analysis_flow:
  statuses:
    backlog: "Backlog"
    analysis_in_progress: "Analysis In Progress"
    waiting_for_clarification: "Waiting for Clarification"
    waiting_for_plan_review: "Waiting for Plan Review"
    ready_for_implementation: "Ready for Implementation"
    analysis_blocked: "Analysis Blocked"

runtime:
  max_parallel: 1
  poll_interval_seconds: 3600

zellij:
  session_name: "ai-teamlead-test"
  tab_name: "issue-analysis"
EOF
}

cleanup_zellij() {
    zellij kill-session ai-teamlead-test >/dev/null 2>&1 || true
}

print_summary() {
    echo ""
    echo "=== Summary ==="
    echo "PASS: $PASS"
    echo "FAIL: $FAIL"
    if [[ "$FAIL" -ne 0 ]]; then
        exit 1
    fi
}
