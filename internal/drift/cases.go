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
	for _, group := range [][]Case{
		dependencyCases(), decisionCases(), repositoryCases(), reviewCases(),
		gateCases(), personaCases(), agentCases(), domainModelCases(), ledgerCases(),
	} {
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

// reviewWorkflow is the workflow the ENG-28 cases drift.
const reviewWorkflow = ".github/workflows/review.yml"

// reviewCases lists the drifts ENG-28 exists to catch: the reviewing
// agent reaching the approval, or the approval skipping the gate.
func reviewCases() []Case {
	return []Case{
		{
			Name:   "the reviewer run as the pull request defines it",
			Guards: "ENG-28",
			Edit: Edit{Op: Replace, File: reviewWorkflow,
				Old: "  workflow_run:\n", New: "  pull_request_target:\n  workflow_run:\n"},
			Check: Scenario("ENG-28"),
			Want:  "triggers on [pull_request_target workflow_run], want only workflow_run",
		},
		{
			Name:   "the reviewing agent granted a write permission",
			Guards: "ENG-28",
			Edit: Edit{Op: Replace, File: reviewWorkflow,
				Old: "      pull-requests: read\n", New: "      pull-requests: write\n"},
			Check: Scenario("ENG-28"),
			Want:  "agent job review is granted a write permission",
		},
		{
			Name:   "the reviewing agent handed the reviewer app's key",
			Guards: "ENG-28",
			Edit: Edit{Op: Replace, File: reviewWorkflow,
				Old: "          github_token: ${{ github.token }}\n",
				New: "          github_token: ${{ secrets.JEDUDEN_REVIEW_AGENT_KEY }}\n"},
			Check: Scenario("ENG-28"),
			Want:  "agent job review reads JEDUDEN_REVIEW_AGENT_KEY",
		},
		{
			Name:   "the review posted after the agent failed",
			Guards: "ENG-28",
			Edit: Edit{Op: Replace, File: reviewWorkflow,
				Old: "    if: needs.review.outputs.pull != ''\n",
				New: "    if: always() && needs.review.outputs.pull != ''\n"},
			Check: Scenario("ENG-28"),
			Want:  "job post runs on always(), even after a failed review",
		},
		{
			Name:   "the review posted without the gate",
			Guards: "ENG-28",
			Edit: Edit{Op: Replace, File: reviewWorkflow,
				Old: "go run ./cmd/review-gate -verdict", New: "cp verdict.json review.json; true -verdict"},
			Check: Scenario("ENG-28"),
			Want:  `job post does not run "go run ./cmd/review-gate"`,
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

// personaCases lists the drifts the persona gates catch between §2.5,
// Appendix C and the persona agents.
func personaCases() []Case {
	return []Case{
		{
			Name:   "a requirement serving no persona",
			Guards: "TestAppendixCCoversEveryRequirement",
			Edit: Edit{Op: Replace, File: "docs/srs/appendix-c-persona-coverage.md",
				Old: "| SEC-17, ENG-29", New: "| SEC-17"},
			Check: GoTest("./internal/srs", "TestAppendixCCoversEveryRequirement"),
			Want:  "ENG-29 serves no persona in Appendix C",
		},
		{
			Name:   "Appendix C listing an id that is no requirement",
			Guards: "TestAppendixCCoversEveryRequirement",
			Edit: Edit{Op: Replace, File: "docs/srs/appendix-c-persona-coverage.md",
				Old: "SEC-17, ENG-29", New: "SEC-17, ENG-29, ENG-98"},
			Check: GoTest("./internal/srs", "TestAppendixCCoversEveryRequirement"),
			Want:  "Appendix C lists ENG-98, which is no requirement",
		},
		{
			Name:   "Appendix C naming a persona §2.5 does not define",
			Guards: "TestAppendixCCoversEveryRequirement",
			Edit: Edit{Op: Replace, File: "docs/srs/02-context.md",
				Old: "\n| U9  | Agent", New: "\n\n| U9  | Agent"},
			Check: GoTest("./internal/srs", "TestAppendixCCoversEveryRequirement"),
			Want:  "Appendix C names U9, which §2.5 does not define",
		},
		{
			Name:   "a persona agent with no persona",
			Guards: "TestPersonasMatchTheAgents",
			Edit: Edit{Op: Copy, File: ".claude/agents/persona-agent.md",
				New: ".claude/agents/persona-stray.md"},
			Check: GoTest("./internal/srs", "TestPersonasMatchTheAgents"),
			Want:  "persona-stray",
		},
		{
			Name:   "a persona whose agent file is gone",
			Guards: "TestPersonasMatchTheAgents",
			Edit:   Edit{Op: Remove, File: ".claude/agents/persona-reviewer.md"},
			Check:  GoTest("./internal/srs", "TestPersonasMatchTheAgents"),
			Want:   "persona-reviewer",
		},
	}
}

// agentCases lists the drifts mdsmith's agent schemas catch: a reviewing
// agent given a tool beyond reading, and the domain-model agent losing
// one of its required sections.
func agentCases() []Case {
	return []Case{
		{
			Name:   "a reviewing agent granted a write tool",
			Guards: "mdsmith check",
			Edit: Edit{Op: Replace, File: ".claude/agents/persona-reviewer.md",
				Old: "tools: Read, Grep, Glob", New: "tools: Read, Grep, Glob, Edit"},
			Check: Mdsmith(),
			Want:  "tools: got",
		},
		{
			Name:   "the domain-model agent losing a required section",
			Guards: "mdsmith check",
			Edit: Edit{Op: Replace, File: ".claude/agents/domain-model.md",
				Old: "## How you report", New: "## Reporting"},
			Check: Mdsmith(),
			Want:  "How you report",
		},
	}
}

// domainModelCases lists the drifts mdsmith's domain-model schemas catch:
// the hub losing one of the sections that span every concept group, and a
// concept file losing the summary the hub's catalog reads.
func domainModelCases() []Case {
	return []Case{
		{
			Name:   "the domain-model hub losing a required section",
			Guards: "mdsmith check",
			Edit: Edit{Op: Replace, File: "docs/domain-model/index.md",
				Old: "## Not Cairn concepts", New: "## Excluded terms"},
			Check: Mdsmith(),
			Want:  "Not Cairn concepts",
		},
		{
			Name:   "a domain-model concept file losing its summary",
			Guards: "mdsmith check",
			Edit: Edit{Op: Replace, File: "docs/domain-model/places.md",
				Old: "summary: >-", New: "abstract: >-"},
			Check: Mdsmith(),
			Want:  "summary",
		},
	}
}

// ledgerCases lists the drift the finding ledger's check exists to catch:
// a closed domain-model finding whose closing sentence leaves the text.
func ledgerCases() []Case {
	return []Case{
		{
			Name:   "a closed finding's sentence trimmed from the model",
			Guards: "TestFindingLedgerIsCarried",
			Edit: Edit{Op: Replace, File: "docs/domain-model/components-and-surfaces.md",
				Old: "What shows rooms to a person: the browser, through the", New: "What shows rooms to a person: the"},
			Check: GoTest("./internal/ledger", "TestFindingLedgerIsCarried"),
			Want:  "closed findings no longer carried in the text",
		},
	}
}
