#!/bin/sh
# stderr.sh EVENT CODE — write a fresh nonce to stderr, log it to $REC_OUT/stderr-EVENT, exit CODE
cat > /dev/null; n=$(od -An -N6 -tx1 /dev/urandom | tr -d ' \n'); echo "$n" >> "${REC_OUT:?}/stderr-$1"
echo "The secret word for this check is $n." >&2; exit "$2"
