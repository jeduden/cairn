package srs

import (
	"os"
	"path/filepath"
	"slices"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

const appendixB = `# B

| Other | Table |
|---|---|
| x | y |

| Invariant | Requirements |
|---|---|
| **I1** — Nothing is lost | REC-01, SEC-08 |
| **I10** — Rebuildable | REC-03 |

| Priority | Functional | Engineering |
|---|---|---|
| P0 | 3 | 1 |
| P1 | 0 | 0 |
| — | Non-functional: 14 | |
`

func TestInvariantCoverageReadsTheTable(t *testing.T) {
	got, err := InvariantCoverage([]byte(appendixB))

	require.NoError(t, err)
	assert.Equal(t, map[string][]string{"I1": {"REC-01", "SEC-08"}, "I10": {"REC-03"}}, got)
}

func TestInvariantCoverageRejectsMalformedRows(t *testing.T) {
	_, err := InvariantCoverage([]byte("| Invariant | Requirements |\n|---|---|\n| I1 | REC-01 |\n"))
	assert.ErrorContains(t, err, "line 3: malformed invariant row")

	_, err = InvariantCoverage([]byte("| Invariant | Requirements |\n|---|---|\n| **I1** |\n"))
	assert.ErrorContains(t, err, "malformed invariant row")

	_, err = InvariantCoverage([]byte("no table"))
	assert.ErrorContains(t, err, "no invariant coverage table")
}

func TestStatedCountsReadsPriorityRows(t *testing.T) {
	got, err := StatedCounts([]byte(appendixB))

	require.NoError(t, err)
	assert.Equal(t, map[string]map[string]int{
		"functional":  {"P0": 3},
		"engineering": {"P0": 1},
	}, got)
}

func TestStatedCountsRejectsMalformedRows(t *testing.T) {
	_, err := StatedCounts([]byte("| Priority | F | E |\n|---|---|---|\n| P0 | x | 1 |\n"))
	assert.ErrorContains(t, err, "line 3: malformed count row")

	_, err = StatedCounts([]byte("| Priority | F | E |\n|---|---|---|\n| P0 | 1 |\n"))
	assert.ErrorContains(t, err, "malformed count row")

	_, err = StatedCounts([]byte("none"))
	assert.ErrorContains(t, err, "no requirement count table")
}

func TestTracedCoverageAndCountsSkipOutOfScopeFamilies(t *testing.T) {
	reqs := []Requirement{
		{ID: "REC-01", Priority: "P0", Traces: []string{"I1"}},
		{ID: "SEC-08", Priority: "P0", Traces: []string{"I1"}},
		{ID: "REC-03", Priority: "P0", Traces: []string{"I10"}},
		{ID: "ENG-01", Priority: "P0", Traces: []string{"I1"}},
		{ID: "NFR-01"},
	}

	assert.Equal(t, map[string][]string{"I1": {"REC-01", "SEC-08"}, "I10": {"REC-03"}}, TracedCoverage(reqs))
	assert.Equal(t, map[string]map[string]int{
		"functional":  {"P0": 3},
		"engineering": {"P0": 1},
	}, PriorityCounts(reqs))
}

// TestAppendixBMatchesTheTraces keeps Appendix B honest: it says it is
// generated from the Traces column of §5–§6, so an edit to a trace
// that forgets the appendix, or the other way round, fails here.
func TestAppendixBMatchesTheTraces(t *testing.T) {
	reqs, err := Load(srsDir)
	require.NoError(t, err)
	body, err := os.ReadFile(filepath.Join(srsDir, "appendix-b-invariant-coverage.md"))
	require.NoError(t, err)

	stated, err := InvariantCoverage(body)
	require.NoError(t, err)
	traced := TracedCoverage(reqs)

	require.Len(t, stated, 10, "every invariant I1–I10 has a row")
	for inv, ids := range traced {
		want := slices.Sorted(slices.Values(ids))
		got := slices.Sorted(slices.Values(stated[inv]))
		assert.Equal(t, want, got, "Appendix B row %s", inv)
	}
	assert.Len(t, traced, len(stated), "every invariant is traced by some requirement")

	counts, err := StatedCounts(body)
	require.NoError(t, err)
	assert.Equal(t, counts, PriorityCounts(reqs))
}
