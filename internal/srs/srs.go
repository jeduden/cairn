// Package srs reads the requirement tables of the Software
// Requirements Specification under docs/srs, so the test suite can
// hold the specification and the executable scenarios in step. It is
// build-time tooling for the gates in internal/scenario and never
// links into the shipped binary.
package srs

import (
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"slices"
	"strings"
)

// Requirement is one row of a requirement or assumption table.
// Priority and Traces are empty when the table has no such column —
// the non-functional and assumption tables carry neither, the
// engineering tables no Traces.
type Requirement struct {
	ID       string
	Priority string
	Verify   string
	Traces   []string
	Text     string
	Path     string
	Line     int
}

// idPattern is the shape of every requirement id: a family prefix and
// a two-digit number.
var idPattern = regexp.MustCompile(`^(` + Families + `)-[0-9]{2}$`)

// Families lists every requirement family prefix as a regular
// expression alternation, so the scenario gate reads ids the same way.
const Families = `REC|PRV|PIN|RCL|LMK|INJ|CMP|ADM|MEM|OPS|LANE|VIEW|OWN|PEER|SEC|NFR|ENG|ASM`

// invariantPattern is the shape of one trace, I1 to I10.
var invariantPattern = regexp.MustCompile(`^I([1-9]|10)$`)

// Load reads every requirement from the Markdown files directly in
// dir, in file-name order, and reports an id two rows share.
func Load(dir string) ([]Requirement, error) {
	paths, err := filepath.Glob(filepath.Join(dir, "*.md"))
	if err != nil {
		return nil, fmt.Errorf("srs: list %s: %w", dir, err)
	}
	slices.Sort(paths)

	var all []Requirement
	seen := map[string]Requirement{}
	for _, p := range paths {
		body, err := os.ReadFile(p) //nolint:gosec // p comes from a glob over the repository's own docs
		if err != nil {
			return nil, fmt.Errorf("srs: read %s: %w", p, err)
		}
		reqs, err := Parse(p, body)
		if err != nil {
			return nil, err
		}
		for _, r := range reqs {
			if prev, ok := seen[r.ID]; ok {
				return nil, fmt.Errorf("srs: %s:%d: %s already defined at %s:%d",
					r.Path, r.Line, r.ID, prev.Path, prev.Line)
			}
			seen[r.ID] = r
		}
		all = append(all, reqs...)
	}

	return all, nil
}

// Parse reads the requirements from one document. A requirement table
// is one whose header leads with "ID" and names a "Requirement" or an
// "Assumption" column; every other table — constraints, open
// questions, glossaries — is passed by. Within a requirement table
// every row must lead with a well-formed id and trace only real
// invariants, or the row is reported with its line.
func Parse(path string, body []byte) ([]Requirement, error) {
	var out []Requirement
	for _, tbl := range Tables(body) {
		cols := columns(tbl.Header)
		if tbl.Header[0] != "ID" || !isRequirementTable(cols) {
			continue
		}
		for _, row := range tbl.Rows {
			r, err := requirementOf(path, row, cols)
			if err != nil {
				return nil, err
			}
			out = append(out, r)
		}
	}

	return out, nil
}

func isRequirementTable(cols map[string]int) bool {
	_, req := cols["Requirement"]
	_, asm := cols["Assumption"]

	return req || asm
}

// columns maps each header name to its index.
func columns(header []string) map[string]int {
	m := make(map[string]int, len(header))
	for i, h := range header {
		m[h] = i
	}

	return m
}

// requirementOf reads one table row.
func requirementOf(path string, row Row, cols map[string]int) (Requirement, error) {
	cell := func(name string) string {
		i, ok := cols[name]
		if !ok || i >= len(row.Cells) {
			return ""
		}

		return row.Cells[i]
	}

	r := Requirement{
		ID:       cell("ID"),
		Priority: cell("Pri"),
		Verify:   cell("Ver"),
		Text:     cell("Requirement") + cell("Assumption"),
		Path:     path,
		Line:     row.Line,
	}
	if !idPattern.MatchString(r.ID) {
		return Requirement{}, fmt.Errorf("srs: %s:%d: malformed requirement id %q", path, row.Line, r.ID)
	}
	traces, err := parseTraces(cell("Traces"))
	if err != nil {
		return Requirement{}, fmt.Errorf("srs: %s:%d: %s: %w", path, row.Line, r.ID, err)
	}
	r.Traces = traces

	return r, nil
}

// parseTraces reads a Traces cell: "—" or empty for none, else a
// comma-separated list of invariants.
func parseTraces(cell string) ([]string, error) {
	if cell == "" || cell == "—" {
		return nil, nil
	}

	return splitList(cell, invariantPattern, "trace")
}

// splitList reads a comma-separated cell whose every entry, trimmed,
// matches pattern; the first entry that does not is reported as a
// malformed kind.
func splitList(cell string, pattern *regexp.Regexp, kind string) ([]string, error) {
	var out []string
	for part := range strings.SplitSeq(cell, ",") {
		entry := strings.TrimSpace(part)
		if !pattern.MatchString(entry) {
			return nil, fmt.Errorf("malformed %s %q", kind, entry)
		}
		out = append(out, entry)
	}

	return out, nil
}
