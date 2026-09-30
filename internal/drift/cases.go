package drift

// testStack is the dependency decision the ENG-18 and ENG-26 cases
// drift.
const testStack = "docs/adr/ADR-2609292234-test-stack.md"

// Cases lists every registered drift. A check that keeps the records in
// step gets a case here in the change that adds it (ENG-27); the ENG-27
// scenario fails while a repository-inspecting scenario or a gate test
// has none.
func Cases() []Case {
	var out []Case
	for _, group := range [][]Case{dependencyCases(), decisionCases(), repositoryCases(), gateCases()} {
		out = append(out, group...)
	}

	return out
}

// dependencyCases lists the drifts ENG-18 exists to catch: go.mod and the dependency ADRs
// disagreeing.
func dependencyCases() []Case {
	return []Case{
		{
			Name:   "a direct dependency no ADR justifies",
			Guards: "ENG-18",
			Edit: Edit{Op: Replace, File: "go.mod",
				Old: "\tgo.yaml.in/yaml/v3 v3.0.5 // indirect", New: "\tgo.yaml.in/yaml/v3 v3.0.5"},
			Check: Scenario("ENG-18"),
			Want:  "go.yaml.in/yaml/v3 is named by 0 accepted ADRs",
		},
		{
			Name:   "the ADR justifying the test stack removed",
			Guards: "ENG-18",
			Edit:   Edit{Op: Remove, File: testStack},
			Check:  Scenario("ENG-18"),
			Want:   "github.com/cucumber/godog is named by 0 accepted ADRs",
		},
		{
			Name:   "an ADR still justifying a module go.mod dropped",
			Guards: "ENG-18",
			Edit: Edit{Op: Replace, File: testStack,
				Old: "| `github.com/stretchr/testify`",
				New: "| `example.com/stale` | p | MIT | m |\n| `github.com/stretchr/testify`"},
			Check: Scenario("ENG-18"),
			Want:  "names example.com/stale, which is not a direct dependency",
		},
		{
			Name:   "a license off the allow-list",
			Guards: "ENG-18",
			Edit: Edit{Op: Replace, File: testStack,
				Old: "| MIT     | Active; the de-facto", New: "| GPL-3.0 | Active; the de-facto"},
			Check: Scenario("ENG-18"),
			Want:  `is licensed "GPL-3.0", not on the allow-list`,
		},
	}
}

// decisionCases lists the drifts ENG-26 exists to catch in the decision records.
func decisionCases() []Case {
	return []Case{
		{
			Name:   "two ADRs sharing an id",
			Guards: "ENG-26",
			Edit:   Edit{Op: Copy, File: testStack, New: "docs/adr/ADR-2609292234-copy.md"},
			Check:  Scenario("ENG-26"),
			Want:   "id ADR-2609292234 already used by",
		},
		{
			Name:   "an ADR superseded without a successor",
			Guards: "ENG-26",
			Edit:   Edit{Op: Replace, File: testStack, Old: "status: accepted", New: "status: superseded"},
			Check:  Scenario("ENG-26"),
			Want:   `is superseded by "", which is no other ADR`,
		},
		{
			// Phase 3 of plan 2609292156 moves ADR-01 to ADR-10 into files
			// and makes this drift fail ENG-26.
			Name:   "the SRS citing an ADR with no file",
			Guards: "ENG-26",
			Edit: Edit{Op: Replace, File: "docs/srs/01-introduction.md",
				Old: "## 1.1 Purpose\n", New: "## 1.1 Purpose\n\nSee ADR-99.\n"},
			Check:    Scenario("ENG-26"),
			Want:     "ADR-99",
			KnownGap: true,
		},
	}
}

// repositoryCases lists the drifts the other repository-inspecting scenarios catch.
func repositoryCases() []Case {
	return []Case{
		{
			Name:   "release builds no longer static",
			Guards: "ENG-01",
			Edit: Edit{Op: Replace, File: ".github/workflows/release.yml",
				Old: `CGO_ENABLED: "0"`, New: `CGO_ENABLED: "1"`},
			Check: Scenario("ENG-01"),
			Want:  `does not contain "CGO_ENABLED: \"0\""`,
		},
		{
			Name:   "the disclosure policy dropped",
			Guards: "ENG-24",
			Edit: Edit{Op: Replace, File: "SECURITY.md",
				Old: "We follow a 90-day coordinated disclosure policy.", New: "We follow a disclosure policy."},
			Check: Scenario("ENG-24"),
			Want:  `does not contain "90-day coordinated disclosure"`,
		},
		{
			Name:   "the drift suite dropped from CI",
			Guards: "ENG-27",
			Edit: Edit{Op: Replace, File: ".github/workflows/ci.yml",
				Old: "go test -tags drift ./internal/drift", New: "true"},
			Check: Scenario("ENG-27"),
			Want:  `does not contain "go test -tags drift ./internal/drift"`,
		},
	}
}

// gateCases lists the drifts the gate tests and mdsmith catch between the SRS,
// the scenarios and the generated sections.
func gateCases() []Case {
	return []Case{
		{
			Name:   "a requirement left without its scenario",
			Guards: "TestSpecificationAndFeaturesAgree",
			Edit: Edit{Op: Replace, File: "features/engineering.feature",
				Old: "@ENG-25 @P1 @pending", New: "@ENG-99 @P1 @pending"},
			Check: GoTest("./internal/scenario", "TestSpecificationAndFeaturesAgree"),
			Want:  "ENG-25: no scenario under features/",
		},
		{
			Name:   "a scenario's priority off its requirement's",
			Guards: "TestSpecificationAndFeaturesAgree",
			Edit: Edit{Op: Replace, File: "features/engineering.feature",
				Old: "@ENG-25 @P1 @pending", New: "@ENG-25 @P0 @pending"},
			Check: GoTest("./internal/scenario", "TestSpecificationAndFeaturesAgree"),
			Want:  `ENG-25: scenario priority "P0", requirement says "P1"`,
		},
		{
			Name:   "a requirement demoted without Appendix B's count",
			Guards: "TestAppendixBMatchesTheTraces",
			Edit: Edit{Op: Replace, File: "docs/srs/10-engineering-quality.md",
				Old: "| ENG-25 | P1  |", New: "| ENG-25 | P2  |"},
			Check: GoTest("./internal/srs", "TestAppendixBMatchesTheTraces"),
			Want:  "Not equal",
		},
		{
			Name:   "an ADR edited without regenerating DEPENDENCIES.md",
			Guards: "mdsmith check",
			Edit: Edit{Op: Replace, File: testStack,
				Old: "All three are test-only.", New: "All three are test-only, for now."},
			Check: Mdsmith(),
			Want:  "generated section is out of date",
		},
	}
}
