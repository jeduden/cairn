# Implementation path: the options

Scope: step 1 of answering OQ-32, written 4 October 2026. Each option
is one complete way to build Cairn: it fills every slot of the
[evaluation frame](constraints.md) in the same template, so step 2 can
[compare](comparison.md) them criterion by criterion. The component
facts come from the
[terminal](terminal-components.md),
[protocol and data](protocol-and-data-components.md) and
[UI and packaging](ui-and-packaging-components.md) notes, and the
precedents from the [prior-art note](prior-art-stacks.md). Nothing
here is decided; a choice lands as an ADR and an SRS change.

## How the options were cut

The options differ on one question: which language holds which slot,
and how the result becomes one binary. Everything that does not depend
on that question is held equal across them, in
[common ground](#common-ground), so a difference in step 2 traces to
the option and not to an incidental choice.

The stakeholder's candidates map onto the frame like this:

- **TypeScript, Rust, Go, Zig and WebAssembly** are languages for the
  slots. Each appears as the main language of an option, or, for Zig
  and WebAssembly, as the way foreign cores enter a Go binary.
- **Kotlin Multiplatform** is an option of its own.
- **A VS Code plugin, Electron and extending Ghostty** are shells
  around a surface, not ways to build the slots. No terminal or editor
  is multiplayer, and decision 9 of the [pitch][pitch] puts the
  standalone UI in the browser. They appear under
  [shells](#shells-and-hosts-any-option-can-add) as optional additions
  any option can carry, with what each costs.
- **herdr and tmux** are hosts the run component can follow, also
  under shells and hosts.

## Common ground

Every option takes these positions. They answer the frame's
[open points](constraints.md#open-points) that do not depend on the
language.

1. **Packaging.** One multi-call executable per target. The first
   argument picks exactly one component before any component code runs
   (HC-2). Each process runs one component; the core never starts
   another (SEC-01, X8). The reach evidence is per entry point, taken
   from the shipped artifact (HC-3).
2. **Boundaries unchanged.** Core in B0; lane view and run component in
   B1 on loopback; peer and publish components P2 (SEC-19).
3. **The browser page is the main surface** (decision 9), served by
   `cairn ui` on loopback. `cairn ui` prints its URL by default; it does
   not start a browser (open point 2, ENG-16).
4. **Session credential.** A per-launch token in the URL fragment,
   exchanged for a session secret held in page memory and sent as a
   header or as the first message of the live stream, never a cookie,
   since cookies cross ports (open point 3, HC-5).
5. **One view model.** Statuses, the Needs you order and the vocabulary
   are computed once in the core and sent to every client as data
   (VIEW-04, VIEW-05, VIEW-14; open point 9).
6. **Harness hosting by stdio protocol.** The run component drives
   Claude Code by stream-json, ACP agents and the Codex app-server as
   child processes speaking JSON over stdio. None of the options embeds
   the Claude Agent SDK as a library: its Commercial Terms are off the
   ENG-18 allow-list (open point 10).
7. **A pty as fallback, with a headless VT model** for late joiners and
   takeover, owned by the run component; owner input only (OWN-19).
8. **Storage.** SQLite with FTS5 in every option; all five languages
   reach it (open point 16).
9. **Phone, stage one.** The same page through the owner's tunnel,
   installed as a PWA, with no vendor push service (VIEW-17).
10. **Device keys on iPhone.** The Secure Enclave holds P-256, not
    Ed25519. Every option either wraps a software Ed25519 key with an
    enclave key or asks for an SRS change admitting P-256 device keys
    (open point 13). The language does not change this.
11. **Running-app preview.** The app opens on its own loopback origin,
    in a separate tab or a sandboxed frame, never inside the lane
    view's origin; a mark on it is a snapshot the run component takes
    (open points 4 and 5). This needs a requirement and a SEC-21
    decision in any option.

## The options at a glance

| Slot                  | A. Go, hypermedia                       | B. Go and a TS page                   | C. Go host, WASM cores                              | D. Rust and Tauri                       | E. TypeScript on Bun              | F. Kotlin Multiplatform                  |
| --------------------- | --------------------------------------- | ------------------------------------- | --------------------------------------------------- | --------------------------------------- | --------------------------------- | ---------------------------------------- |
| Core (B0)             | Go                                      | Go                                    | Go                                                  | Rust                                    | TypeScript on Bun                 | Kotlin/Native                            |
| Lane-view server (B1) | Go, server-rendered HTML over SSE       | Go, JSON events over SSE or WebSocket | Go, JSON events over SSE or WebSocket               | Rust (hyper or axum)                    | Bun's server                      | Kotlin/Native (Ktor)                     |
| Page                  | `html/template` + Datastar, two islands | TS SPA (Preact or Solid)              | TS SPA plus shared WASM modules                     | TS SPA                                  | TS SPA                            | Compose for Web (Wasm)                   |
| Run component         | Go pty; ghostty-vt.wasm via wasm2go     | as A                                  | as A, the same `.wasm` in the page                  | portable-pty; libghostty-vt native      | `Bun.Terminal`; `@xterm/headless` | POSIX pty by interop; VT model to write  |
| Terminal pane         | ghostty-web island                      | ghostty-web                           | own renderer on upstream ghostty-vt.wasm            | ghostty-web                             | xterm.js                          | own Compose canvas                       |
| Diff pane             | CodeMirror 6 merge island               | CodeMirror 6 merge                    | CodeMirror 6 merge                                  | CodeMirror 6 merge                      | CodeMirror 6 merge                | own Compose view                         |
| TUI                   | Bubble Tea                              | Bubble Tea                            | Bubble Tea                                          | ratatui                                 | OpenTUI or Ink                    | Mosaic or own                            |
| Phone, stage two      | small native Swift and Kotlin signer    | Tauri 2 mobile wrapping the SPA       | Capacitor or Tauri wrapping the SPA, WASM seal code | Tauri 2 mobile sharing the Rust crates  | Capacitor sharing the TS core     | Compose on iOS and Android, shared core  |
| Peer (P2)             | Go transports, own sealed-range sync    | as A                                  | Go transports; CRDT as WASM (Loro or Automerge)     | iroh or p2panda, own relays             | Hypercore or js-libp2p            | iroh-ffi (MPL-2.0) or own                |
| One binary            | Go multi-call + `embed.FS`              | Go multi-call + `embed.FS` of the SPA | Go multi-call + `embed.FS` + embedded `.wasm`       | Rust multi-call + `include_dir`         | `bun build --compile`             | Kotlin/Native executable + embedded Wasm |
| Size, linux/amd64     | about 6 MB (measured shape)             | about 6 MB                            | about 7 to 10 MB                                    | unmeasured, likely 10 to 20 MB          | about 95 MB (measured)            | unmeasured                               |
| Reach evidence        | import closure and call graph per entry | as A                                  | as A, plus the WASM imports each module may call    | one crate per component, lints, symbols | runtime permissions only          | per-module klib deps; tooling unproven   |
| Changes to the SRS    | none to the language rows               | none to the language rows             | none to the language rows                           | CON-01, ENG-01 to 09, ENG-16, ADRs      | as D, and CON-03, HC-22 regex     | as D, and Compose web maturity           |

## Option A: Go, hypermedia

**Thesis.** Keep Go for every slot and keep the page server-rendered,
so the release path has no JavaScript toolchain at all.

- **Core.** Go as CON-01 and the ADRs assume: ncruces/go-sqlite3,
  `crypto/ed25519` and `crypto/sha256` from the standard library, a
  hand-written stdio MCP server (the Go MCP SDKs import `net/http`),
  Starlark in Go for the kernel.
- **Lane view.** `html/template` fragments pushed over one SSE stream
  with Datastar (one vendored 33.5 KB file, CSP mode on). The two panes
  that need client code, the diff and the terminal, are web-component
  islands: CodeMirror 6's merge view and ghostty-web, vendored as
  prebuilt files.
- **Run component.** creack/pty; upstream `ghostty-vt.wasm`, pinned by
  hash and translated to Go by wasm2go, as the headless VT model: no
  runtime dependency, the go-sqlite3 precedent.
- **TUI.** Bubble Tea and Lip Gloss, reading the store directly.
- **Phone, stage two.** A small native app per platform that signs
  allow and deny events, reimplementing canonical JSON, SHA-256 and the
  seal against shared test vectors (HC-10).
- **Peer.** quic-go or Noise for transport; Cairn's own sealed-range
  exchange of writer logs (PEER-03). A CRDT for co-edited text has no
  cgo-free Go library and would be written or deferred.
- **One binary.** `CGO_ENABLED=0` multi-call executable, page assets in
  `embed.FS`; the measured shape is 6.2 MB with assets, 1.6 MB core.
  Bit-identical builds were measured.
- **Evidence.** Today's import-closure test, made per entry point,
  plus a call graph from each entry; `net/http` code is in the file but
  unreachable from the core's entry (HC-1, HC-3).
- **Direct dependencies.** sqlite, Starlark, pty, Bubble Tea, Lip
  Gloss, the three test modules, Datastar, CodeMirror merge, ghostty-web
  and the ghostty-vt module: eleven, one over the target.
- **Precedent.** Crush and Claude Squad ship Go with Bubble Tea as
  static binaries of 2 to 27 MB; no surveyed agent UI renders its
  page on the server ([prior art][prior]).
- **Biggest risk.** A rich, live three-pane page in server-rendered
  HTML: follow-an-agent, region marks and replay may push more code
  into islands until it is an SPA in all but name.

## Option B: Go and a TypeScript page

**Thesis.** Go for every process; the page is a small TypeScript SPA,
built once inside the release pipeline and embedded.

- **Core, run component, TUI, peer.** As option A.
- **Lane view.** A Go server that sends the core's view model as JSON
  events over SSE or a WebSocket. The page is a Preact or Solid SPA
  (about 5 KB of framework) with CodeMirror 6 merge and ghostty-web,
  under a CSP with no inline script and no `eval`.
- **Phone, stage two.** Tauri 2 mobile or Capacitor wrapping the same
  SPA, with the device key in the platform key store; a second language
  (Rust) or a store-packaged app enters only here.
- **One binary.** As A, with the SPA's `dist` in `embed.FS`. The JS
  build is pinned and runs inside the two-builder comparison (HC-12).
- **Evidence.** As A.
- **Direct dependencies.** As A with the framework and a bundler in
  place of Datastar; whether build-only tools count toward the ten is
  open point 8.
- **Precedent.** The most common shape in the survey: a compiled core
  serving an embedded web UI, as Vibe Kanban (Rust and React), Crush's
  Go server and the Codex app-server do ([prior art][prior]).
- **Biggest risk.** A second toolchain (npm) in the release path, and
  the view model drifting into client code instead of staying in the
  core.

## Option C: Go host, shared WASM cores

**Thesis.** Go for every process, and the hard algorithms that Go
lacks or that must agree across surfaces enter as WebAssembly modules,
run inside the Go binary and, unchanged, in the browser and the phone.
This is where Zig and Rust come in without becoming the core's
language.

- **Core, TUI.** As option A.
- **Shared modules.** `ghostty-vt.wasm` (Zig) as the VT model in the
  run component and the page; later a CRDT such as Loro or Automerge
  (Rust) for co-edited text; possibly the seal verifier, so the page
  and phone check seals with the core's own code.
- **Run component.** As A, with the same module the page renders from,
  so server and browser cannot disagree on the screen.
- **Lane view and page.** As B, with the page loading the shared
  modules; the terminal pane is a small renderer on upstream
  `ghostty-vt.wasm` instead of ghostty-web's patched build.
- **Phone, stage two.** Capacitor or Tauri wrapping the SPA, with seal
  code from the shared module.
- **Peer.** Go transports; the CRDT as a WASM module, which closes
  Go's weakest slot in the [protocol matrix][matrix].
- **One binary.** As B, plus each `.wasm` pinned by hash and either
  translated by wasm2go or run by wazero (one Apache-2.0 dependency,
  SIMD kept); measured at 5.4 MB with ghostty-vt.wasm under wazero.
- **Evidence.** As A, plus each module's import list, which is the
  whole of what it can call: a module given no socket import has none.
- **Direct dependencies.** As B, plus wazero or the wasm2go tool, plus
  each guest module.
- **Precedent.** herdr links libghostty-vt natively and Warp ships a
  WASM web terminal; go-sqlite3 already runs a C library inside Go by
  WASM translation, the route the SQLite ADR takes
  ([prior art][prior], [terminal note](terminal-components.md)).
- **Biggest risk.** Guest modules are prebuilt inputs from other
  toolchains (Zig, Rust); reproducing them bit for bit is unverified,
  and libghostty-vt's API is not yet stable.

## Option D: Rust and Tauri

**Thesis.** Rewrite the plan in Rust: the strongest peer and CRDT
ecosystem, and the same crates on the desktop, the server and the
phone.

- **Core.** rusqlite with FTS5 bundled, `rmcp` with HTTP features off,
  `ed25519-dalek`, `sha2`, `serde_json_canonicalizer`, the `regex`
  crate for RE2-compatible redaction, Starlark in Rust for the kernel.
- **Lane view.** hyper or axum on loopback; the page is a TS SPA as in
  B.
- **Run component.** portable-pty and libghostty-vt linked natively, as
  herdr builds it (Apache-2.0 since 0.8.0, a usable reference).
- **TUI.** ratatui.
- **Phone, stage two.** Tauri 2 mobile, sharing the core's crates for
  canonical encoding and seals.
- **Peer.** iroh, p2panda or Willow, with Cairn's own relays and
  discovery in place of the default public ones (CON-06).
- **One binary.** A Rust multi-call executable with `include_dir`;
  static on musl for Linux. An optional desktop window is a separate
  Tauri artifact.
- **Evidence.** One crate per component, clippy `disallowed-types` and
  `disallowed-methods` on `std::net` and `std::process`, and a symbol
  check of the build (HC-1, HC-3).
- **Direct dependencies.** Rust's standard library has no HTTP, JSON or
  cryptography, so the core alone is near the target of ten.
- **Precedent.** Vibe Kanban, Codex, Goose, herdr and Block Buzz are
  Rust; Block Buzz pairs Tauri 2 with rooms on its own relay. opcode
  never published release builds, and OpenCode left Tauri for Electron
  ([prior art][prior]).
- **Biggest risk.** The cost of the change: CON-01, ENG-01 to ENG-09,
  ENG-16, two ADRs, the gates and the scenario bindings, and fewer
  maintainers fluent in Rust (ENG-25).

## Option E: TypeScript on Bun

**Thesis.** One language from core to page, with the richest
ecosystem for harnesses, CRDTs and UI.

- **Core.** `bun:sqlite`, WebCrypto, the reference MCP SDK, the
  `canonicalize` package; Starlark has no maintained JavaScript
  implementation, so the kernel runs another language or a WASM build.
- **Lane view and page.** Bun's server and a TS SPA, sharing types and
  the view model code with the server.
- **Run component.** `Bun.Terminal` and `@xterm/headless`, the same
  emulator as the page's xterm.js (POSIX only).
- **TUI.** OpenTUI or Ink.
- **Phone, stage two.** Capacitor, sharing the TS core.
- **Peer.** Hypercore, Autobase or js-libp2p.
- **One binary.** `bun build --compile`: 94.6 MB on linux-x64, 63.4 MB
  on darwin-arm64 (measured), with JavaScriptCore statically linked.
- **Evidence.** `fetch`, `Bun.listen` and `Bun.spawn` are globals, so
  no import graph shows a component's reach; the proof falls back to a
  runtime sandbox, which HC-3 does not accept as build-time evidence.
- **Precedent.** OpenCode, Claude Code, Amp and Cline's CLI ship Bun
  executables of 134 to 246 MB; OpenCode rewrote its TUI and changed
  desktop shells within a year ([prior art][prior]).
- **Biggest risk.** It fails three hard constraints as written: HC-3
  (build-time evidence), HC-13 (LGPL-2 JavaScriptCore is off the
  allow-list) and HC-22 (redaction needs RE2 semantics; JavaScript's
  engine backtracks). CON-03 needs a ruling on an embedded runtime.

## Option F: Kotlin Multiplatform

**Thesis.** One Kotlin codebase, with native Compose apps on desktop,
Android and iOS and the best Android key-store story.

- **Core.** Kotlin/Native with SQLDelight's native SQLite driver;
  canonical JSON and Starlark have no maintained Kotlin/Native
  libraries (the Java ones need the JVM).
- **Lane view and page.** Ktor on Kotlin/Native; the page in Compose
  for Web on WebAssembly.
- **Run component.** A POSIX pty through C interop; a VT model written
  or taken from a JVM library that does not reach Kotlin/Native.
- **TUI.** Mosaic or hand-written.
- **Phone, stage two.** Compose on iOS and Android, sharing the core.
- **Peer.** iroh-ffi's Kotlin bindings (MPL-2.0, off the allow-list) or
  Cairn's own.
- **One binary.** A Kotlin/Native executable with the Wasm page
  embedded; static linking and reproducibility are unmeasured.
- **Evidence.** Per-module dependencies; no existing tool derives
  reach per entry point.
- **Precedent.** None of the 27 surveyed products builds on Kotlin
  Multiplatform; Nimbalyst uses Kotlin only for its Android app
  ([prior art][prior]).
- **Biggest risk.** Most core slots are weak or missing on Kotlin/Native
  ([matrix]); the strength is the phone, which is the smallest slot
  (OWN-16, OWN-17).

## Shells and hosts any option can add

These wrap a surface or follow a host; none fills a slot on its own.
Each is optional and each is a second artifact or a second process.

| Addition                       | What it gives                                       | What it costs                                                                                                   | Verdict                                      |
| ------------------------------ | --------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | -------------------------------------------- |
| VS Code webview extension      | the room beside the code, in the editor people use  | a second signed artifact; `asExternalUri` tunnels the loopback page off a remote machine, against SEC-01 and I4 | later, view only, local editors only         |
| VS Code layout takeover        | an IDE-shaped Cairn                                 | fights the host; the agent often runs inside the same extension host (T21); breaks "your setup stays"           | no                                           |
| Electron desktop app           | a window, a tray, notifications                     | not one binary; 120 MB or more; CON-03 bars Node.js                                                             | no; Tauri if a window is ever needed         |
| Tauri desktop window           | a window pointed at the loopback page               | a second artifact; Rust                                                                                         | optional, after v1                           |
| Ghostty fork or plugin         | Cairn inside the terminal people use                | no plugin API; a Zig fork to maintain; single-player                                                            | no; reuse libghostty-vt instead (A to D)     |
| herdr or tmux as followed host | agents the run component did not start, in the room | their sockets let any same-user process type and read (T21), so input there is never the owner's                | follow-only adapter, input shown unavailable |
| JetBrains JCEF panel           | the room inside JetBrains IDEs                      | a second artifact; loopback loading unverified                                                                  | later                                        |
| Zed                            | recall inside Zed                                   | no webviews, so no room                                                                                         | MCP server only                              |

## Options left out

- **Zig for the whole core.** No MCP, ACP, canonical JSON or peer
  libraries, and the language is pre-1.0 ([matrix]). Zig enters through
  option C as the source of `ghostty-vt.wasm`.
- **Node or Deno compiled executables.** Bun dominates them on size
  and start-up for option E's shape; Node also falls under CON-03.
- **A Wails desktop app.** Its webview needs cgo, against CON-02 for a
  binary that also carries the core, and it is still in beta.

[pitch]: ../../../plan/2610012322_cairn-for-agent-fleets/pitch.md
[matrix]: protocol-and-data-components.md#matrix
[prior]: prior-art-stacks.md#patterns
