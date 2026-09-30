package main

import (
	"bufio"
	"bytes"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/cucumber/godog"
	"github.com/jeduden/cairn/internal/adr"
	"github.com/jeduden/cairn/internal/drift"
)

func init() {
	registrars = append(registrars, bindEngineering)
}

// engineering is what the §10 scenarios track beside the shared world:
// the checkout they inspect and what a step read out of it.
type engineering struct {
	root   string
	deps   []string
	adrDir string
	adrs   []adr.ADR
	drifts []drift.Case
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
	sc.Step(`^the ADRs are read from "([^"]+)"$`, func(dir string) error {
		return e().readADRs(dir)
	})
	sc.Step(`^each direct dependency is named by exactly one accepted ADR$`, func() error {
		return e().depsNamedOnce()
	})
	sc.Step(`^every module an accepted ADR names is a direct dependency$`, func() error {
		return e().modulesAreDeps()
	})
	sc.Step(`^each named module has a purpose, a license and a maintenance status, and its ADR weighs alternatives$`,
		func() error { return e().depsJustified() })
	sc.Step(`^each license is on the allow-list the ENG-18 requirement states$`, func() error {
		return e().licensesAllowed()
	})
	sc.Step(`^the direct dependencies stay within the target the ENG-18 requirement states$`, func() error {
		return e().withinTarget()
	})
	sc.Step(`^"([^"]+)" lists every ADR that names a module$`, func(file string) error {
		return e().listsDependencyADRs(file)
	})
	sc.Step(`^every ADR has an id, a title, a status and a summary$`, func() error {
		return e().adrsComplete()
	})
	sc.Step(`^every ADR's file is named for its id$`, func() error {
		return e().adrsNamedForID()
	})
	sc.Step(`^every ADR's status is proposed, accepted or superseded$`, func() error {
		return e().adrStatusesValid()
	})
	sc.Step(`^no two ADRs share an id$`, func() error {
		return e().adrIDsUnique()
	})
	sc.Step(`^every superseded ADR names an ADR that exists as its successor$`, func() error {
		return e().supersededNamesSuccessor()
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
