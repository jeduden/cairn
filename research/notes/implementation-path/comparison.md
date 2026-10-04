# Implementation path: comparing the options

Scope: step 2 of answering OQ-32, written 4 October 2026. It rates the
six [options](options.md) on the fifteen criteria of the
[evaluation frame](constraints.md#comparison-criteria), in the frame's
order. It is a desk assessment from the
[terminal](terminal-components.md),
[protocol and data](protocol-and-data-components.md),
[UI and packaging](ui-and-packaging-components.md) and
[prior-art](prior-art-stacks.md) notes. Every criterion names a
measurement; where a rating rests on a measurement already taken, the
note says so, and the rest wait for the spikes at the end.

## Ratings

- **strong**: meets the criterion with maintained parts as they are.
- **workable**: meets it with Cairn's own code or one stated caveat.
- **weak**: experimental, unmeasured on a risky point, or a poor fit.
- **fails**: breaks a hard constraint as the SRS words it today.

## The table

| #   | Criterion                         | A. Go, hypermedia | B. Go and a TS page | C. Go host, WASM cores | D. Rust and Tauri | E. TypeScript on Bun | F. Kotlin Multiplatform |
| --- | --------------------------------- | ----------------- | ------------------- | ---------------------- | ----------------- | -------------------- | ----------------------- |
| 1   | Hook path under the full binary   | strong            | strong              | strong                 | strong            | weak                 | weak                    |
| 2   | Boundary evidence in one binary   | strong            | strong              | strong                 | workable          | fails                | weak                    |
| 3   | Determinism and cryptography      | strong            | strong              | strong                 | strong            | workable             | weak                    |
| 4   | Store engine                      | strong            | strong              | strong                 | strong            | strong               | workable                |
| 5   | Dependency budget and licences    | workable          | workable            | workable               | weak              | fails                | weak                    |
| 6   | One reproducible, signed artifact | strong            | workable            | workable               | workable          | weak                 | weak                    |
| 7   | Run component: hosting and pty    | strong            | strong              | strong                 | strong            | workable             | weak                    |
| 8   | Lane view and the three panes     | workable          | strong              | strong                 | strong            | strong               | weak                    |
| 9   | Terminal parity                   | strong            | strong              | strong                 | strong            | workable             | weak                    |
| 10  | One view model across surfaces    | strong            | strong              | strong                 | strong            | workable             | strong                  |
| 11  | Git and signatures in the core    | workable          | workable            | workable               | strong            | workable             | weak                    |
| 12  | Kernel and MCP                    | workable          | workable            | workable               | strong            | weak                 | weak                    |
| 13  | P2 reach without a rewrite        | workable          | workable            | strong                 | strong            | workable             | weak                    |
| 14  | Verification toolchain            | strong            | strong              | strong                 | workable          | weak                 | weak                    |
| 15  | Cost of the change                | strong            | strong              | strong                 | weak              | fails                | weak                    |

## Why each row reads as it does

1. **Hook path.** A Go or Rust static binary starts in milliseconds
   and loads embedded assets only in the lane view. Bun's runtime and a
   95 MB image meet the 50 MiB hook budget (NFR-09) only if measured to;
   Kotlin/Native's start-up and garbage collector are unmeasured.
2. **Boundary evidence.** Go already has the import-closure test; per
   entry point plus a call graph covers `init` code (HC-3). C adds each
   module's import list, which is everything a WASM module can call.
   Rust needs a crate per component, clippy bans and a symbol check,
   all still to build. In Bun `fetch`, `Bun.listen` and `Bun.spawn` are
   globals, so no build-time evidence exists, as HC-3 requires.
3. **Determinism and cryptography.** Go's standard library holds
   SHA-256, Ed25519, HMAC and an AEAD; canonical JSON is own code or one
   small module. Rust needs a crate for each but all are strong.
   JavaScript's number formatting is what RFC 8785 was written around,
   but bans on `Date.now` and `Math.random` are lint-only. Kotlin/Native
   has no canonical JSON library.
4. **Store.** SQLite with FTS5 is strong in Go (the proposed driver,
   measured at 10M events), Rust and Bun; SQLDelight's native driver is
   workable.
5. **Dependencies.** A comes to about eleven direct dependencies, one
   over the target; B and C add a bundler or wazero. Rust's standard
   library lacks HTTP, JSON and cryptography, so D passes ten in the
   core alone. Bun links LGPL-2 JavaScriptCore, off the allow-list.
6. **Reproducible artifact.** A has no JavaScript build in the release
   path and Go builds were measured bit-identical. B adds a pinned npm
   build inside the two-builder check; C adds guest modules from Zig and
   Rust toolchains whose reproducibility is unverified. Rust matched for
   a hello build but not yet with Apple's linker. Bun embeds the output
   name and has open macOS signing bugs.
7. **Run component.** creack/pty covers all three targets (Windows is
   not one, CON-02); `ghostty-vt.wasm` under wasm2go is shown working
   in Go by hauntty. Rust has herdr's native build as a reference. Bun's
   terminal is POSIX only; Kotlin/Native has neither a pty library nor
   a VT model.
8. **Lane view.** An SPA in B to E carries live follow, replay and
   region marks naturally. A pushes HTML over SSE and needs islands for
   the diff and terminal; how far that stretches is the open question.
   Compose for Web draws to a canvas, which makes text selection for
   marks and line links hard.
9. **Terminal parity.** Bubble Tea and ratatui are mature; OpenTUI is
   young and changed under OpenCode within a year.
10. **One view model.** Every option computes it in the core by common
    ground. A renders it on the server, so the page holds none; E's
    shared types make it easy to reimplement in the client by accident.
11. **Git in the core.** Rust's gix is the strongest in-process git.
    Go has go-git and x/crypto for SSH signatures, workable with care
    for the `Stop` budget.
12. **Kernel and MCP.** Starlark exists in Go and Rust. Go's MCP SDKs
    import `net/http`, so A to C write their own stdio server; Rust's
    `rmcp` turns HTTP off with features. JavaScript has no maintained
    Starlark.
13. **P2 reach.** Go has the transports (QUIC, Noise, WireGuard) but no
    cgo-free CRDT; C closes that gap with a CRDT as a WASM module. Rust
    holds iroh, p2panda, Automerge and Loro natively. Every option
    replaces default public relays and discovery (CON-06).
14. **Verification toolchain.** Native fuzzing, the race detector,
    coverage floors and the godog bindings exist in Go today. Rust has
    cargo-fuzz and strong static guarantees, but the gates under
    `internal/` would be ported or kept in Go as black-box drivers.
15. **Cost of the change.** A to C keep CON-01 and every ADR. D
    rewords CON-01, ENG-01 to ENG-09, ENG-16 and supersedes two ADRs.
    E breaks three hard constraints besides.

## What the comparison says

- **E and F leave the field.** E fails HC-3, HC-13 and HC-22 as written
  and costs 95 MB per binary; F is weak in most core slots and strong
  only on the phone, the smallest slot. Their strengths (E's ecosystem,
  F's native phone) survive as parts: a TypeScript page in B to D, and
  a native signer for the phone in any option.
- **A, B and C are one family.** They share the core, run component,
  TUI and binary, and differ only in the page and in how foreign cores
  enter. B is A with a richer page; C is B with shared WASM modules. A
  team can start at B and grow into C when the VT model or a CRDT
  needs it.
- **D is the challenger.** It wins on P2 reach, git and the MCP server,
  and reuses the core on the phone through Tauri. It loses on the
  dependency budget, the verification toolchain and the cost of the
  change. It is worth its cost only if peer sync and CRDTs move ahead
  of the single-developer release.
- **The phone does not pick the language.** No option holds an Ed25519
  key in the iPhone's Secure Enclave, and the phone's job is small
  (OWN-16, OWN-17). A native signer or a wrapped SPA serves every
  option.

## Shortlist

1. **C, entered through B**: Go for every process, a TypeScript page,
   and `ghostty-vt.wasm` as the shared VT model from the start; a CRDT
   module when P2 needs it.
2. **A**: the same, with a server-rendered page, if the page spike
   shows the three panes hold up without an SPA. It keeps npm out of
   the release path.
3. **D**: only if the stakeholder weights P2 above the cost of the
   change.

## Spikes that decide

Each answers a criterion the desk cannot.

| Spike                   | Builds                                                                                                                  | Measures                                     | Decides                       |
| ----------------------- | ----------------------------------------------------------------------------------------------------------------------- | -------------------------------------------- | ----------------------------- |
| One binary, per-entry   | a multi-call Go binary with core, lane view and run entries; the per-entry reach test and its drift case                | criterion 2 and 6; hook p95 and RSS (1)      | whether one binary holds      |
| Page: hypermedia or SPA | the three-pane room twice, Datastar islands and a small SPA, over the same SSE feed                                     | criterion 8, 5 and 6; code and deps per page | A against B                   |
| Shared VT model         | `ghostty-vt.wasm` in the run component via wasm2go and wazero, and in the page; ten hosted harnesses, a late joiner     | criterion 7; keystroke-to-echo p95, RSS      | C's core idea                 |
| Session credential      | the fragment token, page-memory secret and live-stream auth; replay from another loopback port; WebAuthn on `localhost` | HC-5, open points 3 and 12                   | any option; SEC-20 wording    |
| Rust core, if D stays   | the core slot in Rust: store, seal, MCP over stdio, per-crate reach evidence                                            | criteria 2, 5 and 14                         | whether D's cost is justified |

## Decisions for the stakeholder

- **The weight of P2.** If peer sync and co-edited text are next after
  v1, D gains; if they follow the single-developer release by a long
  way, the Go family keeps them in reach through C.
- **Frontend toolchain and ENG-18.** Whether build-only tools and
  vendored page assets count toward the ten (open point 8).
- **The phone at stage two.** A native signer per platform, or the SPA
  wrapped by Tauri or Capacitor; and P-256 device keys or a wrapped
  Ed25519 key (common ground item 10).
- **The running-app preview.** It needs a requirement and a SEC-21
  decision in every option (common ground item 11).
