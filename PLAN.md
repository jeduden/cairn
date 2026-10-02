# Plans

## In progress

<?catalog
glob:
  - "plan/*.md"
  - "plan/*/plan.md"
  - "!plan/proto.md"
where: 'status: "🔳"'
sort: numeric:id
header: |

  | ID  | Model | Title |
  |-----|-------|-------|
row: "| {id} | {model} | [{title}]({filename}) |"
footer: |

empty: |

  Nothing in progress.

?>

| ID         | Model  | Title                                                                                                       |
| ---------- | ------ | ----------------------------------------------------------------------------------------------------------- |
| 2609292156 | sonnet | [The repository's knowledge follows Cairn's layers](plan/2609292156_knowledge-follows-cairn-layers/plan.md) |
<?/catalog?>

## All plans

<?catalog
glob:
  - "plan/*.md"
  - "plan/*/plan.md"
  - "!plan/proto.md"
sort: numeric:id
header: |

  | ID  | Status | Model | Title |
  |-----|--------|-------|-------|
row: "| {id} | {status} | {model} | [{title}]({filename}) |"
footer: |

empty: |

  No plans yet.

?>

| ID         | Status | Model  | Title                                                                                                              |
| ---------- | ------ | ------ | ------------------------------------------------------------------------------------------------------------------ |
| 2609292002 | ✅     | opus   | [Bootstrap the repository: SRS, requirement matrix, CI and release](plan/2609292002_bootstrap-repository.md)       |
| 2609292003 | 🔲     | sonnet | [Spike S1: record the hook contract](plan/2609292003_spike-s1-hook-contract.md)                                    |
| 2609292004 | ✅     | opus   | [Spike S2: choose the pure-Go SQLite driver](plan/2609292004_spike-s2-sqlite-driver/plan.md)                       |
| 2609292005 | 🔲     | sonnet | [Spike S3: catalogue the transcript formats](plan/2609292005_spike-s3-transcript-formats.md)                       |
| 2609292006 | 🔲     | sonnet | [Spike S4: prove plugin distribution](plan/2609292006_spike-s4-plugin-distribution.md)                             |
| 2609292007 | 🔲     | opus   | [Spike S5: Starlark or Python for the kernel](plan/2609292007_spike-s5-kernel-language.md)                         |
| 2609292008 | 🔲     | sonnet | [Spike S6: can compaction be deferred](plan/2609292008_spike-s6-compaction-control.md)                             |
| 2609292009 | 🔲     | sonnet | [Spike S7: the official MCP Go SDK or an in-house server](plan/2609292009_spike-s7-mcp-sdk.md)                     |
| 2609292010 | 🔲     | sonnet | [Spike S8: calibrate the token estimator and freeze the targets](plan/2609292010_spike-s8-token-estimator.md)      |
| 2609292011 | 🔲     | sonnet | [The remaining engineering gates in CI](plan/2609292011_ci-engineering-gates.md)                                   |
| 2609292012 | 🔲     | opus   | [M1: record and recall](plan/2609292012_m1-record-and-recall.md)                                                   |
| 2609292013 | 🔲     | opus   | [M2: pins and restore](plan/2609292013_m2-pins-and-restore.md)                                                     |
| 2609292014 | 🔲     | opus   | [M3: landmarks and lifecycle](plan/2609292014_m3-landmarks-and-lifecycle.md)                                       |
| 2609292015 | 🔲     | opus   | [M4: the hermetic compute kernel](plan/2609292015_m4-compute-kernel.md)                                            |
| 2609292016 | 🔲     | opus   | [M5: hardening and evaluation](plan/2609292016_m5-hardening-and-evaluation.md)                                     |
| 2609292156 | 🔳     | sonnet | [The repository's knowledge follows Cairn's layers](plan/2609292156_knowledge-follows-cairn-layers/plan.md)        |
| 2609301942 | ✅     | sonnet | [Agent review through the reviewer app](plan/2609301942_agent-review-app/plan.md)                                  |
| 2610012322 | 🔲     | opus   | [Scope Cairn for agent fleets: shared, real-time, public sessions](plan/2610012322_cairn-for-agent-fleets/plan.md) |
| 2610022338 | 🔲     | sonnet | [Build Cairn's network side: sync agent, relay and public host](plan/2610022338_cairn-network-side/plan.md)        |
<?/catalog?>
