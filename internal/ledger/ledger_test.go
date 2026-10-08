package ledger

import (
	"io/fs"
	"os"
	"testing"
	"testing/fstest"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// The repository root and the ledger the domain-model rounds keep.
const (
	root       = "../.."
	ledgerPath = "plan/2610012322_cairn-for-agent-fleets/finding-ledger.json"
)

const sample = `{"themes": [
  {"id": "T-1", "title": "fixed one", "status": "fixed",
   "texts": [{"file": "docs/a.md", "quote": "the rule  stands"}]},
  {"id": "T-2", "title": "still open", "status": "open"}
]}`

func fixed(id string, texts ...Text) Theme {
	return Theme{ID: id, Title: id, Status: Fixed, Texts: texts}
}

func TestParseReadsThemes(t *testing.T) {
	l, err := Parse([]byte(sample))
	require.NoError(t, err)
	require.Len(t, l.Themes, 2)
	assert.Equal(t, Fixed, l.Themes[0].Status)
	assert.Equal(t, []Text{{File: "docs/a.md", Quote: "the rule  stands"}}, l.Themes[0].Texts)
	assert.Equal(t, Open, l.Themes[1].Status)
}

func TestParseRefusesWhatItCannotRead(t *testing.T) {
	_, err := Parse([]byte(`{"themes": [`))
	require.ErrorContains(t, err, "ledger: parse")
}

func TestValidateAcceptsEveryWellFormedClosure(t *testing.T) {
	x := Text{File: "docs/a.md", Quote: "q"}
	l := Ledger{Themes: []Theme{
		{ID: "T-1", Status: Open},
		fixed("T-2", x),
		{ID: "T-3", Status: Deferred, OQ: "OQ-42", Milestone: "M7", Texts: []Text{x}},
		{ID: "T-4", Status: Accepted, Risk: "R7", Texts: []Text{x}},
	}}
	assert.Empty(t, l.Validate())
}

func TestValidateReportsEachMalformedTheme(t *testing.T) {
	x := Text{File: "docs/a.md", Quote: "q"}
	cases := []struct {
		theme Theme
		want  string
	}{
		{Theme{Title: "nameless", Status: Open}, `theme "nameless" has no id`},
		{Theme{ID: "T-1", Status: "done"}, `T-1: unknown status "done"`},
		{Theme{ID: "T-2", Status: Deferred, OQ: "OQ-x", Milestone: "M7", Texts: []Text{x}},
			"T-2: deferred without an OQ-n and an Mn milestone"},
		{Theme{ID: "T-3", Status: Deferred, OQ: "OQ-1", Texts: []Text{x}},
			"T-3: deferred without an OQ-n and an Mn milestone"},
		{Theme{ID: "T-4", Status: Accepted, Risk: "risk", Texts: []Text{x}},
			"T-4: accepted without a residual risk Rn"},
		{Theme{ID: "T-5", Status: Fixed}, "T-5: fixed but cites no text"},
		{fixed("T-6", Text{File: "docs/a.md", Quote: "  "}), "T-6: a cited text needs a file and a quote"},
		{fixed("T-7", Text{Quote: "q"}), "T-7: a cited text needs a file and a quote"},
	}
	for _, c := range cases {
		errs := Ledger{Themes: []Theme{c.theme}}.Validate()
		require.Len(t, errs, 1, c.want)
		assert.EqualError(t, errs[0], c.want)
	}
}

func TestValidateRefusesADuplicateID(t *testing.T) {
	x := Text{File: "docs/a.md", Quote: "q"}
	errs := Ledger{Themes: []Theme{fixed("T-1", x), fixed("T-1", x)}}.Validate()
	require.Len(t, errs, 1)
	assert.EqualError(t, errs[0], "theme id T-1 used twice")
}

func TestNumbered(t *testing.T) {
	assert.True(t, numbered("OQ-42", "OQ-"))
	assert.True(t, numbered("M7", "M"))
	assert.False(t, numbered("M", "M"))
	assert.False(t, numbered("M7a", "M"))
	assert.False(t, numbered("R7", "M"))
}

func TestNormalize(t *testing.T) {
	assert.Equal(t, "a b c", normalize("  a\n  b\t\tc \n"))
}

func TestMissingIgnoresWrappingAndOpenThemes(t *testing.T) {
	fsys := fstest.MapFS{"docs/a.md": {Data: []byte("text before\nthe rule\n  stands, and more\n")}}
	l := Ledger{Themes: []Theme{
		fixed("T-1", Text{File: "docs/a.md", Quote: "the rule stands"},
			Text{File: "docs/a.md", Quote: "the rule is gone"}),
		{ID: "T-2", Status: Open, Texts: []Text{{File: "docs/none.md", Quote: "never read"}}},
	}}
	missing, err := l.Missing(fsys)
	require.NoError(t, err)
	assert.Equal(t, []string{"T-1: docs/a.md: the rule is gone"}, missing)
}

func TestMissingReportsAFileItCannotRead(t *testing.T) {
	l := Ledger{Themes: []Theme{fixed("T-1", Text{File: "docs/none.md", Quote: "q"})}}
	_, err := l.Missing(fstest.MapFS{})
	require.ErrorIs(t, err, fs.ErrNotExist)
	assert.ErrorContains(t, err, "T-1 cites docs/none.md")
}

func TestLoadReadsTheLedgerAndReportsAMissingFile(t *testing.T) {
	fsys := fstest.MapFS{"ledger.json": {Data: []byte(sample)}}
	l, err := Load(fsys, "ledger.json")
	require.NoError(t, err)
	assert.Len(t, l.Themes, 2)

	_, err = Load(fsys, "none.json")
	require.ErrorIs(t, err, fs.ErrNotExist)
}

func TestCheck(t *testing.T) {
	fsys := fstest.MapFS{"docs/a.md": {Data: []byte("the rule stands")}}
	l, err := Parse([]byte(sample))
	require.NoError(t, err)
	require.NoError(t, Check(l, fsys))

	gone := fstest.MapFS{"docs/a.md": {Data: []byte("another rule")}}
	err = Check(l, gone)
	require.ErrorIs(t, err, ErrNotCarried)
	assert.ErrorContains(t, err, "T-1: docs/a.md: the rule  stands")

	require.ErrorIs(t, Check(l, fstest.MapFS{}), fs.ErrNotExist)

	bad := Ledger{Themes: []Theme{{ID: "T-1", Status: Fixed}}}
	assert.ErrorContains(t, Check(bad, fsys), "ledger: malformed: T-1: fixed but cites no text")
}

// TestFindingLedgerIsCarried checks the repository's own ledger: every
// finding theme the domain-model rounds closed still has its closing
// sentences in the model and the SRS.
func TestFindingLedgerIsCarried(t *testing.T) {
	fsys := os.DirFS(root)
	l, err := Load(fsys, ledgerPath)
	require.NoError(t, err)
	require.NoError(t, Check(l, fsys))
}
