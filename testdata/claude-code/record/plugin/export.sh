#!/bin/sh
# export.sh BASE DEST [TARGET...] — spike S4: copy what record.sh recorded under BASE into
# fixtures under DEST/<target>: the hook inputs and what each handler saw,
# numbered in firing order; the MCP server's log; the harness session's
# transcripts; the init message of the stream, trimmed to the plugin's view;
# and the install output. Then redact scratch paths as ../redact.sh does.
set -eu
BASE="${1:?scratch directory record.sh wrote}"; DEST="${2:?fixture directory}"
shift 2
TARGETS="${*:-workstation runner-image sdk-worker interactive launch-key forged-env}"
KIT=$(cd "$(dirname "$0")/.." && pwd)
for t in $TARGETS; do
  src="$BASE/$t"; out="$DEST/$t"; rm -rf "$out"; mkdir -p "$out/hooks" "$out/transcripts"
  n=0
  for f in $(ls "$src/out/hooks" | grep '\.json$' | sort); do
    n=$((n + 1)); name=$(printf '%02d-%s' "$n" "${f#*-}")
    cp "$src/out/hooks/$f" "$out/hooks/$name"
    cp "$src/out/hooks/${f%.json}.env" "$out/hooks/${name%.json}.env"
  done
  cat "$src/out/mcp/"*.jsonl >"$out/mcp.jsonl"
  [ -f "$src/out/install.txt" ] && cp "$src/out/install.txt" "$out/"
  # one project directory, named for the work directory: /w/ws once redacted
  proj=$(ls -d "$src/home/.claude/projects/"*/)
  (cd "$proj" && find . -name '*.jsonl' -o -name '*.meta.json') |
    while read -r p; do
      mkdir -p "$out/transcripts/-w-ws/$(dirname "$p")"; cp "$proj/$p" "$out/transcripts/-w-ws/$p"
    done
  if [ -f "$src/out/stream.jsonl" ]; then
    python3 - "$src/out/stream.jsonl" >"$out/init.json" <<'PY'
import json, sys
keep = ("type", "subtype", "claude_code_version", "permissionMode", "tools",
        "mcp_servers", "plugins")
for line in open(sys.argv[1]):
    message = json.loads(line)
    if message.get("type") == "system" and message.get("subtype") == "init":
        print(json.dumps({k: message[k] for k in keep if k in message}, indent=2))
        break
PY
  fi
  files=$(find "$out" -type f)
  # shellcheck disable=SC2086
  sed -i -e "s#$src/src#/opt/rec/plugin#g" $files
  # shellcheck disable=SC2086
  "$KIT/redact.sh" "$src" $files
done
