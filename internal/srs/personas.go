package srs

import (
	"fmt"
	"regexp"
	"slices"
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
// id listed twice, a malformed id, an empty persona list, a persona
// outside U1–U9 or one named twice in a row is reported with its line.
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
			ids, err := splitList(row.Cells[0], idPattern, "requirement id")
			if err != nil {
				return nil, fmt.Errorf("srs: line %d: %w", row.Line, err)
			}
			for _, id := range ids {
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
// list of U1–U9, each named once.
func splitPersonas(cell string) ([]string, error) {
	out, err := splitList(cell, personaPattern, "persona")
	if err != nil {
		return nil, err
	}
	for i, p := range out {
		if slices.Contains(out[:i], p) {
			return nil, fmt.Errorf("persona %s listed twice", p)
		}
	}

	return out, nil
}

// PersonaAgents reads §2.5's persona table in body — the table whose
// header leads with "#", "Persona", "Agent" — keyed by persona, each
// value the agent that encodes it under .claude/agents. A persona
// listed twice is reported with its line.
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
			if _, dup := out[row.Cells[0]]; dup {
				return nil, fmt.Errorf("srs: line %d: %s listed twice", row.Line, row.Cells[0])
			}
			out[row.Cells[0]] = m[1]
		}

		return out, nil
	}

	return nil, fmt.Errorf("srs: no persona table")
}
