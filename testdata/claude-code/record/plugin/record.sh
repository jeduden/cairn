#!/bin/sh
# record.sh BASE SDK_DIR — spike S4: install the stub Cairn plugin on each
# target and record one harness session per target under BASE/<target>:
#   workstation   `claude plugin marketplace add` and `claude plugin install`
#   runner-image  a ~/.claude baked as an image build would: settings.json declares
#                 the marketplace and enables the plugin; no install command
#   sdk-worker    an Agent SDK worker loading the plugin by path
#   interactive   the workstation install, driven through tmux (OQ-06)
# BASE must be a scratch directory; every target gets its own HOME inside it.
set -eu
BASE="${1:?scratch directory}"; SDK="${2:?directory holding the Agent SDK}"
HERE=$(cd "$(dirname "$0")" && pwd)
TOOL=mcp__plugin_cairn_cairn__event_search
PROMPT="Do these steps in order. 1) Call the tool $TOOL with query \"alpha\". \
2) Use the Agent tool to start one general-purpose subagent whose only job is to \
call $TOOL with query \"beta\" and report its result. 3) Run the Bash command: \
touch denied.txt 4) Reply with the word done."

prepare() { # TARGET — a fresh HOME and work directory, and the marketplace
  export REC_ROOT="$BASE/$1" REC_OUT="$BASE/$1/out"
  rm -rf "$REC_ROOT"; mkdir -p "$REC_ROOT/home/.claude" "$REC_ROOT/w/ws" "$REC_OUT"
  cp -r "$HERE" "$REC_ROOT/src"
}

print_mode() { # run the prompt in print mode, the stream to out/stream.jsonl
  (cd "$REC_ROOT/w/ws" && timeout 300 "$REC_ROOT/src/run" -p "$PROMPT" \
    --model sonnet --permission-mode default --output-format stream-json --verbose \
    --allowedTools "$TOOL,Agent" </dev/null >"$REC_OUT/stream.jsonl" 2>"$REC_OUT/stderr")
}

prepare workstation
"$REC_ROOT/src/run" plugin marketplace add "$REC_ROOT/src" >"$REC_OUT/install.txt"
"$REC_ROOT/src/run" plugin install cairn@cairn-s4 >>"$REC_OUT/install.txt"
"$REC_ROOT/src/run" plugin list >>"$REC_OUT/install.txt"
print_mode

prepare runner-image
cat >"$REC_ROOT/home/.claude/settings.json" <<JSON
{
  "extraKnownMarketplaces": {
    "cairn-s4": { "source": { "source": "directory", "path": "$REC_ROOT/src" } }
  },
  "enabledPlugins": { "cairn@cairn-s4": true }
}
JSON
print_mode

prepare sdk-worker
(cd "$REC_ROOT/w/ws" && REC_BIN=node timeout 300 "$REC_ROOT/src/run" \
  "$REC_ROOT/src/sdk-worker.mjs" "$SDK" "$REC_ROOT/src/cairn" "$PROMPT" \
  </dev/null >"$REC_OUT/stream.jsonl" 2>"$REC_OUT/stderr")

prepare interactive
"$REC_ROOT/src/run" plugin marketplace add "$REC_ROOT/src" >"$REC_OUT/install.txt"
"$REC_ROOT/src/run" plugin install cairn@cairn-s4 >>"$REC_OUT/install.txt"
python3 - "$REC_ROOT" <<'PY'  # skip the first-run screens and the trust dialog
import json, os, sys
root = sys.argv[1]; path = f"{root}/home/.claude/.claude.json"
state = json.load(open(path)) if os.path.exists(path) else {}
state.update(hasCompletedOnboarding=True, theme="dark")
state.setdefault("projects", {})[f"{root}/w/ws"] = {
    "hasTrustDialogAccepted": True, "hasCompletedProjectOnboarding": True}
json.dump(state, open(path, "w"))
PY
T=cairn-s4-$$
tmux new-session -d -s "$T" -x 200 -y 50 -c "$REC_ROOT/w/ws" \
  "REC_ROOT='$REC_ROOT' REC_OUT='$REC_OUT' '$REC_ROOT/src/run' --model sonnet --permission-mode default --allowedTools '$TOOL'"
sleep 10
tmux send-keys -t "$T" -l "Call the tool $TOOL with query \"gamma\", then run the Bash command: touch denied.txt"
sleep 1; tmux send-keys -t "$T" C-m      # C-m submits; a plain Enter does not
sleep 30
tmux send-keys -t "$T" Escape; sleep 2   # refuse the Bash permission prompt
tmux capture-pane -p -t "$T" >"$REC_OUT/screen.txt"
tmux send-keys -t "$T" -l "/exit"; sleep 1; tmux send-keys -t "$T" C-m; sleep 5
tmux kill-session -t "$T" 2>/dev/null || true
