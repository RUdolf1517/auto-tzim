# Dependencies

> **Status: specification, no implementation.** Nothing is built yet; this page records what the MVP intends to depend on
> and what it refuses to depend on.
> Related: [architecture](architecture.md) · [privacy](privacy.md) · [releases](releases.md) · [THIRD-PARTY](../THIRD-PARTY.md)

The rule: **the standard library first, and a new crate needs a sentence explaining what it does that `std` does not.** A
dependency that phones home, runs code at build time on the user's machine, or drags in an async runtime is refused
regardless of convenience.

The stack was changed from Python to Rust on 2026-11-01 ([D-33](manager/DECISIONS.md), [CONFLICTS C-5](spec/CONFLICTS.md)).
The counting rule that comes with that: a Rust dependency is one line in `Cargo.lock` on disk, while a Python dependency
could arrive through the environment — which is the property this page is built on now.

## 1. Runtime — the shipped binary

| Crate | Why | What `std` cannot do | Refused alternative |
| --- | --- | --- | --- |
| `clap` (derive) | argv parsing, help text, `--` handling | no argument parser in `std`; hand-rolling one is how a flag like `--json` becomes a filename | a hand-written parser; a shell wrapper |
| `serde` + `toml` | the documented TOML config, strict schema, unknown fields rejected | no TOML reader in `std` | a hand-written TOML parser — a correctness risk in the one file that decides what may be touched |
| `serde_json` | the documented NDJSON output | no JSON in `std`; string-building JSON is how output becomes unparseable | manual `format!` construction |
| `sha2` | binary identity, record hashes | no SHA-256 in `std` | shelling out to `shasum`/`sha256sum` — a shell dependency and a locale hazard |
| `thiserror` | typed errors that map to exit codes | `std` gives no derive; without typed errors the exit-code contract becomes string matching | `anyhow` everywhere (an opaque error is not a code) |

That is the entire runtime set: **five** crates, all of them boring, none of them capable of opening a socket — and **not one
line of C**. State and history are plain files written with `std` alone (D-35), so the build has no C toolchain in it at
all, which is also what makes a static musl binary reproducible on a machine that has nothing installed.

**Not a runtime dependency, deliberately invoked:** `tzim`. It is a separate MIT-licensed executable the user installs; we
run it as a child process and never link, vendor or import it. That process boundary is the licence boundary.

**Refused outright:** any async runtime (`tokio`, `async-std` — the runner is sequential, and a second execution model is a
second set of failure modes to test); any HTTP crate (`reqwest`, `ureq`, `hyper`, `curl`) while v0.1 makes no request;
**any crate that compiles C** (`rusqlite`, `libsqlite3-sys`, `openssl`, `ring`'s C build, `zlib`) — a database engine was
removed for this reason alone (D-35); any crate that shells out or executes downloaded content; any crate whose build
script downloads something at build time.

**How these refusals are enforced, not just stated:** `cargo deny` fails the build on a licence outside the allow-list, a
duplicate crate, a banned crate name, or a source outside crates.io; `cargo tree` in CI asserts that no crate in the graph
can open a socket; `Cargo.lock` is committed and reviewed as part of the pull request, so an added dependency is a visible
line rather than an accident of installation.

## 2. Build and test only

| Crate | Why | Notes |
| --- | --- | --- |
| `assert_cmd` + `predicates` | black-box CLI tests: spawn the binary, assert on stdout, stderr and the exit code | the exit-code contract is tested the way a user sees it, not through an internal function |
| `tempfile` | isolated fixture directories per case | every case gets its own tree; no test writes into the repository |
| `serde_json` (dev) | parsing our own NDJSON back, so a test asserts a schema rather than a substring | a test that greps stdout proves nothing about JSON |
| `insta` (optional) | snapshots of the proposal text and the human output | optional: a snapshot that nobody reads is a rubber stamp |

`cargo test` is the runner; `cargo fmt --check`, `clippy -D warnings` and `cargo deny check` are CI gates. Nothing in this
list ships in the binary.

## 2a. How a dependency gets admitted

Four mechanisms, and the review is the one that matters ([D-38](manager/DECISIONS.md)):

1. **exact pins** — every declared version is `=X.Y.Z`; a range fails a CI job (`scripts/check-pins.py`, or whatever
   replaces it), because a range resolves to something nobody chose;
2. **deny-by-default allowlist** in `deny.toml` — a crate that is not listed fails the build, transitive ones included;
3. **explicit deny list** — `cc`, `rusqlite`, `libsqlite3-sys`, `openssl-sys`, `libz-sys` and the network families, with a
   message that says why;
4. **review** — adding the allowlist entry is the human step, and it is the only thing that can tell whether a crate
   compiles C or opens a socket. `cargo-deny` reads manifests and licences; it does not read behaviour.

Mechanisms 1 and 2 are checked by `scripts/check-pins.py` and `deny.toml`. The script is Python because it runs only in
<<<<<<< HEAD
CI and reads manifests ([D-41](manager/DECISIONS.md)); it has a self-test (`scripts/test-check-pins.py`) that builds two fixture manifests and asserts the checker
rejects `*` and accepts `=1.2.3` — proven live by breaking the checker on purpose and watching the test go red
([review](manager/reviews/REVIEW-017ae80.md) §4). It and
=======
CI and reads manifests ([D-41](manager/DECISIONS.md)); its self-test runs in CI ([D-41](manager/DECISIONS.md)). It and
>>>>>>> origin/main
the allowlist are the two places a dependency is *judged* — nothing else in the tree inspects a crate.

So a new dependency is deliberately **two commits**: the dependency, then the allowlist entry. A dependency change should
also carry `cargo tree -e features`, because enabling a feature on an already-allowed crate is the one hole the allowlist
does not see.

**The harness is part of the workspace**, not an external tool ([testing-harness](testing-harness.md)): the fake `tzim`,
the case runner and the pinned-binary fetcher are one `auto-tzim-harness` binary, so `run-case <ID>` is a command anyone can
reproduce without installing a language runtime.

## 3. Later stages (not now)

| Crate | Stage | Why then |
| --- | --- | --- |
| `ratatui` + `crossterm` | v0.2 | the TUI, as a **separate binary** (`auto-tzim-tui`), never a mode of the CLI (D-34) |
| `cxx-qt` | v0.4 | the Qt app, as a **separate binary and its own package** (`auto-tzim-qt`), linking `auto-tzim-core` (D-34) |

Both stay outside the MVP on purpose: a front door that becomes a mandatory dependency of the CLI is a regression in the
only thing this tool promises — being a small, auditable orchestrator.

The Qt artifact is the one place where Rust costs something visible: it needs a Qt runtime and a packaging story that
`PySide6` would have given for free. D-34 answers that by separating the artifact rather than by pretending the cost is
small.

## 4. Pinning and verification

- `Cargo.lock` is committed; a build without it (`--locked` in CI) is a failed build.
- The toolchain is pinned in `rust-toolchain.toml`; a toolchain bump is a minor release and is recorded in the changelog,
  because it can change code generation.
- `cargo deny` enforces the licence allow-list, the crate ban list and the registry.
- A dependency addition is a pull-request line in `Cargo.lock` plus an entry in this table; a crate that is not in the
  table is not accepted, even if the code compiles.

## 5. What would make us add a dependency

A dependency is accepted when all four hold: `std` or an existing crate cannot do it without a correctness risk; it is
maintained; its licence is on the allow-list; and it can be explained in one sentence on this page. Convenience is not on
the list, and neither is "it saved twenty lines".

## 6. What there is no longer

`tomli-w`, `pytest`, `pytest-timeout`, `ruff`, `mypy`/`pyright` and `urllib.request` left with the Python stack (D-33), and
`urllib` had already left with D-28 when the MVP gave up its network request. Their replacements are in the tables above;
the requirement they existed for — a strict config, a real test runner, a linter, a licence check — is unchanged.
