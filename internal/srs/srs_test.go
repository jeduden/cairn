package srs

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// srsDir is the specification this package reads in production.
const srsDir = "../../docs/srs"

const reqTable = `# Doc

| ID | Pri | Requirement | Ver | Traces |
|---|---|---|---|---|
| REC-01 | P0 | Ingest. | T | I1 |
| REC-02 | P1 | Link \| parent. | T | I1, I10 |
| REC-03 | P2 | Index. | T | — |
`

func TestParseReadsRequirementRows(t *testing.T) {
	got, err := Parse("doc.md", []byte(reqTable))

	require.NoError(t, err)
	require.Len(t, got, 3)
	assert.Equal(t, Requirement{
		ID: "REC-02", Priority: "P1", Verify: "T", Traces: []string{"I1", "I10"},
		Text: "Link | parent.", Path: "doc.md", Line: 6,
	}, got[1])
	assert.Nil(t, got[2].Traces)
}

func TestParseReadsAssumptionAndNonFunctionalTables(t *testing.T) {
	body := "| ID | Assumption | Verified in |\n|---|---|---|\n| ASM-01 | Hooks carry ids | S1 |\n\n" +
		"| ID | Category | Requirement | Ver |\n|---|---|---|---|\n| NFR-01 | Latency | Fast. | A |\n"

	got, err := Parse("doc.md", []byte(body))

	require.NoError(t, err)
	require.Len(t, got, 2)
	assert.Equal(t, "Hooks carry ids", got[0].Text)
	assert.Empty(t, got[0].Priority)
	assert.Equal(t, "A", got[1].Verify)
	assert.Empty(t, got[1].Priority)
}

func TestParsePassesOtherTablesBy(t *testing.T) {
	body := "| ID | Constraint |\n|---|---|\n| CON-01 | Go. |\n\n" +
		"| # | Requirement |\n|---|---|\n| X | y |\n"

	got, err := Parse("doc.md", []byte(body))

	require.NoError(t, err)
	assert.Empty(t, got)
}

func TestParseRejectsMalformedIDsAndTraces(t *testing.T) {
	_, err := Parse("doc.md", []byte("| ID | Requirement |\n|---|---|\n| rec-1 | x |\n"))
	assert.ErrorContains(t, err, `doc.md:3: malformed requirement id "rec-1"`)

	_, err = Parse("doc.md", []byte("| ID | Requirement | Traces |\n|---|---|---|\n| REC-01 | x | I11 |\n"))
	assert.ErrorContains(t, err, `doc.md:3: REC-01: malformed trace "I11"`)
}

func TestParseToleratesShortRows(t *testing.T) {
	got, err := Parse("doc.md", []byte("| ID | Requirement | Traces |\n|---|---|---|\n| REC-01 |\n"))

	require.NoError(t, err)
	require.Len(t, got, 1)
	assert.Empty(t, got[0].Text)
}

func TestLoadReadsEveryFileAndRejectsDuplicates(t *testing.T) {
	dir := t.TempDir()
	write := func(name, body string) {
		require.NoError(t, os.WriteFile(filepath.Join(dir, name), []byte(body), 0o600))
	}
	write("a.md", reqTable)
	write("b.md", "| ID | Requirement |\n|---|---|\n| SEC-01 | x |\n")

	got, err := Load(dir)
	require.NoError(t, err)
	assert.Len(t, got, 4)

	write("c.md", "| ID | Requirement |\n|---|---|\n| SEC-01 | again |\n")
	_, err = Load(dir)
	assert.ErrorContains(t, err, "SEC-01 already defined at")
}

func TestLoadReportsParseAndReadErrors(t *testing.T) {
	dir := t.TempDir()
	bad := []byte("| ID | Requirement |\n|---|---|\n| X |\n")
	require.NoError(t, os.WriteFile(filepath.Join(dir, "a.md"), bad, 0o600))
	_, err := Load(dir)
	assert.ErrorContains(t, err, "malformed requirement id")

	_, err = Load("[")
	assert.ErrorContains(t, err, "srs: list")

	dir = t.TempDir()
	require.NoError(t, os.Mkdir(filepath.Join(dir, "dir.md"), 0o700))
	_, err = Load(dir)
	assert.ErrorContains(t, err, "srs: read")
}

// TestSpecificationParses is the gate on the real specification:
// every requirement table under docs/srs reads cleanly, and every id
// is unique across the whole document.
func TestSpecificationParses(t *testing.T) {
	reqs, err := Load(srsDir)

	require.NoError(t, err)
	assert.Len(t, reqs, 100+14+27+10, "§5–§6, NFR, ENG and ASM rows")
}
