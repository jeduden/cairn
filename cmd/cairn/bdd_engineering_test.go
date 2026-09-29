package main

import (
	"bufio"
	"bytes"
	"fmt"
	"os"
	"path/filepath"
	"slices"
	"strings"

	"github.com/cucumber/godog"
	"github.com/jeduden/cairn/internal/srs"
)

func init() {
	registrars = append(registrars, bindEngineering)
}

// engineering is what the §10 scenarios track beside the shared world:
// the checkout they inspect and what a step read out of it.
type engineering struct {
	root string
	deps []string
}

// bindEngineering binds the step texts of features/engineering.feature.
func bindEngineering(w *world, sc *godog.ScenarioContext) {
	e := func() *engineering { return section[engineering](w) }

	sc.Step(`^the repository checkout$`, func() error {
		e().root = repoRoot

		return nil
	})
	sc.Step(`^"([^"]+)" pins the Go toolchain with a toolchain directive$`, func(file string) error {
		return e().toolchainPinned(file)
	})
	sc.Step(`^the release workflow sets CGO_ENABLED to "0"$`, func() error {
		return e().releaseContains(`CGO_ENABLED: "0"`)
	})
	sc.Step(`^the release workflow builds with "([^"]+)"$`, func(flag string) error {
		return e().releaseContains(flag)
	})
	sc.Step(`^the direct dependencies are read from "([^"]+)"$`, func(file string) error {
		return e().readDirectDeps(file)
	})
	sc.Step(`^each one has a row in "([^"]+)" naming its purpose, license, maintenance status and alternatives$`,
		func(file string) error { return e().depsJustified(file) })
	sc.Step(`^each license is one of "([^"]+)"$`, func(list string) error {
		return e().licensesAllowed(list)
	})
	sc.Step(`^there are at most (\d+) direct dependencies$`, func(n int) error {
		if len(e().deps) > n {
			return fmt.Errorf("%d direct dependencies, want at most %d: %v", len(e().deps), n, e().deps)
		}

		return nil
	})
	sc.Step(`^"SECURITY.md" links the private vulnerability reporting channel$`, func() error {
		return e().fileContains("SECURITY.md", "/security/advisories/new")
	})
	sc.Step(`^"SECURITY.md" states a 90-day coordinated disclosure policy$`, func() error {
		return e().fileContains("SECURITY.md", "90-day coordinated disclosure")
	})
	sc.Step(`^"SECURITY.md" states that security fixes are backported to the latest minor release$`, func() error {
		return e().fileContains("SECURITY.md", "backported to the latest minor release")
	})
}

func (e *engineering) read(file string) ([]byte, error) {
	body, err := os.ReadFile(filepath.Join(e.root, file)) //nolint:gosec // a repository file the scenario names
	if err != nil {
		return nil, fmt.Errorf("read %s: %w", file, err)
	}

	return body, nil
}

func (e *engineering) fileContains(file, want string) error {
	body, err := e.read(file)
	if err != nil {
		return err
	}
	if !bytes.Contains(body, []byte(want)) {
		return fmt.Errorf("%s does not contain %q", file, want)
	}

	return nil
}

func (e *engineering) releaseContains(want string) error {
	return e.fileContains(".github/workflows/release.yml", want)
}

// toolchainPinned finds a `toolchain goX.Y.Z` line in file.
func (e *engineering) toolchainPinned(file string) error {
	body, err := e.read(file)
	if err != nil {
		return err
	}
	for line := range strings.Lines(string(body)) {
		if f := strings.Fields(line); len(f) == 2 && f[0] == "toolchain" && strings.HasPrefix(f[1], "go1.") {
			return nil
		}
	}

	return fmt.Errorf("%s has no toolchain directive", file)
}

// readDirectDeps lists the module paths go.mod requires without an
// "// indirect" marker, in either the block or the one-line form.
func (e *engineering) readDirectDeps(file string) error {
	body, err := e.read(file)
	if err != nil {
		return err
	}
	e.deps = directDeps(body)

	return nil
}

func directDeps(gomod []byte) []string {
	var (
		deps    []string
		inBlock bool
	)
	sc := bufio.NewScanner(bytes.NewReader(gomod))
	for sc.Scan() {
		line := strings.TrimSpace(sc.Text())
		switch {
		case line == "require (":
			inBlock = true

			continue
		case inBlock && line == ")":
			inBlock = false

			continue
		case strings.HasPrefix(line, "require "):
			line = strings.TrimPrefix(line, "require ")
		case !inBlock:
			continue
		}
		if f := strings.Fields(line); len(f) >= 2 && !strings.Contains(line, "// indirect") {
			deps = append(deps, f[0])
		}
	}

	return deps
}

// dependencyRows reads the DEPENDENCIES.md table, keyed by module, each
// value the row's cells by header name.
func (e *engineering) dependencyRows(file string) (map[string]map[string]string, error) {
	body, err := e.read(file)
	if err != nil {
		return nil, err
	}
	rows := map[string]map[string]string{}
	for _, tbl := range srs.Tables(body) {
		if tbl.Header[0] != "Module" {
			continue
		}
		for _, r := range tbl.Rows {
			cells := map[string]string{}
			for i, h := range tbl.Header {
				if i < len(r.Cells) {
					cells[h] = r.Cells[i]
				}
			}
			rows[strings.Trim(r.Cells[0], "`")] = cells
		}
	}

	return rows, nil
}

func (e *engineering) depsJustified(file string) error {
	rows, err := e.dependencyRows(file)
	if err != nil {
		return err
	}
	for _, dep := range e.deps {
		row, ok := rows[dep]
		if !ok {
			return fmt.Errorf("%s has no row for %s", file, dep)
		}
		for _, col := range []string{"Purpose", "License", "Maintenance", "Alternatives"} {
			if row[col] == "" {
				return fmt.Errorf("%s: %s leaves %q empty", file, dep, col)
			}
		}
	}

	return nil
}

func (e *engineering) licensesAllowed(list string) error {
	allowed := strings.Split(list, ", ")
	rows, err := e.dependencyRows("DEPENDENCIES.md")
	if err != nil {
		return err
	}
	for _, dep := range e.deps {
		if lic := rows[dep]["License"]; !slices.Contains(allowed, lic) {
			return fmt.Errorf("%s is licensed %q, not on the allow-list %v", dep, lic, allowed)
		}
	}

	return nil
}
