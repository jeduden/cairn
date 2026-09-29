package main

import (
	"bytes"
	"os"
	"path/filepath"
	"reflect"
	"testing"

	"github.com/cucumber/godog"
	"github.com/jeduden/cairn/internal/scenario"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// featuresDir holds the executable scenarios: one tagged Gherkin
// scenario per SRS requirement, kept in bijection by
// internal/scenario's gate.
const featuresDir = "../../features"

// repoRoot is the repository the engineering scenarios inspect.
const repoRoot = "../.."

// TestFeatures runs every scenario under features/ as its own subtest,
// named "<id>: <title>" so `-run 'TestFeatures/^REC-01:'` picks exactly
// one out. A scenario tagged @pending is declared but unwritten: it is
// skipped, and reported as such, rather than run as a pass that proves
// nothing. Every other scenario runs through godog in strict mode, so
// a step whose text matches no definition fails instead of passing as
// undefined. The suite lives in cmd/cairn because this is the package
// where everything a requirement can name meets: the CLI, the hook
// and MCP entry points, and the fixtures that drive them.
func TestFeatures(t *testing.T) {
	scenarios, err := scenario.Scenarios(featuresDir)
	require.NoError(t, err)

	for _, sc := range scenarios {
		t.Run(sc.ID+": "+sc.Name, func(t *testing.T) {
			if sc.Pending {
				t.Skip("pending: declared in the SRS, its steps not yet written")
			}
			runScenario(t, sc.ID, sc.Path)
		})
	}
}

// world is the state one scenario threads through its steps. It is
// built on the subtest's own *testing.T, with HOME and CAIRN_HOME
// pointed at fresh temporary directories (ENG-14), so no step can
// reach the real user's home or Claude Code configuration.
type world struct {
	t        *testing.T
	home     string
	cairn    string
	sections map[reflect.Type]any
}

// newWorld builds an isolated world for one scenario.
func newWorld(t *testing.T) *world {
	t.Helper()
	home := t.TempDir()
	cairnHome := filepath.Join(home, ".cairn")
	t.Setenv("HOME", home)
	t.Setenv("CAIRN_HOME", cairnHome)

	return &world{t: t, home: home, cairn: cairnHome, sections: map[reflect.Type]any{}}
}

// registrar binds one section's step texts to the scenario's world.
type registrar func(*world, *godog.ScenarioContext)

// registrars is the step registry. Each section's step file —
// bdd_<section>_test.go — appends its registrar from init, so writing
// a section adds a file and never edits this one, and two sections
// land in any order. godog runs strict, so a step text two sections
// both define fails as ambiguous rather than shadowing silently:
// reuse the existing text.
var registrars []registrar

// bindAll registers every section's steps on w for one scenario run.
func bindAll(w *world, sc *godog.ScenarioContext, regs []registrar) {
	w.t.Helper()
	for _, bind := range regs {
		bind(w, sc)
	}
}

// section is a section's own state beside the shared world, keyed by
// its type: created on first use, the same value for the rest of the
// scenario, gone with the world. A section declares a struct for what
// its scenarios track and never adds a field to world.
func section[T any](w *world) *T {
	key := reflect.TypeFor[T]()
	if v, ok := w.sections[key]; ok {
		return v.(*T)
	}
	v := new(T)
	w.sections[key] = v

	return v
}

// runScenario drives the one scenario tagged id, in the feature file
// at path, through godog on this subtest's own goroutine, so a failing
// step fails exactly this subtest. godog's report is shown only when
// the scenario fails.
func runScenario(t *testing.T, id, path string) {
	t.Helper()
	var report bytes.Buffer
	w := newWorld(t)
	suite := godog.TestSuite{
		ScenarioInitializer: func(sc *godog.ScenarioContext) { bindAll(w, sc, registrars) },
		Options: &godog.Options{
			Paths:    []string{path},
			Tags:     "@" + id,
			Strict:   true,
			Format:   "progress",
			NoColors: true,
			Output:   &report,
		},
	}
	if status := suite.Run(); status != 0 {
		t.Errorf("scenario %s: godog exit status %d\n%s", id, status, report.String())
	}
}

func TestNewWorldIsolatesHomeAndCairnHome(t *testing.T) {
	realHome := os.Getenv("HOME")

	w := newWorld(t)

	assert.Equal(t, w.home, os.Getenv("HOME"))
	assert.Equal(t, w.cairn, os.Getenv("CAIRN_HOME"))
	assert.NotEqual(t, realHome, w.home)
	assert.Equal(t, filepath.Join(w.home, ".cairn"), w.cairn)
}

func TestBindAllBindsEveryRegistrarOnOneWorld(t *testing.T) {
	var seen []*world
	probe := func(w *world, _ *godog.ScenarioContext) { seen = append(seen, w) }
	w := newWorld(t)

	bindAll(w, nil, []registrar{probe, probe})

	assert.Equal(t, []*world{w, w}, seen)
	assert.Same(t, t, w.t)
	assert.NotEmpty(t, registrars, "the engineering steps register themselves")
}

func TestSectionStateIsOnePerTypePerWorld(t *testing.T) {
	type store struct{ path string }
	w := newWorld(t)

	first := section[store](w)
	first.path = "/s"

	assert.Same(t, first, section[store](w))
	assert.Equal(t, "/s", section[store](w).path)
	assert.NotSame(t, first, section[store](newWorld(t)), "another scenario, another value")
}
