package drift

import (
	"errors"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// repoRoot is the checkout the registered cases edit.
const repoRoot = "../.."

func tree(t *testing.T) string {
	t.Helper()
	root := t.TempDir()
	require.NoError(t, os.WriteFile(filepath.Join(root, "a.md"), []byte("one two one\n"), 0o600))

	return root
}

func read(t *testing.T, path string) string {
	t.Helper()
	body, err := os.ReadFile(path) //nolint:gosec // a test's own temp file
	require.NoError(t, err)

	return string(body)
}

func TestApplyReplacesTheFirstOccurrence(t *testing.T) {
	root := tree(t)

	require.NoError(t, Apply(root, Edit{Op: Replace, File: "a.md", Old: "one", New: "1"}))
	assert.Equal(t, "1 two one\n", read(t, filepath.Join(root, "a.md")))
}

func TestApplyRemovesAndCopies(t *testing.T) {
	root := tree(t)

	require.NoError(t, Apply(root, Edit{Op: Copy, File: "a.md", New: "b.md"}))
	assert.Equal(t, "one two one\n", read(t, filepath.Join(root, "b.md")))

	require.NoError(t, Apply(root, Edit{Op: Remove, File: "a.md"}))
	_, err := os.Stat(filepath.Join(root, "a.md"))
	assert.True(t, errors.Is(err, os.ErrNotExist))
}

func TestValidateRefusesAnEditThatWouldInjectNothing(t *testing.T) {
	root := tree(t)
	cases := []struct {
		edit Edit
		want string
	}{
		{Edit{Op: Replace, File: "missing.md"}, "drift: missing.md:"},
		{Edit{Op: Replace, File: "a.md", Old: "three"}, `drift: a.md: "three" not found`},
		{Edit{Op: Replace, File: "a.md"}, `drift: a.md: "" not found`},
		{Edit{Op: Copy, File: "a.md", New: "a.md"}, "copy target a.md already exists"},
		{Edit{Op: "rename", File: "a.md"}, `unknown op "rename"`},
	}
	for _, c := range cases {
		assert.ErrorContains(t, Validate(root, c.edit), c.want)
		assert.ErrorContains(t, Apply(root, c.edit), c.want)
	}
	assert.NoError(t, Validate(root, Edit{Op: Remove, File: "a.md"}))
}

func TestWrapNamesTheFile(t *testing.T) {
	assert.NoError(t, wrap(Edit{File: "a.md"}, nil))
	assert.EqualError(t, wrap(Edit{File: "a.md"}, errors.New("boom")), "drift: a.md: boom")
}

func TestCheckConstructors(t *testing.T) {
	scenarioArgs := []string{"test", "./cmd/cairn", "-count=1", "-run", "^TestFeatures$/^ENG-18:"}
	assert.Equal(t, Check{Tool: "go", Args: scenarioArgs}, Scenario("ENG-18"))
	assert.Equal(t, Check{Tool: "go", Args: []string{"test", "./internal/srs", "-count=1", "-run", "^TestX$"}},
		GoTest("./internal/srs", "TestX"))
	assert.Equal(t, Check{Tool: "mdsmith", Args: []string{"check", "."}}, Mdsmith())
}

// TestCasesApplyToTheRepository keeps the registry from rotting: every
// case's edit still finds its target in the real checkout.
func TestCasesApplyToTheRepository(t *testing.T) {
	cases := Cases()

	require.NotEmpty(t, cases)
	for _, c := range cases {
		assert.NoError(t, Validate(repoRoot, c.Edit), c.Name)
		assert.NotEmpty(t, c.Guards, c.Name)
		assert.NotEmpty(t, c.Want, c.Name)
	}
}
