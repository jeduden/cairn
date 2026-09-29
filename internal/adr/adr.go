// Package adr reads the architecture decision records under docs/adr
// (ENG-26): one file per decision, its identity and status in front
// matter, its reasoning in sections. A decision about dependencies
// also carries a Module table, which ENG-18 checks against go.mod. It
// is build-time tooling and never links into the shipped binary.
package adr

import (
	"bufio"
	"bytes"
	"fmt"
	"os"
	"path/filepath"
	"slices"
	"strings"

	"github.com/jeduden/cairn/internal/srs"
)

// Status values an ADR may carry.
const (
	Proposed   = "proposed"
	Accepted   = "accepted"
	Superseded = "superseded"
)

// ADR is one decision record.
type ADR struct {
	Path         string
	ID           string
	Title        string
	Status       string
	Summary      string
	SupersededBy string
	// Sections maps each level-2 heading to the trimmed text under it.
	Sections map[string]string
	// Modules is the decision's Module table, empty when it names no
	// dependency.
	Modules []Module
}

// Module is one row of an ADR's Module table.
type Module struct {
	Path        string
	Purpose     string
	License     string
	Maintenance string
}

// Load reads every ADR directly in dir, in file-name order.
func Load(dir string) ([]ADR, error) {
	paths, err := filepath.Glob(filepath.Join(dir, "*.md"))
	if err != nil {
		return nil, fmt.Errorf("adr: list %s: %w", dir, err)
	}
	slices.Sort(paths)

	out := make([]ADR, 0, len(paths))
	for _, p := range paths {
		body, err := os.ReadFile(p) //nolint:gosec // p comes from a glob over the repository's own docs
		if err != nil {
			return nil, fmt.Errorf("adr: read %s: %w", p, err)
		}
		a, err := Parse(p, body)
		if err != nil {
			return nil, err
		}
		out = append(out, a)
	}

	return out, nil
}

// Parse reads one ADR from body. CRLF line endings read as LF, so a
// Windows checkout parses the same as any other.
func Parse(path string, body []byte) (ADR, error) {
	body = bytes.ReplaceAll(body, []byte("\r\n"), []byte("\n"))
	fm, rest, err := frontMatter(body)
	if err != nil {
		return ADR{}, fmt.Errorf("adr: %s: %w", path, err)
	}
	secs := sections(rest)

	return ADR{
		Path:         path,
		ID:           fm["id"],
		Title:        fm["title"],
		Status:       fm["status"],
		Summary:      fm["summary"],
		SupersededBy: fm["superseded-by"],
		Sections:     secs,
		Modules:      modules([]byte(secs["Decision"])),
	}, nil
}

// frontMatter splits body into its front matter, read as flat scalar
// keys, and the rest. A folded or literal block scalar (">-", "|")
// joins its indented lines with spaces. Anything else — a list, a
// nested map — is refused rather than misread.
func frontMatter(body []byte) (map[string]string, []byte, error) {
	head, rest, ok := bytes.Cut(body, []byte("\n---\n"))
	if !ok || !bytes.HasPrefix(head, []byte("---\n")) {
		return nil, nil, fmt.Errorf("no front matter")
	}

	out := map[string]string{}
	var block string
	sc := bufio.NewScanner(bytes.NewReader(head[len("---\n"):]))
	for n := 2; sc.Scan(); n++ {
		line := sc.Text()
		switch {
		case strings.TrimSpace(line) == "":
			continue
		case line[0] == ' ' || line[0] == '\t':
			if block == "" {
				return nil, nil, fmt.Errorf("line %d: unsupported front matter %q", n, line)
			}
			out[block] = strings.TrimSpace(out[block] + " " + strings.TrimSpace(line))

			continue
		}
		key, value, ok := strings.Cut(line, ":")
		if !ok || key == "" {
			return nil, nil, fmt.Errorf("line %d: malformed front matter %q", n, line)
		}
		value = strings.TrimSpace(value)
		block = ""
		if slices.Contains([]string{">", ">-", "|", "|-"}, value) {
			block, value = key, ""
		}
		out[key] = unquote(value)
	}
	if err := sc.Err(); err != nil {
		return nil, nil, fmt.Errorf("scan front matter: %w", err)
	}

	return out, rest, nil
}

// unquote strips one pair of matching single or double quotes.
func unquote(s string) string {
	if len(s) >= 2 && (s[0] == '"' || s[0] == '\'') && s[len(s)-1] == s[0] {
		return s[1 : len(s)-1]
	}

	return s
}

// sections maps each level-2 heading in body to the trimmed text
// under it, up to the next level-2 heading. A "## " line inside a
// fenced code block is text, not a heading.
func sections(body []byte) map[string]string {
	out := map[string]string{}
	var (
		name, fence string
		text        []string
	)
	flush := func() {
		if name != "" {
			out[name] = strings.TrimSpace(strings.Join(text, "\n"))
		}
	}
	for line := range strings.Lines(string(body)) {
		fence = fenceAfter(fence, line)
		if h, ok := strings.CutPrefix(strings.TrimRight(line, "\n"), "## "); ok && fence == "" {
			flush()
			name, text = strings.TrimSpace(h), nil

			continue
		}
		text = append(text, strings.TrimRight(line, "\n"))
	}
	flush()

	return out
}

// fenceAfter returns the fence open after line, given the fence open
// before it: a fence closes only on a run of its own character at
// least as long, the way srs.Tables reads fences.
func fenceAfter(open, line string) string {
	f := srs.FenceMarker(strings.TrimSpace(line))
	switch {
	case f == "":
		return open
	case open == "":
		return f
	case f[0] == open[0] && len(f) >= len(open):
		return ""
	}

	return open
}

// modules reads body's Module table: the table whose header leads with
// "Module", each module path with its backticks trimmed.
func modules(body []byte) []Module {
	var out []Module
	for _, tbl := range srs.Tables(body) {
		if tbl.Header[0] != "Module" {
			continue
		}
		col := map[string]int{}
		for i, h := range tbl.Header {
			col[h] = i
		}
		cell := func(r srs.Row, name string) string {
			if i, ok := col[name]; ok && i < len(r.Cells) {
				return r.Cells[i]
			}

			return ""
		}
		for _, r := range tbl.Rows {
			out = append(out, Module{
				Path:        strings.Trim(r.Cells[0], "`"),
				Purpose:     cell(r, "Purpose"),
				License:     cell(r, "License"),
				Maintenance: cell(r, "Maintenance"),
			})
		}
	}

	return out
}
