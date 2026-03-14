#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd -P)"

exec "$REPO_ROOT/.codex/skills/doc-review-parallel/scripts/run-doc-review.sh" "$@"
