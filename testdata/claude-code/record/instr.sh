#!/bin/sh
# instr.sh — PreCompact: ask the summarizer to carry a fresh nonce, logged to $REC_OUT/instr
cat > "${REC_OUT:?}/precompact-$(date +%s%N).json"; n=$(od -An -N4 -tx1 /dev/urandom | tr -d ' \n'); echo "$n" >> "$REC_OUT/instr"
echo "The summary MUST contain the exact token ZEBRA-$n on its own line."
