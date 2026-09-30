package review

import (
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

const (
	reviewed = "206b0a76a81abb510e53d87ef187e83b5aacbcbb"
	moved    = "490efa734617788d1dc1f831b26d0e23888e90fa"
)

func TestParseVerdictReadsTheAgentsOutput(t *testing.T) {
	v, err := ParseVerdict([]byte(`{"verdict":"request_changes","summary":"ENG-28 has no drift case.",
		"findings":[{"path":"internal/drift/cases.go","line":12,"severity":"blocking","body":"add one"}]}`))
	require.NoError(t, err)
	assert.Equal(t, Verdict{
		Verdict: RequestChangesVerdict,
		Summary: "ENG-28 has no drift case.",
		Findings: []Finding{
			{Path: "internal/drift/cases.go", Line: 12, Severity: Blocking, Body: "add one"},
		},
	}, v)
}

func TestParseVerdictRefusesWhatTheSchemaDoesNot(t *testing.T) {
	cases := map[string]string{
		"not JSON":          `approve`,
		"an unknown field":  `{"verdict":"approve","summary":"s","findings":[],"approve":true}`,
		"trailing data":     `{"verdict":"approve","summary":"s","findings":[]} {}`,
		"an unknown answer": `{"verdict":"lgtm","summary":"s","findings":[]}`,
		"no summary":        `{"verdict":"approve","summary":" ","findings":[]}`,
		"an unknown severity": `{"verdict":"approve","summary":"s",
			"findings":[{"path":"a.go","line":1,"severity":"minor","body":"b"}]}`,
		"a finding with no path": `{"verdict":"approve","summary":"s",
			"findings":[{"path":"","line":1,"severity":"nit","body":"b"}]}`,
		"a finding with no body": `{"verdict":"approve","summary":"s",
			"findings":[{"path":"a.go","line":1,"severity":"nit","body":""}]}`,
		"a negative line": `{"verdict":"approve","summary":"s",
			"findings":[{"path":"a.go","line":-1,"severity":"nit","body":"b"}]}`,
	}
	for name, body := range cases {
		t.Run(name, func(t *testing.T) {
			_, err := ParseVerdict([]byte(body))
			require.ErrorIs(t, err, ErrVerdict)
		})
	}
}

func TestParseChecksReadsOneCheckPerLine(t *testing.T) {
	checks, err := ParseChecks([]byte(`{"name":"CI","status":"completed","conclusion":"success"}

{"name":"review","status":"in_progress","conclusion":null}
`))
	require.NoError(t, err)
	assert.Equal(t, []Check{
		{Name: "CI", Status: "completed", Conclusion: "success"},
		{Name: "review", Status: "in_progress"},
	}, checks)
}

func TestParseChecksRefusesALineThatIsNoCheck(t *testing.T) {
	_, err := ParseChecks([]byte("{\"name\":\"CI\"}\nnot json\n"))
	require.ErrorContains(t, err, "check line 2")
}

func TestDecideSkipsAHeadThatMovedPastTheReview(t *testing.T) {
	_, post, err := Decide(Input{Verdict: approve(), Reviewed: reviewed, Head: moved, Checks: green()})
	require.NoError(t, err)
	assert.False(t, post)
}

func TestDecideApprovesOnlyAnApprovingVerdictOnAGreenHead(t *testing.T) {
	r, post, err := Decide(Input{Verdict: approve(), Reviewed: reviewed, Head: reviewed, Checks: green()})
	require.NoError(t, err)
	require.True(t, post)
	assert.Equal(t, Approve, r.Event)
	assert.Equal(t, reviewed, r.CommitID)
	assert.Contains(t, r.Body, "Looks right.")
}

func TestDecideRefusesAHeadWhoseCIDidNotPass(t *testing.T) {
	cases := map[string][]Check{
		"no CI check": {{Name: "lint", Status: "completed", Conclusion: "success"}},
		"CI failed":   {{Name: "CI", Status: "completed", Conclusion: "failure"}},
		"CI running":  {{Name: "CI", Status: "in_progress"}},
	}
	for name, checks := range cases {
		t.Run(name, func(t *testing.T) {
			_, _, err := Decide(Input{Verdict: approve(), Reviewed: reviewed, Head: reviewed, Checks: checks})
			require.ErrorIs(t, err, ErrCI)
		})
	}
}

func TestDecideRequestsChangesOtherwise(t *testing.T) {
	failed := append(green(), Check{Name: "CodeQL", Status: "completed", Conclusion: "failure"})
	blocking := approve()
	blocking.Findings = []Finding{{Path: "a.go", Line: 3, Severity: Blocking, Body: "breaks I4"}}
	cases := map[string]struct {
		in   Input
		want []string
	}{
		"the agent requests changes": {
			Input{Verdict: Verdict{Verdict: RequestChangesVerdict, Summary: "No scenario."},
				Reviewed: reviewed, Head: reviewed, Checks: green()},
			[]string{"No scenario."},
		},
		"another check failed": {
			Input{Verdict: approve(), Reviewed: reviewed, Head: reviewed, Checks: failed},
			[]string{"`CodeQL`: failure"},
		},
		"an approval with a blocking finding": {
			Input{Verdict: blocking, Reviewed: reviewed, Head: reviewed, Checks: green()},
			[]string{"`a.go:3`: breaks I4"},
		},
	}
	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			r, post, err := Decide(c.in)
			require.NoError(t, err)
			require.True(t, post)
			assert.Equal(t, RequestChanges, r.Event)
			for _, w := range c.want {
				assert.Contains(t, r.Body, w)
			}
		})
	}
}

func TestDecideIgnoresChecksStillRunningAndPassingOnes(t *testing.T) {
	checks := append(green(),
		Check{Name: "review", Status: "in_progress"},
		Check{Name: "codecov/patch", Status: "completed", Conclusion: "neutral"},
		Check{Name: "release", Status: "completed", Conclusion: "skipped"})
	r, post, err := Decide(Input{Verdict: approve(), Reviewed: reviewed, Head: reviewed, Checks: checks})
	require.NoError(t, err)
	require.True(t, post)
	assert.Equal(t, Approve, r.Event)
}

func TestBodyListsBlockingFindingsBeforeNits(t *testing.T) {
	v := Verdict{Verdict: RequestChangesVerdict, Summary: "Two things.", Findings: []Finding{
		{Path: "b.go", Line: 0, Severity: Nit, Body: "rename"},
		{Path: "a.go", Line: 7, Severity: Blocking, Body: "unwrapped error"},
	}}
	body := render(v, nil, RequestChanges, reviewed)
	blocking, nit := strings.Index(body, "unwrapped error"), strings.Index(body, "rename")
	require.GreaterOrEqual(t, blocking, 0)
	assert.Less(t, blocking, nit)
	assert.Contains(t, body, "`b.go`: rename")
	assert.Contains(t, body, reviewed[:12])
}

func approve() Verdict {
	return Verdict{Verdict: ApproveVerdict, Summary: "Looks right."}
}

func green() []Check {
	return []Check{
		{Name: "CI", Status: "completed", Conclusion: "success"},
		{Name: "test", Status: "completed", Conclusion: "success"},
	}
}

func TestLocationNamesTheLineOnlyWhenThereIsOne(t *testing.T) {
	assert.Equal(t, "a.go", location(Finding{Path: "a.go"}))
	assert.Equal(t, "a.go:4", location(Finding{Path: "a.go", Line: 4}))
}

func TestFailedChecksCountsOnlyCompletedFailures(t *testing.T) {
	checks := []Check{
		{Name: "a", Status: "completed", Conclusion: "success"},
		{Name: "b", Status: "completed", Conclusion: "cancelled"},
		{Name: "c", Status: "queued"},
	}
	assert.Equal(t, []Check{checks[1]}, failedChecks(checks))
}

func TestPassedCIWantsTheRequiredCheckCompletedGreen(t *testing.T) {
	assert.True(t, passedCI(Check{Name: "CI", Status: "completed", Conclusion: "success"}))
	assert.False(t, passedCI(Check{Name: "ci", Status: "completed", Conclusion: "success"}))
}

func TestIsBlockingIsTheBlockingSeverity(t *testing.T) {
	assert.True(t, isBlocking(Finding{Severity: Blocking}))
	assert.False(t, isBlocking(Finding{Severity: Nit}))
}
