// Package scenario keeps the executable Gherkin scenarios under
// features/ in bijection with the requirement ids of the SRS: every
// requirement has exactly one tagged scenario, every scenario names a
// real requirement, and a scenario's priority and invariant tags agree
// with its requirement's row. It is build-time tooling and never links
// into the shipped binary.
package scenario

import (
	"fmt"
	"regexp"
	"slices"

	"github.com/cucumber/godog"
	messages "github.com/cucumber/messages/go/v34"
)

var (
	// idTag is the shape of a scenario's requirement tag, the same id
	// the SRS row carries: @REC-01, @SEC-18, @ASM-04.
	idTag = regexp.MustCompile(`^@((?:REC|PRV|PIN|RCL|LMK|INJ|CMP|ADM|MEM|OPS|LANE|VIEW|OWN|PEER|SEC|NFR|ENG|ASM)-[0-9]{2})$`)
	// priorityTag mirrors the row's Pri column.
	priorityTag = regexp.MustCompile(`^@(P[0-2])$`)
	// invariantTag mirrors one entry of the row's Traces column.
	invariantTag = regexp.MustCompile(`^@(I(?:[1-9]|10))$`)
)

// PendingTag marks a scenario that is declared but not yet written:
// the runner skips it rather than counting a pass that proves nothing.
const PendingTag = "@pending"

// Scenario is one scenario godog would run, as both the gate and the
// runner see it.
type Scenario struct {
	Path       string
	Line       int64
	Name       string
	ID         string
	Priority   string
	Invariants []string
	Pending    bool
	// Steps are the step texts in order, keywords left out.
	Steps []string
}

// Scenarios lists every scenario under dir in the order godog walks
// them — the same recursive walk and the same Gherkin parser — so the
// gate and the runner never disagree about which scenarios exist. A
// Scenario Outline is one scenario however many Examples rows it has:
// godog compiles a pickle per row, and the rows fold back onto their
// outline. A scenario must carry exactly one requirement tag and at
// most one priority tag; anything else is reported with its place.
func Scenarios(dir string) ([]Scenario, error) {
	suite := godog.TestSuite{Options: &godog.Options{Paths: []string{dir}}}
	features, err := suite.RetrieveFeatures()
	if err != nil {
		return nil, fmt.Errorf("scenario: read features: %w", err)
	}

	var out []Scenario
	for _, f := range features {
		lines := scenarioLines(f.GherkinDocument)
		seen := map[string]bool{}
		for _, p := range f.Pickles {
			node := p.AstNodeIds[0]
			if seen[node] {
				continue
			}
			seen[node] = true
			sc, err := scenarioOf(f.Uri, lines[node], p)
			if err != nil {
				return nil, err
			}
			out = append(out, sc)
		}
	}

	return out, nil
}

// scenarioOf reads one pickle's tags: its requirement id, priority,
// invariants, and whether it is pending.
func scenarioOf(uri string, line int64, p *messages.Pickle) (Scenario, error) {
	sc := Scenario{Path: uri, Line: line, Name: p.Name}
	for _, step := range p.Steps {
		sc.Steps = append(sc.Steps, step.Text)
	}
	var ids, prios []string
	for _, tag := range p.Tags {
		switch {
		case tag.Name == PendingTag:
			sc.Pending = true
		case idTag.MatchString(tag.Name):
			ids = append(ids, idTag.FindStringSubmatch(tag.Name)[1])
		case priorityTag.MatchString(tag.Name):
			prios = append(prios, priorityTag.FindStringSubmatch(tag.Name)[1])
		case invariantTag.MatchString(tag.Name):
			sc.Invariants = append(sc.Invariants, invariantTag.FindStringSubmatch(tag.Name)[1])
		}
	}
	if len(ids) != 1 {
		return Scenario{}, fmt.Errorf("scenario: %s:%d: %q carries %d requirement tags, want exactly one",
			uri, line, p.Name, len(ids))
	}
	if len(prios) > 1 {
		return Scenario{}, fmt.Errorf("scenario: %s:%d: %q carries %d priority tags, want at most one",
			uri, line, p.Name, len(prios))
	}
	sc.ID = ids[0]
	if len(prios) == 1 {
		sc.Priority = prios[0]
	}
	slices.Sort(sc.Invariants)
	sc.Invariants = slices.Compact(sc.Invariants)

	return sc, nil
}

// scenarioLines maps each scenario's AST id to the line it starts on,
// under the feature and under any rule, so an error can point at the
// scenario rather than the file.
func scenarioLines(doc *messages.GherkinDocument) map[string]int64 {
	lines := map[string]int64{}
	if doc.Feature == nil {
		return lines
	}
	for _, child := range doc.Feature.Children {
		if child.Scenario != nil {
			lines[child.Scenario.Id] = child.Scenario.Location.Line
		}
		if child.Rule == nil {
			continue
		}
		for _, nested := range child.Rule.Children {
			if nested.Scenario != nil {
				lines[nested.Scenario.Id] = nested.Scenario.Location.Line
			}
		}
	}

	return lines
}
