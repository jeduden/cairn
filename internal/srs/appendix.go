package srs

import (
	"fmt"
	"regexp"
	"slices"
	"strings"
)

// invariantCell is the leading "**I<n>**" of an Appendix B row.
var invariantCell = regexp.MustCompile(`^\*\*(I(?:[1-9]|10))\*\*`)

// InvariantCoverage reads Appendix B's coverage table in body — the
// table whose header is "Invariant", "Requirements" — keyed by
// invariant, each value the requirement ids listed for it.
func InvariantCoverage(body []byte) (map[string][]string, error) {
	for _, tbl := range Tables(body) {
		if len(tbl.Header) != 2 || tbl.Header[0] != "Invariant" || tbl.Header[1] != "Requirements" {
			continue
		}
		out := map[string][]string{}
		for _, row := range tbl.Rows {
			if len(row.Cells) != 2 {
				return nil, fmt.Errorf("srs: line %d: malformed invariant row", row.Line)
			}
			m := invariantCell.FindStringSubmatch(row.Cells[0])
			if m == nil {
				return nil, fmt.Errorf("srs: line %d: malformed invariant row", row.Line)
			}
			ids, err := splitList(row.Cells[1], idPattern, "requirement id")
			if err != nil {
				return nil, fmt.Errorf("srs: line %d: %w", row.Line, err)
			}
			out[m[1]] = append(out[m[1]], ids...)
		}

		return out, nil
	}

	return nil, fmt.Errorf("srs: no invariant coverage table")
}

// TracedCoverage derives the same map from the requirements' own
// Traces, for the families Appendix B covers (§5 and §6): every family
// but NFR, ENG and ASM.
func TracedCoverage(reqs []Requirement) map[string][]string {
	out := map[string][]string{}
	for _, r := range reqs {
		if !inAppendixB(r.ID) {
			continue
		}
		for _, inv := range r.Traces {
			out[inv] = append(out[inv], r.ID)
		}
	}

	return out
}

func inAppendixB(id string) bool {
	family, _, _ := strings.Cut(id, "-")

	return !slices.Contains([]string{"NFR", "ENG", "ASM"}, family)
}

// PriorityCounts counts requirements by priority, split the way
// Appendix B's count table splits them: "functional" for §5–§6 and
// "engineering" for §10. NFR and ASM carry no priority.
func PriorityCounts(reqs []Requirement) map[string]map[string]int {
	out := map[string]map[string]int{"functional": {}, "engineering": {}}
	for _, r := range reqs {
		switch {
		case r.Priority == "":
			continue
		case strings.HasPrefix(r.ID, "ENG-"):
			out["engineering"][r.Priority]++
		default:
			out["functional"][r.Priority]++
		}
	}

	return out
}

// StatedCounts reads Appendix B's requirement-count table in body:
// the rows P0, P1 and P2, and their functional and engineering
// columns.
func StatedCounts(body []byte) (map[string]map[string]int, error) {
	for _, tbl := range Tables(body) {
		if len(tbl.Header) != 3 || tbl.Header[0] != "Priority" {
			continue
		}
		out := map[string]map[string]int{"functional": {}, "engineering": {}}
		for _, row := range tbl.Rows {
			if !strings.HasPrefix(row.Cells[0], "P") {
				continue
			}
			if len(row.Cells) != 3 {
				return nil, fmt.Errorf("srs: line %d: malformed count row", row.Line)
			}
			var f, e int
			if _, err := fmt.Sscanf(row.Cells[1]+" "+row.Cells[2], "%d %d", &f, &e); err != nil {
				return nil, fmt.Errorf("srs: line %d: malformed count row: %w", row.Line, err)
			}
			// A zero is left out, the way PriorityCounts never
			// creates an entry for a priority no requirement has.
			if f > 0 {
				out["functional"][row.Cells[0]] = f
			}
			if e > 0 {
				out["engineering"][row.Cells[0]] = e
			}
		}

		return out, nil
	}

	return nil, fmt.Errorf("srs: no requirement count table")
}
