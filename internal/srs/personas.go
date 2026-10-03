package srs

import (
	"fmt"
	"regexp"
	"strings"
)

var (
	// personaPattern is the shape of one persona, U1 to U9.
	personaPattern = regexp.MustCompile(`^U[1-9]$`)
	// agentCell is a §2.5 Agent cell: one backticked agent name.
	agentCell = regexp.MustCompile("^`(persona-[a-z-]+)`$")
)

// PersonaCoverage reads Appendix C's coverage table in body — the
// table whose header is "Requirements", "Personas" — keyed by
// requirement id, each value the personas that requirement serves. An
// id listed twice, a malformed id, an empty persona list or a persona
// outside U1–U9 is reported with its line.
func PersonaCoverage(body []byte) (map[string][]string, error) {
	for _, tbl := range Tables(body) {
		if len(tbl.Header) != 2 || tbl.Header[0] != "Requirements" || tbl.Header[1] != "Personas" {
			continue
		}
		out := map[string][]string{}
		for _, row := range tbl.Rows {
			if len(row.Cells) != 2 {
				return nil, fmt.Errorf("srs: line %d: malformed persona coverage row", row.Line)
			}
			personas, err := splitPersonas(row.Cells[1])
			if err != nil {
				return nil, fmt.Errorf("srs: line %d: %w", row.Line, err)
			}
			for id := range strings.SplitSeq(row.Cells[0], ",") {
				id = strings.TrimSpace(id)
				if !idPattern.MatchString(id) {
					return nil, fmt.Errorf("srs: line %d: malformed requirement id %q", row.Line, id)
				}
				if _, dup := out[id]; dup {
					return nil, fmt.Errorf("srs: line %d: %s listed twice", row.Line, id)
				}
				out[id] = personas
			}
		}

		return out, nil
	}

	return nil, fmt.Errorf("srs: no persona coverage table")
}

// splitPersonas reads a Personas cell: a comma-separated, non-empty
// list of U1–U9.
func splitPersonas(cell string) ([]string, error) {
	var out []string
	for part := range strings.SplitSeq(cell, ",") {
		p := strings.TrimSpace(part)
		if !personaPattern.MatchString(p) {
			return nil, fmt.Errorf("malformed persona %q", p)
		}
		out = append(out, p)
	}

	return out, nil
}

// PersonaAgents reads §2.5's persona table in body — the table whose
// header leads with "#", "Persona", "Agent" — keyed by persona, each
// value the agent that encodes it under .claude/agents.
func PersonaAgents(body []byte) (map[string]string, error) {
	for _, tbl := range Tables(body) {
		if len(tbl.Header) < 3 || tbl.Header[0] != "#" || tbl.Header[1] != "Persona" || tbl.Header[2] != "Agent" {
			continue
		}
		out := map[string]string{}
		for _, row := range tbl.Rows {
			if len(row.Cells) < 3 || !personaPattern.MatchString(row.Cells[0]) {
				return nil, fmt.Errorf("srs: line %d: malformed persona row", row.Line)
			}
			m := agentCell.FindStringSubmatch(row.Cells[2])
			if m == nil {
				return nil, fmt.Errorf("srs: line %d: malformed persona row", row.Line)
			}
			out[row.Cells[0]] = m[1]
		}

		return out, nil
	}

	return nil, fmt.Errorf("srs: no persona table")
}
