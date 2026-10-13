#!/bin/sh
# export-defer.sh BASE DEST — spike S6: copy what defer.sh recorded under BASE
# into fixtures under DEST: each flow's hook inputs, numbered in firing order,
# under DEST/hooks/<flow>, and the transcripts under DEST/transcripts/projects;
# then redact them with redact.sh and strip them with strip.sh, keeping no
# skill. The words block.sh wrote stay in BASE/out/<flow>/block, for the
# manifest.
set -eu
BASE="${1:?scratch directory defer.sh wrote}"; DEST="${2:?fixture directory}"
KIT=$(cd "$(dirname "$0")" && pwd)
SLUG=$(printf '%s' "$BASE/w/" | sed 's#[^A-Za-z0-9]#-#g')
for flow in compact-blocked full-window; do
  src="$BASE/out/$flow"; out="$DEST/hooks/$flow"; rm -rf "$out"; mkdir -p "$out"
  n=0
  for f in $(ls "$src" | grep -E '^[0-9]+-[A-Za-z]+\.json$' | sort); do
    n=$((n + 1)); cp "$src/$f" "$out/$(printf '%02d-%s' "$n" "${f#*-}")"
  done
done
for d in "$BASE/home/.claude/projects/$SLUG"*; do
  out="$DEST/transcripts/projects/-w-${d#"$BASE/home/.claude/projects/$SLUG"}"
  rm -rf "$out"; mkdir -p "$out"; cp "$d"/*.jsonl "$out/"
done
files=$(find "$DEST/hooks/compact-blocked" "$DEST/hooks/full-window" \
  "$DEST/transcripts/projects/-w-defer" "$DEST/transcripts/projects/-w-defer-full" -type f)
# shellcheck disable=SC2086
"$KIT/redact.sh" "$BASE" $files
# shellcheck disable=SC2086
"$KIT/strip.sh" "" $(printf '%s\n' $files | grep '\.jsonl$')
