#!/bin/sh
# defer.sh BASE [FLOW...] — spike S6: record a PreCompact handler that exits 2
# (block.sh) in print mode, every other hook saved by dump.sh. BASE must be a
# scratch directory; it becomes REC_ROOT, with its own HOME (ENG-14). Flows:
#   compact-blocked   a prompt, then /compact on --resume, then a prompt asking
#                     for the word block.sh wrote to stderr
#   full-window       one prompt reads twelve files of about 12k tokens each,
#                     past the automatic compaction's threshold
#                     (CLAUDE_CODE_AUTO_COMPACT_WINDOW=100000,
#                     CLAUDE_AUTOCOMPACT_PCT_OVERRIDE=40) and on to the full
#                     window, which CLAUDE_CODE_BLOCKING_LIMIT_OVERRIDE=60000
#                     lowers from the model's own; then a prompt on --resume
#                     asks for the word
# Without FLOWs it records both. Hook inputs land in BASE/out/<flow>, the
# stream of each prompt beside them, the transcripts in BASE/home/.claude.
set -eu
BASE="${1:?scratch directory}"; shift
FLOWS="${*:-compact-blocked full-window}"
HERE=$(cd "$(dirname "$0")" && pwd)
export REC_ROOT="$BASE"
mkdir -p "$BASE/home/.claude" "$BASE/bin"
cp "$HERE"/*.sh "$HERE/plugin/run" "$BASE/bin/"
hook() { printf '[{"hooks":[{"type":"command","command":"%s","timeout":10}]}]' "$1"; }
D="$BASE/bin/dump.sh"
cat > "$BASE/home/.claude/settings.json" <<JSON
{
  "hooks": {
    "SessionStart": $(hook "$D SessionStart"),
    "UserPromptSubmit": $(hook "$D UserPromptSubmit"),
    "PreCompact": $(hook "$BASE/bin/block.sh"),
    "PostCompact": $(hook "$D PostCompact"),
    "SubagentStop": $(hook "$D SubagentStop"),
    "Stop": $(hook "$D Stop"),
    "SessionEnd": $(hook "$D SessionEnd")
  }
}
JSON

ask() { # NAME ARGS... — one print-mode prompt, its stream saved as NAME.jsonl
  n="$1"; shift
  timeout 1200 "$BASE/bin/run" "$@" --model sonnet --output-format stream-json \
    --verbose </dev/null >"$REC_OUT/$n.jsonl" 2>"$REC_OUT/$n.stderr" || echo "exit $?" >"$REC_OUT/$n.exit"
}
sid() { sed -n 's/.*"session_id":"\([^"]*\)".*/\1/p' "$REC_OUT/$1.jsonl" | head -1; }
WORD="Does any message so far in this conversation give a secret word for this \
check? If so, reply with that word only; otherwise reply with the word none."

compact_blocked() {
  export REC_OUT="$BASE/out/compact-blocked"; mkdir -p "$REC_OUT" "$BASE/w/defer"
  cd "$BASE/w/defer"
  ask 1-prompt -p "Reply with the word ok."
  s=$(sid 1-prompt)
  ask 2-compact --resume "$s" -p "/compact"
  ask 3-word --resume "$s" -p "$WORD"
}

full_window() {
  export REC_OUT="$BASE/out/full-window"; mkdir -p "$REC_OUT" "$BASE/w/defer-full/data"
  cd "$BASE/w/defer-full"
  i=1; while [ $i -le 12 ]; do
    f=$(printf 'data/f%02d.txt' $i)
    { echo "file $i"; od -An -tx2 -N 6000 /dev/urandom; } > "$f"; i=$((i+1))
  done
  export REC_ENV="CLAUDE_CODE_AUTO_COMPACT_WINDOW=100000 \
CLAUDE_AUTOCOMPACT_PCT_OVERRIDE=40 CLAUDE_CODE_BLOCKING_LIMIT_OVERRIDE=60000"
  ask 1-read --allowedTools Read -p "Use the Read tool to read data/f01.txt, then \
data/f02.txt, and so on through data/f12.txt: one file per tool call, in order, \
never two at once. After each, say only its first line. When all are read, \
reply with the word done."
  s=$(sid 1-read)
  ask 2-word --resume "$s" -p "$WORD"
  unset REC_ENV
}

for f in $FLOWS; do
  case "$f" in
    compact-blocked) compact_blocked ;;
    full-window) full_window ;;
    *) echo "unknown flow $f" >&2; exit 2 ;;
  esac
done
