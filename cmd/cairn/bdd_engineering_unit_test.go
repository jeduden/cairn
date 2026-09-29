package main

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// checkout lays files into a fresh directory standing in for the
// repository, so the engineering helpers' failure paths are reachable.
func checkout(t *testing.T, files map[string]string) *engineering {
	t.Helper()
	root := t.TempDir()
	for name, body := range files {
		path := filepath.Join(root, name)
		require.NoError(t, os.MkdirAll(filepath.Dir(path), 0o700))
		require.NoError(t, os.WriteFile(path, []byte(body), 0o600))
	}

	return &engineering{root: root}
}

func TestDirectDepsSkipsIndirectAndReadsBothForms(t *testing.T) {
	gomod := []byte(`module x

go 1.26.0

require example.com/one v1.0.0

require (
	example.com/two v1.0.0
	example.com/three v1.0.0 // indirect
)

require (
	example.com/four v1.0.0
)
`)

	assert.Equal(t, []string{"example.com/one", "example.com/two", "example.com/four"}, directDeps(gomod))
}

func TestToolchainPinned(t *testing.T) {
	e := checkout(t, map[string]string{"a.mod": "go 1.26.0\ntoolchain go1.26.8\n", "b.mod": "go 1.26.0\n"})

	assert.NoError(t, e.toolchainPinned("a.mod"))
	assert.ErrorContains(t, e.toolchainPinned("b.mod"), "no toolchain directive")
	assert.ErrorContains(t, e.toolchainPinned("missing.mod"), "read missing.mod")
}

func TestFileContains(t *testing.T) {
	e := checkout(t, map[string]string{".github/workflows/release.yml": "go build -trimpath\n"})

	assert.NoError(t, e.releaseContains("-trimpath"))
	assert.ErrorContains(t, e.releaseContains("-buildvcs=true"), `does not contain "-buildvcs=true"`)
	assert.ErrorContains(t, e.fileContains("nope.md", "x"), "read nope.md")
}

const depsTable = `# Dependencies

| Module | Purpose | License | Maintenance | Alternatives |
|---|---|---|---|---|
| ` + "`example.com/one`" + ` | tests | MIT | active | none |
| example.com/two | tests | GPL-3.0 | active | none |
| example.com/three | tests | MIT |  | none |
`

func TestDepsJustifiedAndLicenses(t *testing.T) {
	e := checkout(t, map[string]string{"DEPENDENCIES.md": depsTable})

	e.deps = []string{"example.com/one"}
	assert.NoError(t, e.depsJustified("DEPENDENCIES.md"))
	assert.NoError(t, e.licensesAllowed("MIT, ISC"))

	e.deps = []string{"example.com/missing"}
	assert.ErrorContains(t, e.depsJustified("DEPENDENCIES.md"), "no row for example.com/missing")

	e.deps = []string{"example.com/three"}
	assert.ErrorContains(t, e.depsJustified("DEPENDENCIES.md"), `leaves "Maintenance" empty`)

	e.deps = []string{"example.com/two"}
	assert.ErrorContains(t, e.licensesAllowed("MIT, ISC"), `licensed "GPL-3.0"`)
}

func TestDependencyHelpersReportAMissingFile(t *testing.T) {
	e := checkout(t, nil)

	assert.ErrorContains(t, e.depsJustified("DEPENDENCIES.md"), "read DEPENDENCIES.md")
	assert.ErrorContains(t, e.licensesAllowed("MIT"), "read DEPENDENCIES.md")
	assert.ErrorContains(t, e.readDirectDeps("go.mod"), "read go.mod")
}
