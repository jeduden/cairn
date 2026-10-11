#!/bin/sh
# slowctx.sh SECONDS — answer SessionStart with additionalContext after a delay
cat > /dev/null; sleep "$1"
printf '{"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":"CAIRN-SLOW-%s-%s"}}\n' "$1" "${REC_TAG:-x}"
