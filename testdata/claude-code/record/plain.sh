#!/bin/sh
# plain.sh EVENT — print a fresh nonce as plain stdout, logged to $REC_OUT/plain-EVENT
cat > /dev/null; n=$(od -An -N6 -tx1 /dev/urandom | tr -d ' \n'); echo "$n" >> "${REC_OUT:?}/plain-$1"
echo "The secret word for this check is $n."
