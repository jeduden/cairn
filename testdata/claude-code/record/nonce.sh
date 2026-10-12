#!/bin/sh
# nonce.sh EVENT — answer with additionalContext holding a fresh nonce, logged to $REC_OUT/nonce-EVENT
cat > /dev/null; sleep "${NONCE_DELAY:-0}"; n=$(od -An -N6 -tx1 /dev/urandom | tr -d ' \n')
echo "$n" >> "${REC_OUT:?}/nonce-$1"
printf '{"hookSpecificOutput":{"hookEventName":"%s","additionalContext":"The secret word for this check is %s."}}\n' "$1" "$n"
