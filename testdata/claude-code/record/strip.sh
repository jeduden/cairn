#!/bin/sh
# strip.sh KEEP FILE... — rewrite recorded transcripts in place: drop every
# skill but those named in KEEP (comma-separated) from skill listings, and
# replace each claude.ai connector's MCP server instructions with a
# placeholder. Only the lines it changes are re-serialized; every other
# line stays byte for byte, so a recording stays a prefix of a later one.
keep="$1"; shift
prog='
def keepset: $keep | split(",");
def kept: . as $e | any(keepset[]; . as $k | $e | startswith("- " + $k + ":"));
def entries: ("\n" + .) | split("\n- ") | .[1:] | map("- " + .);
def strip_list: entries | map(select(kept)) | join("\n");
def strip_rendered:
  (capture("^(?<h>[\\s\\S]*?\\n)(?<b>- [\\s\\S]*?)(?<t>\\n*</system-reminder>[\\s\\S]*)$")
   // {h: ., b: "", t: ""}) as $p
  | $p.h + ($p.b | strip_list) + $p.t;
def connector: gsub("(?<h>## claude\\.ai Connector [A-Z])\\n[\\s\\S]*?(?=\\n## |\\n*</system-reminder>|$)";
                    "\(.h)\n[instructions removed from the fixture]");
if .attachment.type == "skill_listing" then
  .attachment.content |= strip_list
  | .attachment.names |= map(select(IN(keepset[])))
  | .attachment.skillCount = (.attachment.names | length)
  | .rendered |= map(.content |= strip_rendered)
elif .attachment.type == "mcp_instructions_delta" then
  .attachment.addedBlocks |= map(connector)
  | .rendered |= map(.content |= connector)
else . end'
for f in "$@"; do
  tmp="$f.strip"
  while IFS= read -r line || [ -n "$line" ]; do
    case "$line" in
      *'"skill_listing"'*|*'"mcp_instructions_delta"'*)
        printf '%s\n' "$line" | jq -c --arg keep "$keep" "$prog" ;;
      *) printf '%s\n' "$line" ;;
    esac
  done < "$f" > "$tmp" && mv "$tmp" "$f"
done
