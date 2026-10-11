#!/bin/sh
# fixed.sh — SessionStart: answer the same additionalContext every time, except after a compaction
src=$(cat | sed -n 's/.*"source":"\([a-z]*\)".*/\1/p'); echo "$src" >> "${REC_OUT:?}/sources"
[ "$src" = compact ] && exit 0
printf '{"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":"The secret word for this check is %s."}}\n' "$(cat "$REC_OUT/word")"
