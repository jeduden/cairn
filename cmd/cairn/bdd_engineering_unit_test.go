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

func TestReadDirectDepsReportsAMissingFile(t *testing.T) {
	assert.ErrorContains(t, checkout(t, nil).readDirectDeps("go.mod"), "read go.mod")
}
