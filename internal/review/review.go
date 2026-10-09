// Package review is the gate between a reviewing agent and the
// approval it may earn (ENG-21, ENG-28). The agent only writes a
// structured verdict; this package, run by the workflow step that
// alone holds the reviewer app's key, decides what review to post. It
// approves only an approving verdict with no blocking finding, on the
// head the agent read, once CI passed there. It is build-time tooling
// and never links into the shipped binary.
package review

import (
	"bufio"
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"slices"
	"strings"
	"unicode"
)

// The answers a verdict can give.
const (
	ApproveVerdict        = "approve"
	RequestChangesVerdict = "request_changes"
)

// The severities a finding can carry. A blocking finding withholds
// approval whatever the verdict says.
const (
	Blocking = "blocking"
	Nit      = "nit"
)

// The review events the gate posts, as GitHub names them.
const (
	Approve        = "APPROVE"
	RequestChanges = "REQUEST_CHANGES"
)

// ciCheck is the check the branch ruleset requires: the job that
// passes only when every CI job passed.
const ciCheck = "CI"

// ErrVerdict marks agent output that does not fit the verdict schema.
var ErrVerdict = errors.New("review: malformed verdict")

// ErrCI marks a head whose required CI check has not passed.
var ErrCI = errors.New("review: CI has not passed on the reviewed head")

// Verdict is the reviewing agent's structured output.
type Verdict struct {
	Verdict  string    `json:"verdict"`
	Summary  string    `json:"summary"`
	Findings []Finding `json:"findings"`
}

// Finding is one problem the agent found, at a path and line of the
// reviewed tree; line 0 means the whole file.
type Finding struct {
	Path     string `json:"path"`
	Line     int    `json:"line"`
	Severity string `json:"severity"`
	Body     string `json:"body"`
}

// Check is one check run on the reviewed head. A commit can carry
// several runs of one check, when CI ran on it twice; StartedAt, an
// RFC 3339 UTC time, orders them, and ID, which GitHub assigns in
// creation order, breaks a tie within one second.
type Check struct {
	ID         int64  `json:"id"`
	Name       string `json:"name"`
	Status     string `json:"status"`
	Conclusion string `json:"conclusion"`
	StartedAt  string `json:"started_at"`
}

// Input is everything the gate decides from.
type Input struct {
	Verdict Verdict
	// Reviewed is the head commit the agent read.
	Reviewed string
	// Head is the pull request's head commit now.
	Head   string
	Checks []Check
}

// Review is the pull request review to post, in the shape GitHub's
// create-review endpoint takes.
type Review struct {
	CommitID string `json:"commit_id"`
	Event    string `json:"event"`
	Body     string `json:"body"`
}

// ParseVerdict decodes the agent's output strictly: one object, no
// unknown field, and every value one the schema names.
func ParseVerdict(data []byte) (Verdict, error) {
	dec := json.NewDecoder(bytes.NewReader(data))
	dec.DisallowUnknownFields()
	var v Verdict
	if err := dec.Decode(&v); err != nil {
		return Verdict{}, fmt.Errorf("%w: %w", ErrVerdict, err)
	}
	if _, err := dec.Token(); !errors.Is(err, io.EOF) {
		return Verdict{}, fmt.Errorf("%w: data after the verdict", ErrVerdict)
	}

	if err := validate(v); err != nil {
		return Verdict{}, err
	}

	return v, nil
}

// validate checks every value in v is one the schema names.
func validate(v Verdict) error {
	if v.Verdict != ApproveVerdict && v.Verdict != RequestChangesVerdict {
		return fmt.Errorf("%w: verdict %q is neither %s nor %s", ErrVerdict, v.Verdict,
			ApproveVerdict, RequestChangesVerdict)
	}
	if strings.TrimSpace(v.Summary) == "" {
		return fmt.Errorf("%w: no summary", ErrVerdict)
	}
	for i, f := range v.Findings {
		switch {
		case f.Severity != Blocking && f.Severity != Nit:
			return fmt.Errorf("%w: finding %d: severity %q", ErrVerdict, i, f.Severity)
		case f.Path == "" || strings.TrimSpace(f.Body) == "":
			return fmt.Errorf("%w: finding %d has no path or no body", ErrVerdict, i)
		case f.Line < 0:
			return fmt.Errorf("%w: finding %d: line %d", ErrVerdict, i, f.Line)
		}
	}

	return nil
}

// ParseChecks reads one check run per line, as `gh api --jq` prints
// them; blank lines are skipped.
func ParseChecks(data []byte) ([]Check, error) {
	var checks []Check
	sc := bufio.NewScanner(bytes.NewReader(data))
	for n := 1; sc.Scan(); n++ {
		line := bytes.TrimSpace(sc.Bytes())
		if len(line) == 0 {
			continue
		}
		var c Check
		if err := json.Unmarshal(line, &c); err != nil {
			return nil, fmt.Errorf("review: check line %d: %w", n, err)
		}
		checks = append(checks, c)
	}

	return checks, nil
}

// Decide returns the review to post, or post false when the pull
// request moved past the reviewed head: the run for the newer head
// decides instead. A head whose CI has not passed is an error, so the
// step fails loudly rather than posting anything.
func Decide(in Input) (r Review, post bool, err error) {
	if in.Head != in.Reviewed {
		return Review{}, false, nil
	}
	checks := latest(in.Checks)
	if !slices.ContainsFunc(checks, passedCI) {
		return Review{}, false, fmt.Errorf("%w: %s", ErrCI, in.Reviewed)
	}
	failed := failedChecks(checks)
	event := RequestChanges
	if in.Verdict.Verdict == ApproveVerdict && len(failed) == 0 &&
		!slices.ContainsFunc(in.Verdict.Findings, isBlocking) {
		event = Approve
	}

	return Review{CommitID: in.Reviewed, Event: event, Body: render(in.Verdict, failed, event, in.Reviewed)}, true, nil
}

// latest keeps the newest run of each check, in the order the checks
// first appear. An older run that a newer one replaced, cancelled or
// failed, says nothing about the head as it stands.
func latest(checks []Check) []Check {
	var out []Check
	at := map[string]int{}
	for _, c := range checks {
		i, seen := at[c.Name]
		switch {
		case !seen:
			at[c.Name] = len(out)
			out = append(out, c)
		case newer(c, out[i]):
			out[i] = c
		}
	}

	return out
}

// newer reports whether run a started after run b, by start time and
// then by run id.
func newer(a, b Check) bool {
	if a.StartedAt != b.StartedAt {
		return a.StartedAt > b.StartedAt
	}

	return a.ID > b.ID
}

// passedCI reports whether c is the required CI check, passed.
func passedCI(c Check) bool {
	return c.Name == ciCheck && c.Status == "completed" && c.Conclusion == "success"
}

// isBlocking reports whether f withholds approval.
func isBlocking(f Finding) bool {
	return f.Severity == Blocking
}

// failedChecks lists the completed checks that neither passed nor were
// skipped. A check still running, this review among them, is not
// counted: the required CI check has already passed.
func failedChecks(checks []Check) []Check {
	var out []Check
	for _, c := range checks {
		if c.Status == "completed" && !slices.Contains([]string{"success", "neutral", "skipped"}, c.Conclusion) {
			out = append(out, c)
		}
	}

	return out
}

// render writes the review body: the outcome and the head it covers,
// the agent's summary, its blocking findings, then its nits, then the
// checks that failed.
func render(v Verdict, failed []Check, event, head string) string {
	var b strings.Builder
	outcome := "Changes requested"
	if event == Approve {
		outcome = "Approved"
	}
	fmt.Fprintf(&b, "**%s** by the review agent on `%s`.\n\n%s\n",
		outcome, head[:min(len(head), 12)], defuseMentions(v.Summary))
	for _, sev := range []string{Blocking, Nit} {
		for _, f := range v.Findings {
			if f.Severity == sev {
				fmt.Fprintf(&b, "\n- %s `%s`: %s", sev, location(f), defuseMentions(f.Body))
			}
		}
	}
	for _, c := range failed {
		fmt.Fprintf(&b, "\n- check `%s`: %s", c.Name, c.Conclusion)
	}

	return b.String()
}

// location names a finding's file, and its line when it has one.
func location(f Finding) string {
	if f.Line == 0 {
		return f.Path
	}

	return fmt.Sprintf("%s:%d", f.Path, f.Line)
}

// defuseMentions wraps each bare @word outside a code span in
// backticks, so a review quoting a scenario tag such as @I2 mentions
// no GitHub user of that name. Text inside backticks and the @ of an
// email address stay as they are.
func defuseMentions(s string) string {
	var b strings.Builder
	rs := []rune(s)
	inCode := false
	for i := 0; i < len(rs); i++ {
		r := rs[i]
		if r == '`' {
			inCode = !inCode
			b.WriteRune(r)
			continue
		}
		if inCode || r != '@' || (i > 0 && wordRune(rs[i-1])) || i+1 == len(rs) || !wordRune(rs[i+1]) {
			b.WriteRune(r)
			continue
		}
		j := i + 1
		for j < len(rs) && (wordRune(rs[j]) || rs[j] == '-') {
			j++
		}
		b.WriteString("`" + string(rs[i:j]) + "`")
		i = j - 1
	}

	return b.String()
}

// wordRune reports whether r can be part of a GitHub user name.
func wordRune(r rune) bool {
	return unicode.IsLetter(r) || unicode.IsDigit(r) || r == '_'
}
