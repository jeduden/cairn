//! The registry: every drift a check that keeps the records in step
//! exists to catch.

use crate::{Case, Check, Injection, cargo_test, mdsmith, scenario};

/// The dependency decision the ENG-18 and ENG-26 cases drift.
const TEST_STACK: &str = "docs/adr/ADR-2610101442-rust-test-stack.md";

/// The decision [`TEST_STACK`] superseded.
const OLD_TEST_STACK: &str = "docs/adr/ADR-2609292234-test-stack.md";

/// The workflow the ENG-28 cases drift.
const REVIEW_WORKFLOW: &str = ".github/workflows/review.yml";

/// Lists every registered drift. A check that keeps the records in step
/// gets a case here in the change that adds it (ENG-27); the ENG-27
/// scenario fails while a repository-inspecting scenario or a gate test
/// has none.
#[must_use]
pub fn cases() -> Vec<Case> {
    [
        dependency_cases(),
        decision_cases(),
        repository_cases(),
        review_cases(),
        gate_cases(),
        persona_cases(),
        agent_cases(),
        domain_model_cases(),
        ledger_cases(),
    ]
    .concat()
}

fn case(
    name: &'static str,
    guards: &'static str,
    injection: Injection,
    check: Check,
    want: &'static str,
) -> Case {
    Case {
        name,
        guards,
        injection,
        check,
        want,
        known_gap: false,
    }
}

/// The drifts ENG-18 exists to catch: the manifests and the dependency
/// ADRs disagreeing.
fn dependency_cases() -> Vec<Case> {
    vec![
        case(
            "a direct dependency no ADR justifies",
            "ENG-18",
            Injection::replace(
                "go.mod",
                "require go.yaml.in/yaml/v3 v3.0.5 // indirect",
                "require go.yaml.in/yaml/v3 v3.0.5",
            ),
            scenario("ENG-18"),
            "go.yaml.in/yaml/v3 is named by 0 accepted ADRs",
        ),
        case(
            "the ADR justifying the test stack removed",
            "ENG-18",
            Injection::remove(TEST_STACK),
            scenario("ENG-18"),
            "cucumber is named by 0 accepted ADRs",
        ),
        case(
            "an ADR still justifying a module the manifests dropped",
            "ENG-18",
            Injection::replace(
                TEST_STACK,
                "| `serde_json`",
                "| `example.com/stale` | p | MIT | m |\n| `serde_json`",
            ),
            scenario("ENG-18"),
            "names example.com/stale, which is not a direct dependency",
        ),
        case(
            "a license off the allow-list",
            "ENG-18",
            Injection::replace(
                TEST_STACK,
                "MIT OR Apache-2.0 | Active; the de-facto Rust",
                "GPL-3.0 | Active; the de-facto Rust",
            ),
            scenario("ENG-18"),
            r#"serde is licensed "GPL-3.0", not on the allow-list"#,
        ),
    ]
}

/// The drifts ENG-26 exists to catch in the decision records.
fn decision_cases() -> Vec<Case> {
    vec![
        case(
            "two ADRs sharing an id",
            "ENG-26",
            Injection::copy(TEST_STACK, "docs/adr/ADR-2610101442-copy.md"),
            scenario("ENG-26"),
            "id ADR-2610101442 already used by",
        ),
        case(
            "an ADR superseded without a successor",
            "ENG-26",
            Injection::replace(TEST_STACK, "status: accepted", "status: superseded"),
            scenario("ENG-26"),
            r#"is superseded by "", which is no other ADR"#,
        ),
        case(
            "a superseded ADR left in force beside its successor",
            "ENG-26",
            Injection::replace(OLD_TEST_STACK, "status: superseded", "status: accepted"),
            scenario("ENG-26"),
            r#"ADR-2609292234 names successor ADR-2610101442 but its status is "accepted", not superseded"#,
        ),
        // Phase 3 of plan 2609292156 moves ADR-01 to ADR-10 into files
        // and makes this drift fail ENG-26.
        Case {
            known_gap: true,
            ..case(
                "the SRS citing an ADR with no file",
                "ENG-26",
                Injection::replace(
                    "docs/srs/01-introduction.md",
                    "## 1.1 Purpose\n",
                    "## 1.1 Purpose\n\nSee ADR-99.\n",
                ),
                scenario("ENG-26"),
                "ADR-99",
            )
        },
    ]
}

/// The drifts the other repository-inspecting scenarios catch.
fn repository_cases() -> Vec<Case> {
    vec![
        case(
            "release builds no longer static",
            "ENG-01",
            Injection::replace(
                ".github/workflows/release.yml",
                r#"CGO_ENABLED: "0""#,
                r#"CGO_ENABLED: "1""#,
            ),
            scenario("ENG-01"),
            r#"does not contain "CGO_ENABLED: \"0\"""#,
        ),
        case(
            "the Rust toolchain no longer pinned to a release",
            "ENG-01",
            Injection::replace(
                "rust-toolchain.toml",
                r#"channel = "1.99.0""#,
                r#"channel = "stable""#,
            ),
            scenario("ENG-01"),
            r#"rust-toolchain.toml does not pin an exact release: "stable""#,
        ),
        case(
            "the disclosure policy dropped",
            "ENG-24",
            Injection::replace(
                "SECURITY.md",
                "We follow a 90-day coordinated disclosure policy.",
                "We follow a disclosure policy.",
            ),
            scenario("ENG-24"),
            r#"does not contain "90-day coordinated disclosure""#,
        ),
        case(
            "the drift suite dropped from CI",
            "ENG-27",
            Injection::replace(
                ".github/workflows/ci.yml",
                "cargo test --locked -p drift --test suite -- --ignored",
                "true",
            ),
            scenario("ENG-27"),
            r#"does not contain "cargo test --locked -p drift --test suite -- --ignored""#,
        ),
    ]
}

/// The drifts ENG-28 exists to catch: the reviewing agent reaching the
/// approval, or the approval skipping the review gate.
fn review_cases() -> Vec<Case> {
    vec![
        case(
            "the reviewer run as the pull request defines it",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "  workflow_run:\n",
                "  pull_request_target:\n  workflow_run:\n",
            ),
            scenario("ENG-28"),
            "triggers on [pull_request_target workflow_run], want only workflow_run",
        ),
        case(
            "the reviewing agent granted a write permission",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "      pull-requests: read\n",
                "      pull-requests: write\n",
            ),
            scenario("ENG-28"),
            "agent job review is granted a write permission",
        ),
        case(
            "the reviewing agent handed the reviewer app's key",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "          github_token: ${{ github.token }}\n",
                "          github_token: ${{ secrets.JEDUDEN_REVIEW_AGENT_KEY }}\n",
            ),
            scenario("ENG-28"),
            "agent job review reads JEDUDEN_REVIEW_AGENT_KEY",
        ),
        case(
            "the review posted after the agent failed",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "    if: needs.review.outputs.pull != ''\n",
                "    if: always() && needs.review.outputs.pull != ''\n",
            ),
            scenario("ENG-28"),
            "job post runs on always(), even after a failed review",
        ),
        case(
            "the review posted without the review gate",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "cargo run --locked --quiet -p review-gate -- --outcome",
                "cp outcome.json review.json; true --outcome",
            ),
            scenario("ENG-28"),
            r#"job post does not run "cargo run --locked --quiet -p review-gate""#,
        ),
    ]
}

/// The drifts the gate tests and mdsmith catch between the SRS, the
/// scenarios and the generated sections.
fn gate_cases() -> Vec<Case> {
    vec![
        case(
            "a requirement left without its scenario",
            "specification_and_features_agree",
            Injection::replace(
                "features/engineering.feature",
                "@ENG-25 @P1 @pending",
                "@ENG-99 @P1 @pending",
            ),
            cargo_test("scenario", "gate", "specification_and_features_agree"),
            "ENG-25: no scenario under features/",
        ),
        case(
            "a scenario's priority off its requirement's",
            "specification_and_features_agree",
            Injection::replace(
                "features/engineering.feature",
                "@ENG-25 @P1 @pending",
                "@ENG-25 @P0 @pending",
            ),
            cargo_test("scenario", "gate", "specification_and_features_agree"),
            r#"ENG-25: scenario priority "P0", requirement says "P1""#,
        ),
        case(
            "a requirement demoted without Appendix B's count",
            "appendix_b_matches_the_traces",
            Injection::replace(
                "docs/srs/10-engineering-quality.md",
                "| ENG-25 | P1  |",
                "| ENG-25 | P2  |",
            ),
            cargo_test("srs", "gates", "appendix_b_matches_the_traces"),
            "Appendix B's requirement counts",
        ),
        case(
            "an ADR edited without regenerating DEPENDENCIES.md",
            "mdsmith check",
            Injection::replace(
                TEST_STACK,
                "All four serve tests and tooling only;",
                "All four serve tests only;",
            ),
            mdsmith(),
            "generated section is out of date",
        ),
    ]
}

/// The drifts the persona gates catch between §2.5, Appendix C and the
/// persona agents.
fn persona_cases() -> Vec<Case> {
    const APPENDIX_C: &str = "docs/srs/appendix-c-persona-coverage.md";
    let appendix_c_check = || cargo_test("srs", "gates", "appendix_c_covers_every_requirement");
    let agents_check = || cargo_test("srs", "gates", "personas_match_the_agents");
    vec![
        case(
            "a requirement serving no persona",
            "appendix_c_covers_every_requirement",
            Injection::replace(APPENDIX_C, "| SEC-17, ENG-29", "| SEC-17"),
            appendix_c_check(),
            "ENG-29 serves no persona in Appendix C",
        ),
        case(
            "Appendix C listing an id that is no requirement",
            "appendix_c_covers_every_requirement",
            Injection::replace(APPENDIX_C, "SEC-17, ENG-29", "SEC-17, ENG-29, ENG-98"),
            appendix_c_check(),
            "Appendix C lists ENG-98, which is no requirement",
        ),
        case(
            "Appendix C naming a persona §2.5 does not define",
            "appendix_c_covers_every_requirement",
            Injection::replace(
                "docs/srs/02-context.md",
                "\n| U9  | Agent",
                "\n\n| U9  | Agent",
            ),
            appendix_c_check(),
            "Appendix C names U9, which §2.5 does not define",
        ),
        case(
            "a persona agent with no persona",
            "personas_match_the_agents",
            Injection::copy(
                ".claude/agents/persona-agent.md",
                ".claude/agents/persona-stray.md",
            ),
            agents_check(),
            "persona-stray",
        ),
        case(
            "a persona whose agent file is gone",
            "personas_match_the_agents",
            Injection::remove(".claude/agents/persona-reviewer.md"),
            agents_check(),
            "persona-reviewer",
        ),
    ]
}

/// The drifts mdsmith's agent schemas catch: a reviewing agent given a
/// tool beyond reading, and the domain-model agent losing one of its
/// required sections.
fn agent_cases() -> Vec<Case> {
    vec![
        case(
            "a reviewing agent granted a write tool",
            "mdsmith check",
            Injection::replace(
                ".claude/agents/persona-reviewer.md",
                "tools: Read, Grep, Glob",
                "tools: Read, Grep, Glob, Edit",
            ),
            mdsmith(),
            "tools: got",
        ),
        case(
            "the domain-model agent losing a required section",
            "mdsmith check",
            Injection::replace(
                ".claude/agents/domain-model.md",
                "## How you report",
                "## Reporting",
            ),
            mdsmith(),
            "How you report",
        ),
    ]
}

/// The drifts mdsmith's domain-model schemas catch: the hub losing one
/// of the sections that span every concept group, and a concept file
/// losing the summary the hub's catalog reads.
fn domain_model_cases() -> Vec<Case> {
    vec![
        case(
            "the domain-model hub losing a required section",
            "mdsmith check",
            Injection::replace(
                "docs/domain-model/index.md",
                "## Not Cairn concepts",
                "## Excluded terms",
            ),
            mdsmith(),
            "Not Cairn concepts",
        ),
        case(
            "a domain-model concept file losing its summary",
            "mdsmith check",
            Injection::replace("docs/domain-model/places.md", "summary: >-", "abstract: >-"),
            mdsmith(),
            "summary",
        ),
    ]
}

/// The drift the finding ledger's check exists to catch: a closed
/// domain-model finding whose closing sentence leaves the text.
fn ledger_cases() -> Vec<Case> {
    vec![case(
        "a closed finding's sentence trimmed from the model",
        "finding_ledger_is_carried",
        Injection::replace(
            "docs/domain-model/components-and-surfaces.md",
            "What shows rooms to a person: the browser, through the",
            "What shows rooms to a person: the",
        ),
        cargo_test("finding-ledger", "carried", "finding_ledger_is_carried"),
        "closed findings no longer carried in the text",
    )]
}

#[cfg(test)]
mod tests;
