// Package drift registers, for every check that keeps the
// specification, its scenarios and the repository's records in step,
// the drift that check exists to catch (ENG-27). A case is data: one
// edit to a copy of the repository, the check to run on it, and the
// message the check must fail with. The drift-tagged suite in this
// package injects each case; the ENG-27 scenario checks the registry
// itself. It is build-time tooling and never links into the shipped
// binary.
package drift

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// Op is what an Edit does to its file.
type Op string

// The edits a case can make.
const (
	// Replace swaps the first occurrence of Old for New.
	Replace Op = "replace"
	// Remove deletes the file.
	Remove Op = "remove"
	// Copy copies the file to New, which must not exist yet.
	Copy Op = "copy"
)

// Edit is one change to a file, its path relative to the repository.
type Edit struct {
	Op   Op
	File string
	Old  string
	New  string
}

// Check is the command a case runs in the edited copy.
type Check struct {
	Tool string
	Args []string
}

// Case is one registered drift.
type Case struct {
	// Name says what drifted.
	Name string
	// Guards is the requirement id or gate test the case proves.
	Guards string
	Edit   Edit
	Check  Check
	// Want is text the check must print when it fails.
	Want string
	// KnownGap marks a drift the checks miss today. The suite
	// requires the check to pass on it, so the mark fails the build
	// the day a check starts catching it.
	KnownGap bool
}

// Scenario runs the one godog scenario tagged id.
func Scenario(id string) Check {
	return Check{Tool: "go", Args: []string{"test", "./cmd/cairn", "-count=1", "-run", "^TestFeatures$/^" + id + ":"}}
}

// GoTest runs one named test in pkg.
func GoTest(pkg, name string) Check {
	return Check{Tool: "go", Args: []string{"test", pkg, "-count=1", "-run", "^" + name + "$"}}
}

// Mdsmith lints every Markdown file, generated sections included.
func Mdsmith() Check {
	return Check{Tool: "mdsmith", Args: []string{"check", "."}}
}

// Validate reports whether e can be applied to the repository at root
// without changing anything, so a case whose target moved fails loudly
// instead of injecting nothing.
func Validate(root string, e Edit) error {
	_, err := validate(root, e)

	return err
}

// validate checks e against the repository at root and returns the
// file's current contents.
func validate(root string, e Edit) ([]byte, error) {
	body, err := os.ReadFile(filepath.Join(root, e.File)) //nolint:gosec // a repository file a registered case names
	if err != nil {
		return nil, fmt.Errorf("drift: %s: %w", e.File, err)
	}
	switch e.Op {
	case Replace:
		if e.Old == "" || !strings.Contains(string(body), e.Old) {
			return nil, fmt.Errorf("drift: %s: %q not found", e.File, e.Old)
		}
	case Remove:
	case Copy:
		if _, err := os.Stat(filepath.Join(root, e.New)); !errors.Is(err, os.ErrNotExist) {
			return nil, fmt.Errorf("drift: %s: copy target %s already exists", e.File, e.New)
		}
	default:
		return nil, fmt.Errorf("drift: %s: unknown op %q", e.File, e.Op)
	}

	return body, nil
}

// Apply makes e in the repository at root.
func Apply(root string, e Edit) error {
	body, err := validate(root, e)
	if err != nil {
		return err
	}
	path := filepath.Join(root, e.File)
	switch e.Op {
	case Remove:
		return wrap(e, os.Remove(path))
	case Copy:
		return wrap(e, os.WriteFile(filepath.Join(root, e.New), body, 0o600))
	default:
		return wrap(e, os.WriteFile(path, []byte(strings.Replace(string(body), e.Old, e.New, 1)), 0o600))
	}
}

// wrap names the edited file on a failure.
func wrap(e Edit, err error) error {
	if err != nil {
		return fmt.Errorf("drift: %s: %w", e.File, err)
	}

	return nil
}
