#!/bin/sh
# Sets up a Claude Code web environment: installs the mise binary pinned
# in CI's setup action, verified against its SHA-256, then installs every
# tool in mise.toml from mise.lock with `mise install --locked`. It only
# downloads and verifies; a tool that would build from source is refused.
#
# The mise pin lives only in .github/actions/setup-mise/mise.sha256, one
# sha256sum line per platform, which CI's setup action reads too, so a
# bump lands in one place. It fails closed: a missing pin, an
# unsupported platform or a checksum mismatch stops the setup before the
# downloaded file runs.
#
# usage: scripts/setup-web.sh   (set it as the environment's setup script)
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
sums=$root/.github/actions/setup-mise/mise.sha256
bindir=${MISE_BIN_DIR:-$HOME/.local/bin}

case "$(uname -s)/$(uname -m)" in
  Linux/x86_64) platform=linux-x64 ;;
  Linux/aarch64 | Linux/arm64) platform=linux-arm64 ;;
  Darwin/x86_64) platform=macos-x64 ;;
  Darwin/arm64) platform=macos-arm64 ;;
  *) echo "setup-web: unsupported platform $(uname -s)/$(uname -m)" >&2; exit 1 ;;
esac

pin=$(grep -E "  mise-v[0-9.]+-$platform\$" "$sums" || true)
if [ -z "$pin" ]; then
  echo "setup-web: no mise pin for $platform in $sums" >&2
  exit 1
fi
file=${pin##* }
version=${file#mise-v}
version=${version%-"$platform"}

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
url=https://github.com/jdx/mise/releases/download/v$version/$file
curl --proto '=https' --tlsv1.2 -fsSL --retry 3 --max-time 120 -o "$tmp/$file" "$url"

# sha256sum -c refuses the file unless it matches the pinned line.
if command -v sha256sum >/dev/null; then check="sha256sum -c"; else check="shasum -a 256 -c"; fi
if ! (cd "$tmp" && echo "$pin" | $check >/dev/null); then
  echo "setup-web: mise $version does not match $sums" >&2
  exit 1
fi

mkdir -p "$bindir"
install -m 0755 "$tmp/$file" "$bindir/mise"
echo "setup-web: installed mise $version (${pin%% *})"

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
