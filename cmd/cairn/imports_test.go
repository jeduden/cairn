package main

import (
	"os/exec"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestShippedBinaryLinksNoNetworkOrProcessPackage is the import half
// of SEC-01 and ENG-16, run on every build: the transitive import
// closure of cmd/cairn — what actually links into the shipped binary,
// test code excluded — may not contain a network package or os/exec.
// depguard in .golangci.yml stops a direct import in cairn's own
// code; this catches the same package arriving through a dependency.
// When a package genuinely needs one (the kernel worker spawning its
// child, CMP-05), it joins an allow-list here, in the same change,
// under review.
func TestShippedBinaryLinksNoNetworkOrProcessPackage(t *testing.T) {
	out, err := exec.Command("go", "list", "-deps", "-f", "{{.ImportPath}}", ".").Output()
	require.NoError(t, err)

	assert.Empty(t, forbiddenImports(strings.Fields(string(out))))
}

// forbiddenImports lists the packages in deps that a shipped binary
// may not link: net and everything under it, and os/exec.
func forbiddenImports(deps []string) []string {
	var bad []string
	for _, p := range deps {
		if p == "net" || strings.HasPrefix(p, "net/") || p == "os/exec" {
			bad = append(bad, p)
		}
	}

	return bad
}

func TestForbiddenImportsNamesNetworkAndExec(t *testing.T) {
	got := forbiddenImports([]string{"fmt", "net", "net/http", "network/x", "os", "os/exec", "os/execute"})

	assert.Equal(t, []string{"net", "net/http", "os/exec"}, got)
}
