package main

import (
	"errors"
	"fmt"
	"path/filepath"
	"regexp"
	"slices"
	"strconv"
	"strings"

	"github.com/jeduden/cairn/internal/adr"
	"github.com/jeduden/cairn/internal/srs"
)

// The ENG-18 and ENG-26 helpers: what the engineering scenarios check
// in the decision records under docs/adr.

// readADRs loads every decision record in dir, relative to the checkout.
func (e *engineering) readADRs(dir string) error {
	adrs, err := adr.Load(filepath.Join(e.root, dir))
	if err != nil {
		return err
	}
	e.adrDir, e.adrs = dir, adrs

	return nil
}

// accepted lists the records whose decision is in force.
func (e *engineering) accepted() []adr.ADR {
	var out []adr.ADR
	for _, a := range e.adrs {
		if a.Status == adr.Accepted {
			out = append(out, a)
		}
	}

	return out
}

// depsNamedOnce finds, for each direct dependency, the one accepted
// record that justifies it.
func (e *engineering) depsNamedOnce() error {
	by := map[string][]string{}
	for _, a := range e.accepted() {
		for _, m := range a.Modules {
			by[m.Path] = append(by[m.Path], a.ID)
		}
	}
	var errs []error
	for _, dep := range e.deps {
		if ids := by[dep]; len(ids) != 1 {
			errs = append(errs, fmt.Errorf("%s is named by %d accepted ADRs %v, want exactly one", dep, len(ids), ids))
		}
	}

	return errors.Join(errs...)
}

// modulesAreDeps refuses an accepted record that justifies a module
// go.mod no longer requires directly: a stale decision.
func (e *engineering) modulesAreDeps() error {
	var errs []error
	for _, a := range e.accepted() {
		for _, m := range a.Modules {
			if !slices.Contains(e.deps, m.Path) {
				errs = append(errs, fmt.Errorf("%s names %s, which is not a direct dependency", a.ID, m.Path))
			}
		}
	}

	return errors.Join(errs...)
}

// depsJustified checks each accepted module row is filled in, and that
// its record weighs alternatives.
func (e *engineering) depsJustified() error {
	var errs []error
	for _, a := range e.accepted() {
		if len(a.Modules) > 0 && a.Sections["Alternatives"] == "" {
			errs = append(errs, fmt.Errorf("%s weighs no alternatives", a.ID))
		}
		for _, m := range a.Modules {
			cols := []struct{ name, value string }{
				{"Purpose", m.Purpose}, {"License", m.License}, {"Maintenance", m.Maintenance},
			}
			for _, c := range cols {
				if c.value == "" {
					errs = append(errs, fmt.Errorf("%s leaves %q empty for %s", a.ID, c.name, m.Path))
				}
			}
		}
	}

	return errors.Join(errs...)
}

// requirement reads one requirement row from the checkout's SRS, so a
// scenario checks the row's own words rather than a copy of them.
func (e *engineering) requirement(id string) (srs.Requirement, error) {
	reqs, err := srs.Load(filepath.Join(e.root, "docs", "srs"))
	if err != nil {
		return srs.Requirement{}, err
	}
	for _, r := range reqs {
		if r.ID == id {
			return r, nil
		}
	}

	return srs.Requirement{}, fmt.Errorf("no requirement %s in the SRS", id)
}

// statedIn finds the first submatch of pattern in requirement id's text.
func (e *engineering) statedIn(id, pattern string) (string, error) {
	r, err := e.requirement(id)
	if err != nil {
		return "", err
	}
	m := regexp.MustCompile(pattern).FindStringSubmatch(r.Text)
	if m == nil {
		return "", fmt.Errorf("%s does not state %s", id, pattern)
	}

	return m[1], nil
}

// licensesAllowed checks every accepted module's license against the
// allow-list ENG-18 states. A family name such as "BSD" admits its
// variants: BSD-2-Clause, BSD-3-Clause.
func (e *engineering) licensesAllowed() error {
	list, err := e.statedIn("ENG-18", `allow-list \(([^)]+)\)`)
	if err != nil {
		return err
	}
	allowed := strings.Split(list, ", ")
	var errs []error
	for _, a := range e.accepted() {
		for _, m := range a.Modules {
			if !licenseAllowed(m.License, allowed) {
				errs = append(errs, fmt.Errorf("%s is licensed %q, not on the allow-list %v",
					m.Path, m.License, allowed))
			}
		}
	}

	return errors.Join(errs...)
}

func licenseAllowed(license string, allowed []string) bool {
	return slices.ContainsFunc(allowed, func(a string) bool {
		return license == a || strings.HasPrefix(license, a+"-")
	})
}

// withinTarget checks the direct dependencies against ENG-18's target.
func (e *engineering) withinTarget() error {
	stated, err := e.statedIn("ENG-18", `≤ (\d+) direct dependencies`)
	if err != nil {
		return err
	}
	n, _ := strconv.Atoi(stated) // the pattern admits digits only
	if len(e.deps) > n {
		return fmt.Errorf("%d direct dependencies, want at most %d: %v", len(e.deps), n, e.deps)
	}

	return nil
}

// listsDependencyADRs checks file links every record that names a
// module, the way its catalog renders them.
func (e *engineering) listsDependencyADRs(file string) error {
	body, err := e.read(file)
	if err != nil {
		return err
	}
	var errs []error
	for _, a := range e.adrs {
		link := e.adrDir + "/" + filepath.Base(a.Path)
		if len(a.Modules) > 0 && !strings.Contains(string(body), "("+link+")") {
			errs = append(errs, fmt.Errorf("%s does not link %s", file, link))
		}
	}

	return errors.Join(errs...)
}

// adrsComplete checks every record's identity fields.
func (e *engineering) adrsComplete() error {
	idShape := regexp.MustCompile(`^ADR-[0-9]+$`)
	var errs []error
	for _, a := range e.adrs {
		if !idShape.MatchString(a.ID) {
			errs = append(errs, fmt.Errorf("%s: id %q is not ADR-<digits>", a.Path, a.ID))
		}
		fields := []struct{ name, value string }{{"title", a.Title}, {"status", a.Status}, {"summary", a.Summary}}
		for _, f := range fields {
			if f.value == "" {
				errs = append(errs, fmt.Errorf("%s: no %s", a.Path, f.name))
			}
		}
	}

	return errors.Join(errs...)
}

// adrsNamedForID checks each file name leads with its record's id, so
// the id alone finds the file.
func (e *engineering) adrsNamedForID() error {
	var errs []error
	for _, a := range e.adrs {
		if !strings.HasPrefix(filepath.Base(a.Path), a.ID+"-") {
			errs = append(errs, fmt.Errorf("%s is not named for its id %s", a.Path, a.ID))
		}
	}

	return errors.Join(errs...)
}

// adrStatusesValid checks every status is one ENG-26 names.
func (e *engineering) adrStatusesValid() error {
	var errs []error
	for _, a := range e.adrs {
		if !slices.Contains([]string{adr.Proposed, adr.Accepted, adr.Superseded}, a.Status) {
			errs = append(errs, fmt.Errorf("%s: status %q is not proposed, accepted or superseded", a.ID, a.Status))
		}
	}

	return errors.Join(errs...)
}

// adrIDsUnique refuses two records with one id.
func (e *engineering) adrIDsUnique() error {
	seen := map[string]string{}
	var errs []error
	for _, a := range e.adrs {
		if prev, ok := seen[a.ID]; ok {
			errs = append(errs, fmt.Errorf("%s: id %s already used by %s", a.Path, a.ID, prev))

			continue
		}
		seen[a.ID] = a.Path
	}

	return errors.Join(errs...)
}

// supersededNamesSuccessor checks a superseded record points at a
// record that exists.
func (e *engineering) supersededNamesSuccessor() error {
	ids := map[string]bool{}
	for _, a := range e.adrs {
		ids[a.ID] = true
	}
	var errs []error
	for _, a := range e.adrs {
		if a.Status == adr.Superseded && !ids[a.SupersededBy] {
			errs = append(errs, fmt.Errorf("%s is superseded by %q, which is no ADR", a.ID, a.SupersededBy))
		}
	}

	return errors.Join(errs...)
}
