package scenario

import (
	"fmt"
	"slices"

	"github.com/jeduden/cairn/internal/srs"
)

// Check compares the requirements against the scenarios and returns
// every disagreement, sorted, one line each. It is empty exactly when:
//
//   - every requirement has one scenario tagged with its id, and every
//     scenario's id names a requirement;
//   - a scenario's priority tag equals its requirement's Pri column,
//     and is absent when the row has none (NFR, ASM);
//   - a scenario's invariant tags equal its requirement's Traces.
//
// A requirement edited in the SRS without its scenario, or a scenario
// retagged without the SRS, fails the build here rather than drifting.
func Check(reqs []srs.Requirement, scenarios []Scenario) []string {
	byID := make(map[string]Scenario, len(scenarios))
	var problems []string
	for _, sc := range scenarios {
		if prev, ok := byID[sc.ID]; ok {
			problems = append(problems, fmt.Sprintf("%s: tagged on %s:%d and %s:%d",
				sc.ID, prev.Path, prev.Line, sc.Path, sc.Line))

			continue
		}
		byID[sc.ID] = sc
	}

	known := make(map[string]bool, len(reqs))
	for _, r := range reqs {
		known[r.ID] = true
		sc, ok := byID[r.ID]
		if !ok {
			problems = append(problems, fmt.Sprintf("%s: no scenario under features/", r.ID))

			continue
		}
		if sc.Priority != r.Priority {
			problems = append(problems, fmt.Sprintf("%s: scenario priority %q, requirement says %q",
				r.ID, sc.Priority, r.Priority))
		}
		want := slices.Sorted(slices.Values(r.Traces))
		if !slices.Equal(sc.Invariants, want) {
			problems = append(problems, fmt.Sprintf("%s: scenario invariants %v, requirement traces %v",
				r.ID, sc.Invariants, want))
		}
	}
	for id, sc := range byID {
		if !known[id] {
			problems = append(problems, fmt.Sprintf("%s: tagged on %s:%d but no SRS requirement has that id",
				id, sc.Path, sc.Line))
		}
	}
	slices.Sort(problems)

	return problems
}
