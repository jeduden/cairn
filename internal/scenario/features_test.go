package scenario

import (
	"os"
	"path/filepath"
	"testing"

	messages "github.com/cucumber/messages/go/v34"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// writeFeature lays one .feature file into a fresh directory.
func writeFeature(t *testing.T, body string) string {
	t.Helper()
	dir := t.TempDir()
	require.NoError(t, os.WriteFile(filepath.Join(dir, "x.feature"), []byte(body), 0o600))

	return dir
}

func TestScenariosReadsTagsAndPending(t *testing.T) {
	dir := writeFeature(t, `Feature: Record

  @REC-01 @P0 @I10 @I1 @I1 @pending
  Scenario: ingest transcripts
    Given nothing

  Rule: sources

    @NFR-01
    Scenario: fast hooks
      Given nothing
      Then a hook returns
`)

	got, err := Scenarios(dir)

	require.NoError(t, err)
	require.Len(t, got, 2)
	assert.Equal(t, Scenario{
		Path: filepath.Join(dir, "x.feature"), Line: 4, Name: "ingest transcripts",
		ID: "REC-01", Priority: "P0", Invariants: []string{"I1", "I10"}, Pending: true,
		Steps: []string{"nothing"},
	}, got[0])
	assert.Equal(t, "NFR-01", got[1].ID)
	assert.Empty(t, got[1].Priority)
	assert.Nil(t, got[1].Invariants)
	assert.EqualValues(t, 10, got[1].Line)
	assert.False(t, got[1].Pending)
	assert.Equal(t, []string{"nothing", "a hook returns"}, got[1].Steps)
}

func TestScenariosFoldsOutlineRowsOntoOneScenario(t *testing.T) {
	dir := writeFeature(t, `Feature: Outline

  @SEC-04 @P0 @I9
  Scenario Outline: literal terms
    Given <q>

    Examples:
      | q |
      | a |
      | b |
`)

	got, err := Scenarios(dir)

	require.NoError(t, err)
	require.Len(t, got, 1)
	assert.Equal(t, "SEC-04", got[0].ID)
}

func TestScenariosInheritsFeatureTags(t *testing.T) {
	dir := writeFeature(t, `@pending
Feature: Inherited

  @ASM-01
  Scenario: hook payloads
    Given nothing
`)

	got, err := Scenarios(dir)

	require.NoError(t, err)
	require.Len(t, got, 1)
	assert.True(t, got[0].Pending)
}

func TestScenariosRejectsMissingOrDoubledTags(t *testing.T) {
	cases := map[string]string{
		"no requirement tag":   "Feature: F\n\n  @P0\n  Scenario: s\n    Given x\n",
		"two requirement tags": "Feature: F\n\n  @REC-01 @REC-02\n  Scenario: s\n    Given x\n",
		"malformed id":         "Feature: F\n\n  @REC-1\n  Scenario: s\n    Given x\n",
		"two priority tags":    "Feature: F\n\n  @REC-01 @P0 @P1\n  Scenario: s\n    Given x\n",
	}
	for name, body := range cases {
		_, err := Scenarios(writeFeature(t, body))
		assert.Error(t, err, name)
	}
}

func TestScenariosReportsUnreadableFeatures(t *testing.T) {
	_, err := Scenarios(filepath.Join(t.TempDir(), "missing"))
	assert.ErrorContains(t, err, "scenario: read features")
}

func TestScenarioLinesOfADocumentWithNoFeature(t *testing.T) {
	assert.Empty(t, scenarioLines(&messages.GherkinDocument{}))
}

func TestScenariosAcceptsTheLaneFamilies(t *testing.T) {
	dir := writeFeature(t, "Feature: F\n\n"+
		"  @LANE-01 @P0 @I1\n  Scenario: a\n    Given x\n\n"+
		"  @VIEW-19 @P2\n  Scenario: b\n    Given x\n\n"+
		"  @OWN-22 @P1\n  Scenario: c\n    Given x\n\n"+
		"  @PEER-11 @P2\n  Scenario: d\n    Given x\n")

	got, err := Scenarios(dir)

	require.NoError(t, err)
	require.Len(t, got, 4)
	assert.Equal(t, []string{"LANE-01", "VIEW-19", "OWN-22", "PEER-11"},
		[]string{got[0].ID, got[1].ID, got[2].ID, got[3].ID})
}
