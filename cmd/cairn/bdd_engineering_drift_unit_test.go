package main

import (
	"testing"

	"github.com/jeduden/cairn/internal/drift"
	"github.com/stretchr/testify/assert"
)

const guardedFeature = `Feature: x

  @ENG-01 @P0
  Scenario: inspects
    Given the repository checkout

  @ENG-02 @P0 @pending
  Scenario: pending
    Given the repository checkout

  @ENG-03 @P0
  Scenario: product
    Given an isolated Cairn home
`

func TestDriftsApply(t *testing.T) {
	e := checkout(t, map[string]string{"a.md": "one\n"})
	e.drifts = []drift.Case{{Name: "ok", Edit: drift.Edit{Op: drift.Replace, File: "a.md", Old: "one"}}}
	assert.NoError(t, e.driftsApply())

	rotted := drift.Edit{Op: drift.Replace, File: "a.md", Old: "two"}
	e.drifts = append(e.drifts, drift.Case{Name: "rotted", Edit: rotted})
	assert.ErrorContains(t, e.driftsApply(), `drift case "rotted": drift: a.md: "two" not found`)
}

func TestCheckoutScenariosGuarded(t *testing.T) {
	e := checkout(t, map[string]string{"features/x.feature": guardedFeature})

	assert.EqualError(t, e.checkoutScenariosGuarded(), "ENG-01 has no drift case")

	e.drifts = []drift.Case{{Guards: "ENG-01"}}
	assert.NoError(t, e.checkoutScenariosGuarded(), "pending and product scenarios need none")

	bad := checkout(t, map[string]string{"features/x.feature": "Feature: x\n\n  Scenario: untagged\n    Given y\n"})
	assert.Error(t, bad.checkoutScenariosGuarded())
}

func TestGuarded(t *testing.T) {
	e := &engineering{drifts: []drift.Case{{Guards: "TestA"}}}

	assert.NoError(t, e.guarded([]string{"TestA"}))
	assert.EqualError(t, e.guarded([]string{"TestA", "TestB"}), "TestB has no drift case")
	assert.Equal(t, []string{
		"TestSpecificationAndFeaturesAgree", "TestAppendixBMatchesTheTraces",
		"TestAppendixCCoversEveryRequirement", "TestPersonasMatchTheAgents",
		"TestFindingLedgerIsCarried",
	}, gateTests())
}
