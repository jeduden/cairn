//! ENG-14's checks against the repository checkout itself: the agent
//! instruction files, and the one place tests resolve the real home.

use engineering::{Checkout, agent_instructions};

fn checkout() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

/// No agent instruction file directs a destructive command at
/// non-isolated state (ENG-14).
#[test]
fn the_agent_instructions_direct_no_destructive_command_at_real_state() {
    let problems = agent_instructions::no_destructive_instructions(&checkout());

    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The scan reads the instruction files this repository carries.
#[test]
fn the_agent_instructions_scan_reads_the_repository_s_files() {
    let files = agent_instructions::instruction_files(&checkout().root).unwrap();

    for want in ["CLAUDE.md", "AGENTS.md", ".claude/settings.json"] {
        assert!(files.iter().any(|f| f == want), "{want} not in {files:?}");
    }
    assert!(files.iter().any(|f| f.starts_with(".claude/agents/")));
    assert!(files.iter().any(|f| f.starts_with(".claude/skills/")));
}
