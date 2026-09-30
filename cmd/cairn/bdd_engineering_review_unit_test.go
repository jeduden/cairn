package main

import (
	"testing"

	"github.com/cucumber/godog"
	messages "github.com/cucumber/messages/go/v34"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

const key = "APP_KEY"

// reviewFlow is the smallest workflow the ENG-28 steps accept.
const reviewFlow = `name: Review
on: # a comment
  workflow_run:
    workflows: [CI]
    types: [completed]

jobs:
  review:
    permissions:
      contents: read # never write
    steps:
      - uses: anthropics/claude-code-action@abc
  # a comment between jobs
  post:
    needs: [review]
    steps:
      - env:
          K: ${{ secrets.APP_KEY }}
        run: go run ./cmd/review-gate
`

func TestParseWorkflowSplitsTriggersAndJobs(t *testing.T) {
	w := parseWorkflow("stray: before\n" + reviewFlow)
	assert.Equal(t, []string{"workflow_run"}, w.triggers)
	require.Len(t, w.jobs, 2)
	assert.Equal(t, "review", w.jobs[0].name)
	assert.NotContains(t, w.jobs[0].text, "never write", "comments are dropped")
	assert.NotContains(t, w.jobs[0].text, "between jobs")
	assert.Contains(t, w.jobs[1].text, "go run ./cmd/review-gate")
}

func TestUncomment(t *testing.T) {
	assert.Empty(t, uncomment("  # whole line\n"))
	assert.Equal(t, "on:\n", uncomment("on: # trailing\n"))
	assert.Equal(t, "a#b\n", uncomment("a#b\n"))
}

func TestReadWorkflow(t *testing.T) {
	e := checkout(t, map[string]string{"review.yml": reviewFlow})
	require.NoError(t, e.readWorkflow("review.yml"))
	assert.Len(t, e.flow.jobs, 2)
	assert.Error(t, e.readWorkflow("missing.yml"))
}

func TestRunsAfter(t *testing.T) {
	w := parseWorkflow(reviewFlow)
	require.NoError(t, w.runsAfter("CI"))
	assert.ErrorContains(t, w.runsAfter("Build"), `does not say "workflows: [Build]"`)

	w.triggers = append(w.triggers, "workflow_dispatch")
	assert.ErrorContains(t, w.runsAfter("CI"), "want only workflow_run")
}

func TestAgentsUnprivileged(t *testing.T) {
	w := parseWorkflow(reviewFlow)
	require.NoError(t, w.agentsUnprivileged(key))
	assert.Equal(t, key, w.key)

	w.jobs[0].text += "      pull-requests: write\n        K: ${{ secrets.APP_KEY }}\n"
	err := w.agentsUnprivileged(key)
	assert.ErrorContains(t, err, "agent job review is granted a write permission")
	assert.ErrorContains(t, err, "agent job review reads APP_KEY")

	none := parseWorkflow("on:\n  workflow_run:\njobs:\n  a:\n    steps: []\n")
	assert.EqualError(t, none.agentsUnprivileged(key), "no job runs the reviewing agent")
}

func TestKeyHeldApart(t *testing.T) {
	w := parseWorkflow(reviewFlow)
	require.NoError(t, w.keyHeldApart(key))

	w.jobs[1].text += "      - uses: anthropics/claude-code-action@abc\n"
	assert.ErrorContains(t, w.keyHeldApart(key), "job post holds APP_KEY and runs the agent")

	assert.ErrorContains(t, w.keyHeldApart("OTHER"), `0 jobs read "OTHER", want exactly one`)
}

func TestKeyJobAfterAgents(t *testing.T) {
	w := parseWorkflow(reviewFlow)
	w.key = key
	require.NoError(t, w.keyJobAfterAgents())

	w.jobs[1].text = "    needs: [reviewer]\n    if: always()\n    K: ${{ secrets.APP_KEY }}\n"
	err := w.keyJobAfterAgents()
	assert.ErrorContains(t, err, "job post does not need agent job review")
	assert.ErrorContains(t, err, "job post runs on always(), even after a failed review")

	w.key = "OTHER"
	assert.Error(t, w.keyJobAfterAgents())
}

func TestKeyJobRuns(t *testing.T) {
	w := parseWorkflow(reviewFlow)
	w.key = key
	require.NoError(t, w.keyJobRuns("go run ./cmd/review-gate"))
	assert.ErrorContains(t, w.keyJobRuns("go run ./other"), `job post does not run "go run ./other"`)

	w.key = "OTHER"
	assert.Error(t, w.keyJobRuns("go run ./cmd/review-gate"))
}

func table(rows ...[]string) *godog.Table {
	t := &godog.Table{}
	for _, r := range rows {
		row := &messages.PickleTableRow{}
		for _, v := range r {
			row.Cells = append(row.Cells, &messages.PickleTableCell{Value: v})
		}
		t.Rows = append(t.Rows, row)
	}

	return t
}

func gateHeader() []string {
	return []string{"verdict", "finding", "head", "CI", "other check", "review"}
}

func TestGateDecides(t *testing.T) {
	right := table(gateHeader(), []string{"approve", "none", "current", "passed", "passed", "APPROVE"})
	require.NoError(t, gateDecides(right))

	wrong := table(gateHeader(), []string{"approve", "none", "current", "passed", "passed", "REQUEST_CHANGES"})
	assert.ErrorContains(t, gateDecides(wrong), "row 1")

	assert.EqualError(t, gateDecides(table(gateHeader())), "the table has no rows under its header")
}

func TestTableRowsKeysCellsByHeader(t *testing.T) {
	rows, err := tableRows(table([]string{"a", "b"}, []string{"1", "2"}))
	require.NoError(t, err)
	assert.Equal(t, []map[string]string{{"a": "1", "b": "2"}}, rows)
}

func TestGateOutcome(t *testing.T) {
	row := func(verdict, finding, head, ci, other string) map[string]string {
		return map[string]string{"verdict": verdict, "finding": finding, "head": head, "CI": ci, "other check": other}
	}
	assert.Equal(t, "APPROVE", gateOutcome(row("approve", "nit", "current", "passed", "passed")))
	assert.Equal(t, "REQUEST_CHANGES", gateOutcome(row("approve", "blocking", "current", "passed", "passed")))
	assert.Equal(t, "nothing", gateOutcome(row("approve", "none", "moved", "passed", "passed")))
	assert.Equal(t, "an error", gateOutcome(row("approve", "none", "current", "failed", "passed")))
	assert.Equal(t, "an error", gateOutcome(row("malformed", "none", "current", "passed", "passed")))
}
