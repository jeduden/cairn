#!/bin/sh
# Installs the pinned tools: the one install path for CI's setup action,
# a Claude Code web environment and a developer machine.
#
# It downloads the mise binary pinned in scripts/mise.sha256 (one
# sha256sum line per platform) and refuses it unless `sha256sum -c`
# passes. It then runs `mise install --locked`, which checks each tool
# against mise.lock (and mise.dev.lock under MISE_ENV=dev) and verifies
# each attestation the lockfile records. It only downloads and verifies:
# a tool whose backend would build from source is refused. It fails
# closed: a missing pin, an unsupported platform or a mismatch stops it
# before anything downloaded runs.
#
# usage: scripts/setup-tools.sh [TOOL...]
#   With no TOOL, installs every tool of the active configuration.
#   MISE_ENV=dev adds the developer tools of mise.dev.toml.
#   Under GitHub Actions it exports PATH and mise's environment to the
#   job's later steps; elsewhere it adds them to ~/.bashrc.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
sums=$root/scripts/mise.sha256
bindir=${MISE_BIN_DIR:-$HOME/.local/bin}

case "$(uname -s)/$(uname -m)" in
  Linux/x86_64) platform=linux-x64 ;;
  Linux/aarch64 | Linux/arm64) platform=linux-arm64 ;;
  Darwin/x86_64) platform=macos-x64 ;;
  Darwin/arm64) platform=macos-arm64 ;;
  *) echo "setup-tools: unsupported platform $(uname -s)/$(uname -m)" >&2; exit 1 ;;
esac

pin=$(grep -E "  mise-v[0-9.]+-$platform\$" "$sums" || true)
if [ -z "$pin" ]; then
  echo "setup-tools: no mise pin for $platform in $sums" >&2
  exit 1
fi
file=${pin##* }
version=${file#mise-v}
version=${version%-"$platform"}

# Every tool must come from a backend that installs a released binary.
# The Rust toolchain comes from rust-toolchain.toml through rustup.
building=$(awk '
  FNR == 1 { tools = 0 }
  /^\[/ { tools = ($0 == "[tools]"); next }
  tools && /=/ {
    key = $0; sub(/[ \t]*=.*/, "", key); gsub(/"/, "", key)
    if (key !~ /^(github|aqua):/) print FILENAME ": " key
  }' "$root"/mise*.toml)
if [ -n "$building" ]; then
  echo "setup-tools: refusing tools that are not released binaries:" >&2
  echo "$building" >&2
  exit 1
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
url=https://github.com/jdx/mise/releases/download/v$version/$file
curl --proto '=https' --tlsv1.2 -fsSL --retry 3 --max-time 120 -o "$tmp/$file" "$url"

if command -v sha256sum >/dev/null; then check="sha256sum -c"; else check="shasum -a 256 -c"; fi
if ! (cd "$tmp" && echo "$pin" | $check >/dev/null); then
  echo "setup-tools: mise $version does not match $sums" >&2
  exit 1
fi

mkdir -p "$bindir"
install -m 0755 "$tmp/$file" "$bindir/mise"
echo "setup-tools: installed mise $version (${pin%% *})"

PATH=$bindir:$PATH
export PATH
cd "$root"
for config in mise*.toml; do mise trust --yes "$config"; done
mise install --locked "$@"

if [ -n "${GITHUB_ENV:-}" ] && [ -n "${GITHUB_PATH:-}" ]; then
  # The job's later steps: mise itself, each installed tool's bin
  # directory (mise env's PATH entries that are new) and mise's other
  # variables. GITHUB_PATH prepends, so add the entries last to first.
  mise env --json > "$tmp/env.json"
  jq -r '.PATH' "$tmp/env.json" | awk -v cur="$PATH" '
    BEGIN { n = split(cur, c, ":"); for (i = 1; i <= n; i++) seen[c[i]] = 1 }
    { m = split($0, p, ":"); for (i = m; i >= 1; i--) if (!(p[i] in seen)) print p[i] }
  ' >> "$GITHUB_PATH"
  echo "$bindir" >> "$GITHUB_PATH"
  jq -r 'del(.PATH) | to_entries[] | "\(.key)=\(.value)"' "$tmp/env.json" >> "$GITHUB_ENV"
  echo "MISE_YES=1" >> "$GITHUB_ENV"
else
  # Later shells (an agent's tool calls) find mise and the pinned tools.
  profile=$HOME/.bashrc
  marker='# parleyinn setup-tools'
  if ! grep -qF "$marker" "$profile" 2>/dev/null; then
    {
      echo "$marker"
      if [ -n "${MISE_ENV:-}" ]; then echo "export MISE_ENV=$MISE_ENV"; fi
      echo "export PATH=\"$bindir:\$PATH\""
      echo "eval \"\$(mise activate bash)\""
    } >> "$profile"
  fi
fi
echo "setup-tools: done; run \`mise tasks\` to list the checks"
