---
n: 3
title: "Settings layers and principal acts at the terminal"
status: "✅"
result: true
summary: >-
  Settings layers validated and layered; widening acts gated by OWN-22
  and managed policy; counter ack, configuration accept and risk
  acceptance recorded as principal acts. ADM-04's rows 1 to 4 and the
  M1 steps of OWN-11, -12 and -22 pass narrowed; none comes off @pending.
---
# Phase 3 result

## Handoff

Outcome:

- ADM-04, SEC-11, PRV-04's setting, PRV-05, and SEC-22's managed policy
  part hold in the core: `e2e_settings.rs` drives each, confined. The
  workspace's 76 live scenarios pass.
- `@ADM-04` rows 1 to 4 and the M1 steps of `@OWN-11`, `@OWN-12` and
  `@OWN-22` pass with their outlines narrowed locally; none comes off
  `@pending`, as phase-3.md says why.
- I2, I6, I7 and I9 hold: no layer trusts a class, every ignored key,
  failed layer and refused act is audited and counted, a repository
  configuration only tightens, and recording goes on past a bad layer.

Inherited:

- The `configuration` crate: `load(&Sources)` reads the three layers,
  `resolve(&Layers, &accepted)` combines them. `key::lookup` holds
  §9.6's keys and their repository and acceptance rules.
- `administration::act`: `Act`, `Class`, `permit(&Act, &Gate)`,
  `record`, `acts`, `open_residual_risks`. Every run seat counts as an
  unsandboxed run, leaving R1 to R8 open.
- `cairn configuration accept`, `cairn risk-acceptance
  show|record|withdraw`; `cairn counter ack` notes a `principal_act`
  audit entry, then records the act through it.
- Audit counters `settings_layer_invalid`,
  `repository_configuration_key_ignored`, `trust_policy_key_ignored`,
  `managed_policy_invalid`, `principal_act_refused`; `Occurrence::Keyed`
  carries a settings key name, never other text.
- Steps start the core with managed policy read from the scenario's
  home; `administration.rs` and `principal_acts.rs` hold the new steps.
- Main's red `every_crate_holds_the_complexity_limits` is fixed: the
  `audit` and `hook` crates carry the ceilings.

For the stakeholder:

- `@ADM-04` row 5 needs flags (PRV-07, P1) and `@PRV-05`'s `post` and
  `summary` rows need rooms, so neither scenario can pass in M1.
- The scenarios spell the ignored-key audit entry three ways (PRV-04,
  SEC-11, PRV-05); the core writes the domain model's words.
- The crate name `configuration` covers three settings layers.
- Do ended runs keep OWN-22's residual risks open? M1 counts them all.

Parked:

- `cairn hook` and `cairn mcp` read no settings yet (phases 9, 10).
- The parser's fuzz target joins phase 7's first `fuzz/` crate.
- Managed policy that fails also refuses widening acts, beyond
  ADM-04's letter; Windows' ACLs are unchecked.
- A recording command audits ignored keys on every run.
- A refused act audited in a fresh home creates the home.
- `home.id` does not write `home.json` yet.
- An unknown key's name reaches `cairn status`'s stderr when it holds
  only a settings key's characters, up to 160: for security review.
