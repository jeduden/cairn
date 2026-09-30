package main

import (
	"errors"
	"fmt"
	"regexp"
	"slices"
	"strings"

	"github.com/cucumber/godog"
	"github.com/jeduden/cairn/internal/review"
)

// The ENG-28 steps: the review workflow keeps the reviewing agent away
// from the reviewer app's key, and the gate that posts the review
// decides as the requirement says.

// agentAction is the action that runs the reviewing agent.
const agentAction = "anthropics/claude-code-action@"

func init() {
	registrars = append(registrars, bindReview)
}

func bindReview(w *world, sc *godog.ScenarioContext) {
	e := func() *engineering { return section[engineering](w) }

	sc.Step(`^the review workflow is read from "([^"]+)"$`, func(file string) error {
		return e().readWorkflow(file)
	})
	sc.Step(`^it runs only when the "([^"]+)" workflow completes, as the default branch defines it$`,
		func(name string) error { return e().flow.runsAfter(name) })
	sc.Step(`^the job that runs the reviewing agent holds no write permission and not the "([^"]+)" key$`,
		func(key string) error { return e().flow.agentsUnprivileged(key) })
	sc.Step(`^only one job holds the "([^"]+)" key, and it runs no agent$`, func(key string) error {
		return e().flow.keyHeldApart(key)
	})
	sc.Step(`^that job runs only after the agent's job succeeded$`, func() error {
		return e().flow.keyJobAfterAgents()
	})
	sc.Step(`^that job decides the review with "([^"]+)"$`, func(cmd string) error {
		return e().flow.keyJobRuns(cmd)
	})
	sc.Step(`^the review gate decides:$`, func(t *godog.Table) error {
		return gateDecides(t)
	})
}

// workflow is a GitHub Actions workflow split the way the ENG-28 steps
// read it: its triggers and its jobs, comments left out.
type workflow struct {
	triggers []string
	on       string
	jobs     []job
	// key is the secret the steps last named as the reviewer app's key.
	key string
}

// job is one job's name and its lines.
type job struct {
	name string
	text string
}

var (
	topKey = regexp.MustCompile(`^([A-Za-z_-]+):`)
	subKey = regexp.MustCompile(`^  ([A-Za-z0-9_-]+):`)
	write  = regexp.MustCompile(`:\s*write\b`)
)

// readWorkflow parses file from the checkout.
func (e *engineering) readWorkflow(file string) error {
	body, err := e.read(file)
	if err != nil {
		return err
	}
	e.flow = parseWorkflow(string(body))

	return nil
}

// parseWorkflow splits body into its `on:` block and its jobs. It reads
// only the block layout the repository's workflows use: two-space
// indents, and triggers and jobs as keys one level down.
func parseWorkflow(body string) workflow {
	var (
		w   workflow
		top string
	)
	for line := range strings.Lines(body) {
		line = uncomment(line)
		if strings.TrimSpace(line) == "" {
			continue
		}
		if m := topKey.FindStringSubmatch(line); m != nil {
			top = m[1]

			continue
		}
		m := subKey.FindStringSubmatch(line)
		switch {
		case top == "on" && m != nil:
			w.triggers = append(w.triggers, m[1])
			w.on += line
		case top == "on":
			w.on += line
		case top == "jobs" && m != nil:
			w.jobs = append(w.jobs, job{name: m[1]})
		case top == "jobs" && len(w.jobs) > 0:
			w.jobs[len(w.jobs)-1].text += line
		}
	}

	return w
}

// uncomment drops a YAML comment: a whole comment line, or one that
// follows a space.
func uncomment(line string) string {
	if strings.HasPrefix(strings.TrimSpace(line), "#") {
		return ""
	}
	if i := strings.Index(line, " #"); i >= 0 {
		return line[:i] + "\n"
	}

	return line
}

// runsAfter requires workflow_run as the only trigger, on the named
// workflow's completion. GitHub runs a workflow_run workflow as the
// default branch defines it, so a pull request cannot rewrite its own
// reviewer; any other trigger would let a branch's copy run.
func (w workflow) runsAfter(name string) error {
	if !slices.Equal(w.triggers, []string{"workflow_run"}) {
		return fmt.Errorf("the review workflow triggers on %v, want only workflow_run", w.triggers)
	}
	for _, want := range []string{"workflows: [" + name + "]", "types: [completed]"} {
		if !strings.Contains(w.on, want) {
			return fmt.Errorf("the workflow_run trigger does not say %q", want)
		}
	}

	return nil
}

// agentJobs lists the jobs that run the reviewing agent.
func (w workflow) agentJobs() []job {
	var out []job
	for _, j := range w.jobs {
		if strings.Contains(j.text, agentAction) {
			out = append(out, j)
		}
	}

	return out
}

// agentsUnprivileged requires an agent job, and that none of them is
// granted a write permission or reads key.
func (w *workflow) agentsUnprivileged(key string) error {
	w.key = key
	agents := w.agentJobs()
	if len(agents) == 0 {
		return errors.New("no job runs the reviewing agent")
	}
	var errs []error
	for _, j := range agents {
		if write.MatchString(j.text) {
			errs = append(errs, fmt.Errorf("agent job %s is granted a write permission", j.name))
		}
		if strings.Contains(j.text, "secrets."+key) {
			errs = append(errs, fmt.Errorf("agent job %s reads %s", j.name, key))
		}
	}

	return errors.Join(errs...)
}

// keyJob finds the one job that reads the key.
func (w workflow) keyJob() (job, error) {
	var holders []job
	for _, j := range w.jobs {
		if strings.Contains(j.text, "secrets."+w.key) {
			holders = append(holders, j)
		}
	}
	if len(holders) != 1 {
		return job{}, fmt.Errorf("%d jobs read %q, want exactly one", len(holders), w.key)
	}

	return holders[0], nil
}

// keyHeldApart requires one job to hold key, and that job to run no
// agent.
func (w *workflow) keyHeldApart(key string) error {
	w.key = key
	j, err := w.keyJob()
	if err != nil {
		return err
	}
	if strings.Contains(j.text, agentAction) {
		return fmt.Errorf("job %s holds %s and runs the agent", j.name, key)
	}

	return nil
}

// keyJobAfterAgents requires the key's job to need every agent job, and
// not to run when one of them failed.
func (w workflow) keyJobAfterAgents() error {
	j, err := w.keyJob()
	if err != nil {
		return err
	}
	var needs string
	for line := range strings.Lines(j.text) {
		if strings.HasPrefix(strings.TrimSpace(line), "needs:") {
			needs = line
		}
	}
	var errs []error
	for _, a := range w.agentJobs() {
		if !regexp.MustCompile(`\b` + regexp.QuoteMeta(a.name) + `\b`).MatchString(needs) {
			errs = append(errs, fmt.Errorf("job %s does not need agent job %s", j.name, a.name))
		}
	}
	for _, override := range []string{"always()", "failure()", "cancelled()"} {
		if strings.Contains(j.text, override) {
			errs = append(errs, fmt.Errorf("job %s runs on %s, even after a failed review", j.name, override))
		}
	}

	return errors.Join(errs...)
}

// keyJobRuns requires the key's job to run cmd.
func (w workflow) keyJobRuns(cmd string) error {
	j, err := w.keyJob()
	if err != nil {
		return err
	}
	if !strings.Contains(j.text, cmd) {
		return fmt.Errorf("job %s does not run %q", j.name, cmd)
	}

	return nil
}

// gateDecides runs every row of t through the gate and compares the
// outcome with the row's review column.
func gateDecides(t *godog.Table) error {
	rows, err := tableRows(t)
	if err != nil {
		return err
	}
	var errs []error
	for i, r := range rows {
		if got := gateOutcome(r); got != r["review"] {
			errs = append(errs, fmt.Errorf("row %d %v: the gate gives %s", i+1, r, got))
		}
	}

	return errors.Join(errs...)
}

// tableRows keys each body row of t by the header row.
func tableRows(t *godog.Table) ([]map[string]string, error) {
	if len(t.Rows) < 2 {
		return nil, errors.New("the table has no rows under its header")
	}
	header := t.Rows[0].Cells
	var out []map[string]string
	for _, row := range t.Rows[1:] {
		m := map[string]string{}
		for i, c := range row.Cells {
			m[header[i].Value] = c.Value
		}
		out = append(out, m)
	}

	return out, nil
}

// gateOutcome builds the gate's input from one table row and names what
// the gate does with it.
func gateOutcome(r map[string]string) string {
	const reviewed, moved = "206b0a76a81abb510e53d87ef187e83b5aacbcbb", "490efa734617788d1dc1f831b26d0e23888e90fa"
	findings := "[]"
	if r["finding"] != "none" {
		findings = fmt.Sprintf(`[{"path":"a.go","line":1,"severity":%q,"body":"b"}]`, r["finding"])
	}
	body := fmt.Sprintf(`{"verdict":%q,"summary":"s","findings":%s}`, r["verdict"], findings)
	v, err := review.ParseVerdict([]byte(body))
	if err != nil {
		return "an error"
	}
	conclusion := map[string]string{"passed": "success", "failed": "failure"}
	in := review.Input{Verdict: v, Reviewed: reviewed, Head: reviewed, Checks: []review.Check{
		{Name: "CI", Status: "completed", Conclusion: conclusion[r["CI"]]},
		{Name: "lint", Status: "completed", Conclusion: conclusion[r["other check"]]},
	}}
	if r["head"] == "moved" {
		in.Head = moved
	}
	out, post, err := review.Decide(in)
	switch {
	case err != nil:
		return "an error"
	case !post:
		return "nothing"
	default:
		return out.Event
	}
}
