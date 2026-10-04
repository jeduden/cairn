package srs

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

// agentsDir holds the persona agents §2.5 names.
const agentsDir = "../../.claude/agents"

const appendixC = `# C

| Other | Table |
|---|---|
| x | y |

| Requirements | Personas |
|---|---|
| REC-01, REC-02 | U9, U4 |
| SEC-08 | U8 |
`

func TestPersonaCoverageReadsTheTable(t *testing.T) {
	got, err := PersonaCoverage([]byte(appendixC))

	require.NoError(t, err)
	assert.Equal(t, map[string][]string{
		"REC-01": {"U9", "U4"}, "REC-02": {"U9", "U4"}, "SEC-08": {"U8"},
	}, got)
}

func TestPersonaCoverageRejectsBadRows(t *testing.T) {
	const head = "| Requirements | Personas |\n|---|---|\n"
	cases := map[string]struct{ body, want string }{
		"a persona outside U1–U9": {head + "| REC-01 | U10 |\n", `line 3: malformed persona "U10"`},
		"no persona":              {head + "| REC-01 | |\n", `line 3: malformed persona ""`},
		"an id listed twice":      {head + "| REC-01 | U1 |\n| REC-01 | U2 |\n", "line 4: REC-01 listed twice"},
		"a malformed id":          {head + "| rec-1 | U1 |\n", `line 3: malformed requirement id "rec-1"`},
		"a short row":             {head + "| REC-01 |\n", "line 3: malformed persona coverage row"},
	}
	for name, c := range cases {
		_, err := PersonaCoverage([]byte(c.body))
		assert.ErrorContains(t, err, c.want, name)
	}
	_, err := PersonaCoverage([]byte("# no table\n"))
	assert.ErrorContains(t, err, "no persona coverage table")
}

func TestSplitPersonasReadsTheCell(t *testing.T) {
	got, err := splitPersonas(" U9 , U4,U1 ")

	require.NoError(t, err)
	assert.Equal(t, []string{"U9", "U4", "U1"}, got)

	_, err = splitPersonas("U1, X2")
	assert.ErrorContains(t, err, `malformed persona "X2"`)

	_, err = splitPersonas("U1, U4, U1")
	assert.ErrorContains(t, err, "persona U1 listed twice")
}

func TestPersonaAgentsReadsSection25(t *testing.T) {
	body := "| # | Persona | Agent | Who |\n|---|---|---|---|\n" +
		"| U1 | Platform operator | `persona-platform-operator` | x |\n" +
		"| U9 | Agent | `persona-agent` | y |\n"

	got, err := PersonaAgents([]byte(body))

	require.NoError(t, err)
	assert.Equal(t, map[string]string{"U1": "persona-platform-operator", "U9": "persona-agent"}, got)
}

func TestPersonaAgentsRejectsBadRows(t *testing.T) {
	_, err := PersonaAgents([]byte("| # | Persona | Agent |\n|---|---|---|\n| X1 | p | `a` |\n"))
	assert.ErrorContains(t, err, "malformed persona row")
	_, err = PersonaAgents([]byte("| # | Persona | Agent |\n|---|---|---|\n| U1 | p | a |\n"))
	assert.ErrorContains(t, err, "malformed persona row")
	_, err = PersonaAgents([]byte("# none\n"))
	assert.ErrorContains(t, err, "no persona table")
	_, err = PersonaAgents([]byte("| # | Persona | Agent |\n|---|---|---|\n" +
		"| U1 | p | `persona-a` |\n| U1 | p | `persona-a` |\n"))
	assert.ErrorContains(t, err, "U1 listed twice")
	_, err = PersonaAgents([]byte("| # | Persona | Agent |\n|---|---|---|\n" +
		"| U1 | p | `persona-a` |\n| U2 | q | `persona-a` |\n"))
	assert.ErrorContains(t, err, "line 4: persona-a named twice")
}

// TestAppendixCCoversEveryRequirement keeps Appendix C honest: every
// requirement of §5–§7 and §10 serves at least one persona, every id
// it lists is a real requirement, every persona it names is one §2.5
// defines, and every persona of §2.5 serves some requirement.
func TestAppendixCCoversEveryRequirement(t *testing.T) {
	reqs, err := Load(srsDir)
	require.NoError(t, err)
	body, err := os.ReadFile(filepath.Join(srsDir, "appendix-c-persona-coverage.md"))
	require.NoError(t, err)
	coverage, err := PersonaCoverage(body)
	require.NoError(t, err)
	context, err := os.ReadFile(filepath.Join(srsDir, "02-context.md"))
	require.NoError(t, err)
	defined, err := PersonaAgents(context)
	require.NoError(t, err)

	known := map[string]bool{}
	for _, r := range reqs {
		if strings.HasPrefix(r.ID, "ASM-") {
			continue
		}
		known[r.ID] = true
		assert.Contains(t, coverage, r.ID, "%s serves no persona in Appendix C", r.ID)
	}
	served := map[string]bool{}
	for _, id := range slices.Sorted(maps.Keys(coverage)) {
		assert.True(t, known[id], "Appendix C lists %s, which is no requirement", id)
		for _, p := range coverage[id] {
			assert.Contains(t, defined, p, "Appendix C names %s, which §2.5 does not define", p)
			served[p] = true
		}
	}
	for _, p := range slices.Sorted(maps.Keys(defined)) {
		assert.True(t, served[p], "persona %s serves no requirement", p)
	}
}

// TestPersonasMatchTheAgents keeps §2.5 and .claude/agents in step: every
// persona names an agent file that exists, and every persona agent file
// is named by a persona.
func TestPersonasMatchTheAgents(t *testing.T) {
	body, err := os.ReadFile(filepath.Join(srsDir, "02-context.md"))
	require.NoError(t, err)
	named, err := PersonaAgents(body)
	require.NoError(t, err)

	files, err := filepath.Glob(filepath.Join(agentsDir, "persona-*.md"))
	require.NoError(t, err)
	onDisk := make([]string, 0, len(files))
	for _, f := range files {
		onDisk = append(onDisk, strings.TrimSuffix(filepath.Base(f), ".md"))
	}
	slices.Sort(onDisk)
	assert.Equal(t, slices.Sorted(maps.Values(named)), onDisk, "§2.5 personas and .claude/agents/persona-*.md")
}

func TestPersonaRowReadsOneRow(t *testing.T) {
	persona, agent, err := personaRow(Row{Line: 3, Cells: []string{"U2", "p", "`persona-b`"}})

	require.NoError(t, err)
	assert.Equal(t, "U2", persona)
	assert.Equal(t, "persona-b", agent)

	_, _, err = personaRow(Row{Line: 4, Cells: []string{"U2", "p"}})
	assert.EqualError(t, err, "srs: line 4: malformed persona row")
	_, _, err = personaRow(Row{Line: 5, Cells: []string{"U2", "p", "persona-b"}})
	assert.EqualError(t, err, "srs: line 5: malformed persona row")
}
