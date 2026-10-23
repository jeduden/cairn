#!/bin/sh
# Sets up a Claude Code web environment: installs the mise binary pinned
# in CI's setup action, verified against its SHA-256, then installs every
# tool in mise.toml from mise.lock with `mise install --locked`. It only
# downloads and verifies; a tool that would build from source is refused.
#
# The version and checksums live only in
# .github/actions/setup-mise/action.yml; this script reads them from
# there, so a bump lands in one place. It fails closed: a missing pin, an
# unsupported platform or a checksum mismatch stops the setup before the
# downloaded file runs.
#
# usage: scripts/setup-web.sh   (set it as the environment's setup script)
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
action=$root/.github/actions/setup-mise/action.yml
bindir=${MISE_BIN_DIR:-$HOME/.local/bin}

case "$(uname -s)/$(uname -m)" in
  Linux/x86_64) ci=Linux/X64 asset=linux-x64 ;;
  Linux/aarch64 | Linux/arm64) ci=Linux/ARM64 asset=linux-arm64 ;;
  Darwin/x86_64) ci=macOS/X64 asset=macos-x64 ;;
  Darwin/arm64) ci=macOS/ARM64 asset=macos-arm64 ;;
  *) echo "setup-web: unsupported platform $(uname -s)/$(uname -m)" >&2; exit 1 ;;
esac

version=$(sed -n 's/.*echo "version=\([0-9.]*\)".*/\1/p' "$action")
sha256=$(sed -n "s|^ *$ci) sha256=\([0-9a-f]\{64\}\) ;;\$|\1|p" "$action")
if [ -z "$version" ] || [ -z "$sha256" ]; then
  echo "setup-web: no mise pin for $ci in $action" >&2
  exit 1
fi

# Setup only downloads and verifies; it never builds. Every tool must
# come from a backend that installs a released binary, checked against
# mise.lock (and its attestation where the lockfile records one). The
# Rust toolchain comes from rust-toolchain.toml through rustup.
building=$(awk '
  /^\[/ { tools = ($0 == "[tools]"); next }
  tools && /=/ {
    key = $0; sub(/[ \t]*=.*/, "", key); gsub(/"/, "", key)
    if (key !~ /^(github|aqua):/) print key
  }' "$root/mise.toml")
if [ -n "$building" ]; then
  echo "setup-web: refusing tools that are not released binaries:" $building >&2
  exit 1
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
url=https://github.com/jdx/mise/releases/download/v$version/mise-v$version-$asset
curl --proto '=https' --tlsv1.2 -fsSL --retry 3 --max-time 120 -o "$tmp/mise" "$url"

got=$(sha256sum "$tmp/mise" 2>/dev/null || shasum -a 256 "$tmp/mise")
got=${got%% *}
if [ "$got" != "$sha256" ]; then
  echo "setup-web: mise $version checksum mismatch: got $got, want $sha256" >&2
  exit 1
fi

mkdir -p "$bindir"
install -m 0755 "$tmp/mise" "$bindir/mise"
echo "setup-web: installed mise $version ($sha256)"

PATH=$bindir:$PATH
export PATH
cd "$root"
mise trust --yes mise.toml
mise install --locked

# Later shells (the agent's tool calls) find mise and the pinned tools.
# `mise activate` puts the tools on PATH, which the profile edit persists.
profile=$HOME/.bashrc
marker='# parleyinn setup-web'
if ! grep -qF "$marker" "$profile" 2>/dev/null; then
  {
    echo "$marker"
    echo "export PATH=\"$bindir:\$PATH\""
    echo "eval \"\$(mise activate bash)\""
  } >> "$profile"
fi
echo "setup-web: done; run \`mise tasks\` to list the checks"
