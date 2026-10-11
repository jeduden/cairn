#!/bin/sh
# redact.sh REC_ROOT FILE... — rewrite recorded fixtures in place: scratch paths to
# /w, /home/u and /opt/rec; the account's email, organization and claude.ai
# connector names to placeholders.
# Usage after recording: redact.sh "$REC_ROOT" $(find fixtures -type f)
S="$1"; shift
SLUG=$(printf '%s' "$S/w/" | sed 's#[^A-Za-z0-9]#-#g')
for f in "$@"; do
  sed -i \
    -e "s#$SLUG#-w-#g" \
    -e "s#$S/home/.claude#/home/u/.claude#g" \
    -e "s#$S/cfg-timeout#/home/u/.claude#g" \
    -e "s#$S/home#/home/u#g" \
    -e "s#$S/w/#/w/#g" \
    -e "s#$S/bin/#/opt/rec/bin/#g" \
    -e "s#$S/exp/[A-Za-z0-9._-]*#/opt/rec/out#g" \
    -e "s#$S/out#/opt/rec/out#g" \
    -e 's#[A-Za-z0-9._%+-]*@gmail\.com#user@example.com#g' \
    -e 's#"organizationUuid":"[0-9a-f-]*"#"organizationUuid":"00000000-0000-0000-0000-000000000000"#g' \
    -e 's#Linux [0-9][0-9.]*+deb[0-9]*-amd64#Linux#g' \
    -e 's#/tmp/claude-[0-9]*/#/tmp/claude-u/#g' \
    -e 's#Google Calendar#Connector B#g; s#Google_Calendar#Connector_B#g' \
    -e 's#Google Drive#Connector C#g; s#Google_Drive#Connector_C#g' \
    -e 's#Claude Docs#Connector D#g; s#Claude_Docs#Connector_D#g; s#Gmail#Connector A#g' \
    "$f"
done
