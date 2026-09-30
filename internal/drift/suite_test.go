//go:build drift

package drift

import (
	"context"
	"io/fs"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

// The drift suite (ENG-27). It copies the checkout into one work
// directory, proves the unedited copy passes every registered check,
// then injects each case into a fresh copy at the same path, so the
// Go build cache still hits, and requires its check to fail with the
// case's message. CI runs it with `go test -tags drift`; the default
// `go test ./...` skips it because it needs mdsmith on PATH.

// runTimeout bounds one check; a whole `go test` run of cmd/cairn takes
// seconds with a warm cache.
const runTimeout = 5 * time.Minute

func TestUneditedCopyPassesEveryCheck(t *testing.T) {
	work := t.TempDir()
	syncCopy(t, work)

	seen := map[string]bool{}
	for _, c := range Cases() {
		key := c.Check.Tool + " " + strings.Join(c.Check.Args, " ")
		if seen[key] {
			continue
		}
		seen[key] = true
		out, err := run(t, work, c.Check)
		require.NoError(t, err, "%s fails on the unedited copy:\n%s", key, out)
	}
}

func TestEveryDriftIsCaught(t *testing.T) {
	work := t.TempDir()
	for _, c := range Cases() {
		t.Run(c.Name, func(t *testing.T) {
			syncCopy(t, work)
			require.NoError(t, Apply(work, c.Edit))

			out, err := run(t, work, c.Check)
			if c.KnownGap {
				require.NoError(t, err, "known gap %q is caught now: drop KnownGap from the case\n%s", c.Name, out)
				t.Logf("known gap, not yet caught: %s (%s)", c.Name, c.Guards)

				return
			}
			require.Error(t, err, "drift %q went uncaught by %s", c.Name, c.Guards)
			require.Contains(t, out, c.Want, "%s failed, but not with the drift's message", c.Guards)
		})
	}
}

// run executes check in dir with a deadline and returns its combined
// output.
func run(t *testing.T, dir string, check Check) (string, error) {
	t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), runTimeout)
	defer cancel()
	cmd := exec.CommandContext(ctx, check.Tool, check.Args...) //nolint:gosec // registered checks only
	cmd.Dir = dir
	out, err := cmd.CombinedOutput()

	return string(out), err
}

// syncCopy replaces work's contents with a copy of the checkout,
// leaving out .git.
func syncCopy(t *testing.T, work string) {
	t.Helper()
	entries, err := os.ReadDir(work)
	require.NoError(t, err)
	for _, e := range entries {
		require.NoError(t, os.RemoveAll(filepath.Join(work, e.Name())))
	}
	err = filepath.WalkDir(repoRoot, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		rel, err := filepath.Rel(repoRoot, path)
		if err != nil {
			return err
		}
		if d.IsDir() && d.Name() == ".git" {
			return filepath.SkipDir
		}
		dst := filepath.Join(work, rel)
		if d.IsDir() {
			return os.MkdirAll(dst, 0o700)
		}
		body, err := os.ReadFile(path) //nolint:gosec // the checkout's own files
		if err != nil {
			return err
		}

		return os.WriteFile(dst, body, 0o600) //nolint:gosec // dst is rel joined under the test's own work dir
	})
	require.NoError(t, err)
}
