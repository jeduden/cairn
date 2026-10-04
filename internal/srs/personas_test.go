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
	cases := map[string]string{
		"a persona outside U1–U9": "| Requirements | Personas |\n|---|---|\n| REC-01 | U10 |\n",
		"no persona":              "| Requirements | Personas |\n|---|---|\n| REC-01 | |\n",
		"an id listed twice":      "| Requirements | Personas |\n|---|---|\n| REC-01 | U1 |\n| REC-01 | U2 |\n",
		"a malformed id":          "| Requirements | Personas |\n|---|---|\n| rec-1 | U1 |\n",
		"a short row":             "| Requirements | Personas |\n|---|---|\n| REC-01 |\n",
	}
	for name, body := range cases {
		_, err := PersonaCoverage([]byte(body))
		assert.Error(t, err, name)
	}
	_, err := PersonaCoverage([]byte("# no table\n"))
	assert.ErrorContains(t, err, "no persona coverage table")
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
	assert.Equal(t, onDisk, slices.Sorted(maps.Values(named)), "§2.5 personas and .claude/agents/persona-*.md")
}
