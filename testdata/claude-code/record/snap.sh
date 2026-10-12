#!/bin/sh
# snap.sh EVENT — save the hook input, then a snapshot of every transcript of
# its harness session as the hook fires: lines, compaction boundaries, last
# line's type. For SessionStart, answer with additionalContext naming a nonce.
ev="$1"; out="${REC_OUT:?}"; t=$(date +%s%N); f="$out/$t-$ev"
cat > "$f.json"
tp=$(sed -n 's/.*"transcript_path":"\([^"]*\)".*/\1/p' "$f.json")
dir="${tp%.jsonl}"
for x in "$tp" "$dir"/subagents/*.jsonl; do
  [ -f "$x" ] || continue
  printf '%s lines=%s boundaries=%s last=%s\n' "$(basename "$x")" "$(wc -l < "$x")" \
    "$(grep -c '"compact_boundary"' "$x")" \
    "$(tail -1 "$x" | sed -n 's/.*"type":"\([a-z_-]*\)".*/\1/p' | head -1)"
done > "$f.snap"
if [ "$ev" = SessionStart ]; then
  n=$(od -An -N4 -tx1 /dev/urandom | tr -d ' \n'); echo "$n" > "$f.nonce"
  printf '{"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":"RESTORE-%s"}}\n' "$n"
fi
exit 0
