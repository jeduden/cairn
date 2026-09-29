package scenario

import (
	"testing"

	"github.com/jeduden/cairn/internal/srs"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// The real specification and scenario tree the gate holds together.
const (
	srsDir      = "../../docs/srs"
	featuresDir = "../../features"
)

func TestCheckIsEmptyWhenEverythingAgrees(t *testing.T) {
	reqs := []srs.Requirement{
		{ID: "REC-03", Priority: "P0", Traces: []string{"I10", "I1"}},
		{ID: "NFR-01"},
	}
	scenarios := []Scenario{
		{ID: "REC-03", Priority: "P0", Invariants: []string{"I1", "I10"}},
		{ID: "NFR-01"},
	}

	assert.Empty(t, Check(reqs, scenarios))
}

func TestCheckReportsEveryDisagreementSorted(t *testing.T) {
	reqs := []srs.Requirement{
		{ID: "REC-01", Priority: "P0", Traces: []string{"I1"}},
		{ID: "REC-02", Priority: "P0", Traces: []string{"I1"}},
		{ID: "REC-03", Priority: "P1"},
	}
	scenarios := []Scenario{
		{ID: "REC-01", Priority: "P1", Invariants: []string{"I1"}, Path: "a", Line: 1},
		{ID: "REC-01", Path: "b", Line: 2},
		{ID: "REC-03", Priority: "P1", Invariants: []string{"I2"}},
		{ID: "SEC-99", Path: "c", Line: 3},
	}

	assert.Equal(t, []string{
		`REC-01: scenario priority "P1", requirement says "P0"`,
		"REC-01: tagged on a:1 and b:2",
		"REC-02: no scenario under features/",
		"REC-03: scenario invariants [I2], requirement traces []",
		"SEC-99: tagged on c:3 but no SRS requirement has that id",
	}, Check(reqs, scenarios))
}

// TestSpecificationAndFeaturesAgree is the gate itself: every SRS
// requirement has its one scenario under features/, tagged with the
// row's priority and invariants, and no scenario names an id the SRS
// lacks. Adding a requirement means adding its scenario — tagged
// @pending until its steps exist — in the same change.
func TestSpecificationAndFeaturesAgree(t *testing.T) {
	reqs, err := srs.Load(srsDir)
	require.NoError(t, err)
	scenarios, err := Scenarios(featuresDir)
	require.NoError(t, err)

	assert.Empty(t, Check(reqs, scenarios))
}
