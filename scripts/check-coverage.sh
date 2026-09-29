#!/usr/bin/env bash
# Fails when any of the named packages drops below a minimum share of
# its non-excluded statement coverage.
#
# go test -cover measures statement coverage per package but never
# gates on it; this wrapper does, so a change that drops a package
# below its floor reddens CI instead of drifting unnoticed. Called with
# the floor in percent, then the package paths it applies to, e.g.
# `scripts/check-coverage.sh 90 ./internal/redact ./internal/query`
# for ENG-11's security-sensitive floor.
#
# A package may carry a listed, justified exclusion — a pure process
# boundary a unit test cannot reach — in coverage-exclude.txt beside
# this script, one entry per excluded statement range as
# `path/to/file.go:startLine-endLine  # reason`, the path relative to
# the repository root. A package with no matching entries is measured
# exactly as before; this file's own presence changes nothing for it.
set -euo pipefail

exclude_file="$(dirname "$0")/coverage-exclude.txt"

# The exclude file is small and shared by every statement block of
# every gated package, so it is parsed once here into parallel arrays
# rather than re-read from disk on every is_excluded call below.
exclude_files=()
exclude_starts=()
exclude_ends=()
if [ -f "$exclude_file" ]; then
  while IFS= read -r entry; do
    entry="${entry%%#*}"
    # shellcheck disable=SC2086 # word-splitting trims surrounding blanks
    entry=$(echo $entry)
    [ -z "$entry" ] && continue
    erange="${entry#*:}"
    exclude_files+=("${entry%%:*}")
    exclude_starts+=("${erange%-*}")
    exclude_ends+=("${erange#*-}")
  done <"$exclude_file"
fi

# is_excluded reports whether the statement block [start,end] in file
# falls inside a declared exclusion's line range. A block only partly
# inside a declared range is not excluded — the entry must name the
# whole block, so a range widened by hand to cover more than the
# justified boundary is visible in the diff, not silently absorbed.
is_excluded() {
  local file="$1" start="$2" end="$3"
  local i
  for i in "${!exclude_files[@]}"; do
    if [ "$file" = "${exclude_files[$i]}" ] \
      && [ "$start" -ge "${exclude_starts[$i]}" ] \
      && [ "$end" -le "${exclude_ends[$i]}" ]; then
      return 0
    fi
  done

  return 1
}

min="${1:?usage: check-coverage.sh <min-percent> <pkg>...}"
shift

status=0
for pkg in "$@"; do
  profile=$(mktemp)
  # `set -e` would otherwise abort the whole script right here the
  # moment go test exits non-zero — on a compile error or a failing
  # test, not just low coverage — printing nothing and never reaching
  # the remaining packages. The `if !` guard keeps that failure local
  # to this iteration.
  if ! out=$(go test "$pkg" -coverprofile="$profile" 2>&1); then
    echo "$out"
    echo "check-coverage: go test failed for $pkg" >&2
    status=1
    rm -f "$profile"
    continue
  fi
  echo "$out"

  total=0
  covered=0
  # A coverage profile line reads
  # "modpath/file.go:startLine.startCol,endLine.endCol numStmts count",
  # preceded by one "mode: ..." header line.
  while IFS= read -r line; do
    case "$line" in
    mode:*) continue ;;
    esac
    loc="${line%% *}"
    rest="${line#* }"
    nstmts="${rest%% *}"
    count="${rest#* }"
    modfile="${loc%%:*}"
    relfile="${modfile#github.com/jeduden/cairn/}"
    span="${loc#*:}"
    startspan="${span%%,*}"
    endspan="${span#*,}"
    startline="${startspan%%.*}"
    endline="${endspan%%.*}"
    if is_excluded "$relfile" "$startline" "$endline"; then
      continue
    fi
    total=$((total + nstmts))
    if [ "$count" != "0" ]; then
      covered=$((covered + nstmts))
    fi
  done <"$profile"
  rm -f "$profile"

  if [ "$total" -eq 0 ]; then
    echo "check-coverage: could not read coverage for $pkg" >&2
    status=1
    continue
  fi
  # Integer arithmetic, scaled by 100: covered/total >= min/100.
  if [ $((covered * 100)) -lt $((total * min)) ]; then
    pct=$(awk -v c="$covered" -v t="$total" 'BEGIN { printf "%.1f", (c / t) * 100 }')
    echo "check-coverage: $pkg is at $pct% ($covered/$total statements) of its non-excluded statements, want $min%" >&2
    status=1
  fi
done

exit "$status"
