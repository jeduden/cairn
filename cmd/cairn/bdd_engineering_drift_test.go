package main

import (
	"errors"
	"fmt"
	"path/filepath"
	"slices"

	"github.com/cucumber/godog"
	"github.com/jeduden/cairn/internal/drift"
	"github.com/jeduden/cairn/internal/scenario"
)

// The ENG-27 steps: the drift registry covers every check that keeps
// the records in step. The drift CI job injects the cases; these steps
// check the registry itself, so they run without mdsmith.

// gateTests are the gate tests outside the scenarios that ENG-27 holds
// to a drift case: the requirement–scenario gate and the Appendix B
// check.
func gateTests() []string {
	return []string{"TestSpecificationAndFeaturesAgree", "TestAppendixBMatchesTheTraces"}
}

// checkoutStep is the step a scenario opens with when it inspects the
// repository itself.
const checkoutStep = "the repository checkout"

func init() {
	registrars = append(registrars, bindDrift)
}

func bindDrift(w *world, sc *godog.ScenarioContext) {
	e := func() *engineering { return section[engineering](w) }

	sc.Step(`^the drift cases are read$`, func() error {
		e().drifts = drift.Cases()

		return nil
	})
	sc.Step(`^the CI workflow runs the drift suite with "([^"]+)"$`, func(cmd string) error {
		return e().fileContains(".github/workflows/ci.yml", cmd)
	})
	sc.Step(`^every drift case's edit applies to the checkout$`, func() error {
		return e().driftsApply()
	})
	sc.Step(`^every non-pending scenario that inspects the repository checkout has a drift case guarding its id$`,
		func() error { return e().checkoutScenariosGuarded() })
	sc.Step(`^the requirement-scenario gate and the Appendix B check each have a drift case$`, func() error {
		return e().guarded(gateTests())
	})
}

// driftsApply refuses a case whose edit no longer finds its target, so
// a case cannot rot into injecting nothing.
func (e *engineering) driftsApply() error {
	var errs []error
	for _, c := range e.drifts {
		if err := drift.Validate(e.root, c.Edit); err != nil {
			errs = append(errs, fmt.Errorf("drift case %q: %w", c.Name, err))
		}
	}

	return errors.Join(errs...)
}

// checkoutScenariosGuarded finds the non-pending scenarios that open on
// the checkout and requires a drift case for each one's id.
func (e *engineering) checkoutScenariosGuarded() error {
	scenarios, err := scenario.Scenarios(filepath.Join(e.root, "features"))
	if err != nil {
		return err
	}
	var ids []string
	for _, s := range scenarios {
		if !s.Pending && len(s.Steps) > 0 && s.Steps[0] == checkoutStep {
			ids = append(ids, s.ID)
		}
	}

	return e.guarded(ids)
}

// guarded requires a drift case guarding each of names.
func (e *engineering) guarded(names []string) error {
	var errs []error
	for _, name := range names {
		if !slices.ContainsFunc(e.drifts, func(c drift.Case) bool { return c.Guards == name }) {
			errs = append(errs, fmt.Errorf("%s has no drift case", name))
		}
	}

	return errors.Join(errs...)
}
