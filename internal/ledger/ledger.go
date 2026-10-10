// Package ledger reads the domain model's finding ledger and checks that
// every closed finding theme is still carried in the text. A theme is
// closed as fixed, deferred (an open question and a milestone) or
// accepted (a residual risk), and each closed theme cites the sentences
// that close it; a sentence that leaves its file reopens the theme. The
// check keeps a decision from living only in a review note, and a trim
// from dropping a decided rule silently. It is build-time tooling and
// never links into the shipped binary.
package ledger

import (
	"encoding/json"
	"errors"
	"fmt"
	"io/fs"
	"strings"
)

// Status values a theme may carry.
const (
	Open     = "open"
	Fixed    = "fixed"
	Deferred = "deferred"
	Accepted = "accepted"
)

// Ledger is every finding theme the reviews raised.
type Ledger struct {
	Themes []Theme `json:"themes"`
}

// Theme is one recurring finding and how it was closed.
type Theme struct {
	ID     string `json:"id"`
	Title  string `json:"title"`
	Status string `json:"status"`
	// OQ and Milestone say where a deferred theme waits.
	OQ        string `json:"oq,omitempty"`
	Milestone string `json:"milestone,omitempty"`
	// Risk names the residual risk an accepted theme rests on.
	Risk string `json:"risk,omitempty"`
	// Texts are the sentences that close the theme.
	Texts []Text `json:"texts,omitempty"`
}

// Text is one sentence a closed theme cites, by file relative to the
// repository root.
type Text struct {
	File  string `json:"file"`
	Quote string `json:"quote"`
}

// Parse reads a ledger from its JSON.
func Parse(body []byte) (Ledger, error) {
	var l Ledger
	if err := json.Unmarshal(body, &l); err != nil {
		return Ledger{}, fmt.Errorf("ledger: parse: %w", err)
	}

	return l, nil
}

// Validate reports every theme whose identity, status or closure is
// malformed.
func (l Ledger) Validate() []error {
	var errs []error
	seen := map[string]bool{}
	for _, t := range l.Themes {
		if t.ID == "" {
			errs = append(errs, fmt.Errorf("theme %q has no id", t.Title))

			continue
		}
		if seen[t.ID] {
			errs = append(errs, fmt.Errorf("theme id %s used twice", t.ID))
		}
		seen[t.ID] = true
		errs = append(errs, t.validate()...)
	}

	return errs
}

// validate checks one theme's status and the closure it requires.
func (t Theme) validate() []error {
	var errs []error
	switch t.Status {
	case Open:
		return nil
	case Fixed:
	case Deferred:
		if !numbered(t.OQ, "OQ-") || !numbered(t.Milestone, "M") {
			errs = append(errs, fmt.Errorf("%s: deferred without an OQ-n and an Mn milestone", t.ID))
		}
	case Accepted:
		if !numbered(t.Risk, "R") {
			errs = append(errs, fmt.Errorf("%s: accepted without a residual risk Rn", t.ID))
		}
	default:
		return []error{fmt.Errorf("%s: unknown status %q", t.ID, t.Status)}
	}
	if len(t.Texts) == 0 {
		errs = append(errs, fmt.Errorf("%s: %s but cites no text", t.ID, t.Status))
	}
	for _, x := range t.Texts {
		if x.File == "" || strings.TrimSpace(x.Quote) == "" {
			errs = append(errs, fmt.Errorf("%s: a cited text needs a file and a quote", t.ID))
		}
	}

	return errs
}

// numbered reports whether s is prefix followed by one or more digits.
func numbered(s, prefix string) bool {
	rest, ok := strings.CutPrefix(s, prefix)
	if !ok || rest == "" {
		return false
	}
	for _, r := range rest {
		if r < '0' || r > '9' {
			return false
		}
	}

	return true
}

// Missing lists, as "id: file: quote", every cited text of a closed theme
// that its file no longer carries. Whitespace differences, such as a
// rewrapped line, do not count. A cited file that cannot be read is an
// error.
func (l Ledger) Missing(fsys fs.FS) ([]string, error) {
	var out []string
	texts := map[string]string{}
	for _, t := range l.Themes {
		if t.Status == Open {
			continue
		}
		for _, x := range t.Texts {
			body, ok := texts[x.File]
			if !ok {
				raw, err := fs.ReadFile(fsys, x.File)
				if err != nil {
					return nil, fmt.Errorf("ledger: %s cites %s: %w", t.ID, x.File, err)
				}
				body = normalize(string(raw))
				texts[x.File] = body
			}
			if !strings.Contains(body, normalize(x.Quote)) {
				out = append(out, fmt.Sprintf("%s: %s: %s", t.ID, x.File, x.Quote))
			}
		}
	}

	return out, nil
}

// normalize collapses every run of whitespace to one space.
func normalize(s string) string {
	return strings.Join(strings.Fields(s), " ")
}

// ErrNotCarried reports closed themes whose texts left their files.
var ErrNotCarried = errors.New("closed findings no longer carried in the text")

// Load reads the ledger at path in fsys.
func Load(fsys fs.FS, path string) (Ledger, error) {
	body, err := fs.ReadFile(fsys, path)
	if err != nil {
		return Ledger{}, fmt.Errorf("ledger: read %s: %w", path, err)
	}

	return Parse(body)
}

// Check validates l and confirms that fsys, the repository, still
// carries every closed theme's texts.
func Check(l Ledger, fsys fs.FS) error {
	if errs := l.Validate(); len(errs) > 0 {
		return fmt.Errorf("ledger: malformed: %w", errors.Join(errs...))
	}
	missing, err := l.Missing(fsys)
	if err != nil {
		return err
	}
	if len(missing) > 0 {
		return fmt.Errorf("%w:\n%s", ErrNotCarried, strings.Join(missing, "\n"))
	}

	return nil
}
