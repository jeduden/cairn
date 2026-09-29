package adr

import (
	"maps"
	"os"
	"path/filepath"
	"slices"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// The real decision records the ENG-18 and ENG-26 scenarios read.
const adrDir = "../../docs/adr"

const sample = `---
id: ADR-2609292234
title: "The test stack"
status: 'accepted'
summary: >-
  Scenarios run through godog,
  with testify's assertions.
superseded-by: ADR-2609300000
---
# ADR-2609292234: The test stack

## Context

Why.

## Decision

| Module                      | Purpose | License | Maintenance |
| --------------------------- | ------- | ------- | ----------- |
| ` + "`example.com/one`" + ` | tests   | MIT     | active      |
| example.com/two             | tests   | ISC     |

## Alternatives

None worth it.
`

func TestParseReadsFrontMatterSectionsAndModules(t *testing.T) {
	a, err := Parse("docs/adr/ADR-2609292234-test-stack.md", []byte(sample))

	require.NoError(t, err)
	assert.Equal(t, "ADR-2609292234", a.ID)
	assert.Equal(t, "The test stack", a.Title)
	assert.Equal(t, Accepted, a.Status)
	assert.Equal(t, "Scenarios run through godog, with testify's assertions.", a.Summary)
	assert.Equal(t, "ADR-2609300000", a.SupersededBy)
	assert.Equal(t, "Why.", a.Sections["Context"])
	assert.Equal(t, "None worth it.", a.Sections["Alternatives"])
	assert.Equal(t, []Module{
		{Path: "example.com/one", Purpose: "tests", License: "MIT", Maintenance: "active"},
		{Path: "example.com/two", Purpose: "tests", License: "ISC"},
	}, a.Modules)
}

func TestParseWithoutModuleTableHasNoModules(t *testing.T) {
	body := "---\nid: ADR-01\n---\n# ADR-01\n\n## Decision\n\n| A | B |\n| - | - |\n| 1 | 2 |\n"
	a, err := Parse("a.md", []byte(body))

	require.NoError(t, err)
	assert.Empty(t, a.Modules)
	assert.Equal(t, []string{"Decision"}, slices.Collect(maps.Keys(a.Sections)))
}

func TestParseRefusesWhatItCannotRead(t *testing.T) {
	cases := []struct{ body, want string }{
		{"# ADR-01\n", "no front matter"},
		{"---\nid: ADR-01\n", "no front matter"},
		{"---\nmodules:\n  - example.com/one\n---\n", "line 3: unsupported front matter"},
		{"---\nid ADR-01\n---\n", "line 2: malformed front matter"},
		{"---\n: x\n---\n", "line 2: malformed front matter"},
	}
	for _, c := range cases {
		_, err := Parse("a.md", []byte(c.body))
		assert.ErrorContains(t, err, "adr: a.md: "+c.want, c.body)
	}
}

func TestParseSkipsBlankLinesAndKeepsQuotesItCannotPair(t *testing.T) {
	a, err := Parse("a.md", []byte("---\nid: ADR-01\n\ntitle: \"half\n---\n"))

	require.NoError(t, err)
	assert.Equal(t, "ADR-01", a.ID)
	assert.Equal(t, `"half`, a.Title)
}

func TestLoadReadsEveryFileInNameOrder(t *testing.T) {
	dir := t.TempDir()
	require.NoError(t, os.WriteFile(filepath.Join(dir, "ADR-02-b.md"), []byte("---\nid: ADR-02\n---\n"), 0o600))
	require.NoError(t, os.WriteFile(filepath.Join(dir, "ADR-01-a.md"), []byte("---\nid: ADR-01\n---\n"), 0o600))

	adrs, err := Load(dir)

	require.NoError(t, err)
	require.Len(t, adrs, 2)
	assert.Equal(t, "ADR-01", adrs[0].ID)
	assert.Equal(t, filepath.Join(dir, "ADR-02-b.md"), adrs[1].Path)
}

func TestLoadReportsParseReadAndGlobErrors(t *testing.T) {
	dir := t.TempDir()
	require.NoError(t, os.WriteFile(filepath.Join(dir, "bad.md"), []byte("# no front matter\n"), 0o600))
	_, err := Load(dir)
	assert.ErrorContains(t, err, "no front matter")

	unreadable := t.TempDir()
	require.NoError(t, os.Mkdir(filepath.Join(unreadable, "dir.md"), 0o700))
	_, err = Load(unreadable)
	assert.ErrorContains(t, err, "adr: read")

	_, err = Load("[")
	assert.ErrorContains(t, err, "adr: list")
}

// TestDecisionRecordsParse is the gate on the real records: every file
// under docs/adr reads cleanly.
func TestDecisionRecordsParse(t *testing.T) {
	adrs, err := Load(adrDir)

	require.NoError(t, err)
	assert.NotEmpty(t, adrs)
}

func TestParseReadsModulesOnlyFromTheDecision(t *testing.T) {
	body := "---\nid: ADR-01\n---\n# ADR-01\n\n## Decision\n\n| Module | Purpose |\n| - | - |\n| `a` | p |\n\n" +
		"## Alternatives\n\n| Module | Why not |\n| - | - |\n| `b` | slow |\n"
	a, err := Parse("a.md", []byte(body))

	require.NoError(t, err)
	assert.Equal(t, []Module{{Path: "a", Purpose: "p"}}, a.Modules)
}

func TestParseKeepsFencedHeadingsInsideTheirSection(t *testing.T) {
	body := "---\nid: ADR-01\n---\n## Alternatives\n\nOne.\n\n```md\n## Not a section\n```\n\nTwo.\n"
	a, err := Parse("a.md", []byte(body))

	require.NoError(t, err)
	assert.Equal(t, "One.\n\n```md\n## Not a section\n```\n\nTwo.", a.Sections["Alternatives"])
	assert.NotContains(t, a.Sections, "Not a section")
}

func TestParseReadsCRLFLineEndings(t *testing.T) {
	a, err := Parse("a.md", []byte("---\r\nid: ADR-01\r\nstatus: accepted\r\n---\r\n## Context\r\n\r\nWhy.\r\n"))

	require.NoError(t, err)
	assert.Equal(t, "ADR-01", a.ID)
	assert.Equal(t, Accepted, a.Status)
	assert.Equal(t, "Why.", a.Sections["Context"])
}

func TestParseRefusesAFrontMatterLineItCannotScan(t *testing.T) {
	long := "---\nsummary: " + strings.Repeat("x", 2<<20) + "\nstatus: accepted\n---\n"
	_, err := Parse("a.md", []byte(long))

	assert.ErrorContains(t, err, "adr: a.md: scan front matter")
}

func TestFenceAfter(t *testing.T) {
	assert.Empty(t, fenceAfter("", "## Heading\n"))
	assert.Equal(t, "````", fenceAfter("", "````md\n"))
	assert.Equal(t, "````", fenceAfter("````", "```\n"), "a shorter run does not close")
	assert.Equal(t, "````", fenceAfter("````", "~~~~\n"), "another character does not close")
	assert.Equal(t, "````", fenceAfter("````", "text\n"))
	assert.Empty(t, fenceAfter("````", "`````\n"))
}
