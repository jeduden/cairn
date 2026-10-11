#!/bin/sh
# dump.sh EVENT [MODE] — save the hook's stdin, then optionally answer with additionalContext
ev="$1"; mode="$2"; out="${REC_OUT:?}"
f="$out/$(date +%s%N)-$ev.json"
cat > "$f"
case "$mode" in
  ctx) printf '{"hookSpecificOutput":{"hookEventName":"%s","additionalContext":"CAIRN-MARKER-%s-%s"}}\n' "$ev" "$ev" "${REC_TAG:-x}" ;;
esac
exit 0
