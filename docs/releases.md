# Releases

> **Status: specification, no implementation.** There is no release, no tag, no artifact and no package. This page
> fixes the rules before the first one exists.
> Related: [roadmap](roadmap.md) · [update](update.md) · [privacy](privacy.md)

## 1. Versioning

Semantic versioning with the pre-1.0 allowance: while the major is `0`, a minor release may add commands, options and
fields; a patch release only fixes defects.

What counts as a **major** change here is the contract, not the command line: changing a documented status name,
removing a documented exit code, changing the meaning of a config field, weakening a safety invariant, or narrowing
what a profile may rely on. Adding a command or a config field is minor. Making the program refuse *more* — a new
validation rule, a narrower default — is minor only if the reason is documented and the previous behaviour was unsafe;
otherwise it is a major change, because someone's automation may depend on it.

## 2. What a release will contain

| Artifact | Channel | Notes |
| --- | --- | --- |
| `auto-tzim` binary, static musl (Linux x86_64, aarch64) and macOS (x86_64, aarch64) | GitHub releases | one file per target plus a `.tar.gz`; no runtime, no interpreter, nothing to install alongside it |
| `auto-tzim-tui` binary (v0.2) | GitHub releases, separate artifact | optional; a user who wants the CLI never downloads it (D-34) |
| `auto-tzim-qt` package (v0.4) | GitHub releases, **separate package** | its own dependency on the Qt runtime; a packager of the CLI does not inherit Qt (D-34) |
| `auto-tzim` crate on **crates.io** | `cargo install auto-tzim` | built from source on the user's machine; not byte-identical to a release artifact (different compiler), and verified afterwards by the same checks (D-36) |
| `SHA256SUMS` | GitHub releases | one line per artifact; every published file is listed |
| source archives | GitHub | generated from the tag; `Cargo.lock` is committed, so a rebuild is reproducible |
| Homebrew formula | tap, from the first usable release | must be updated for **every** usable release, and verified by a real upgrade test |
| container image | not planned | a tool that rewrites user files in place in a container is a category error |

Release notes state: the version, the supported `tzim` versions, the platforms actually tested, any change to defaults,
and any change to the network surface.

**Publication order is mandatory.** `auto-tzim` depends on `auto-tzim-core` by version, so the core is published
**first** and the CLI second. This is not a preference: before the core exists on the registry, `cargo package -p
auto-tzim` fails with `no matching package named auto-tzim-core found` ([D-39](manager/DECISIONS.md)), which is why the
pre-publication verification below differs from the release-time one.

**Toolchain and MSRV.** The crate declares `rust-version` in both manifests, and it equals the channel pinned in
`rust-toolchain.toml` at release time — the version CI actually runs, never a guess about what might work. A `cargo install` on a toolchain below it fails in the compiler rather than in the
product — which is the honest failure, and it is written in the README next to the command. Two crates are published:
`auto-tzim-core` (the library) and `auto-tzim` (the CLI, producing the `auto-tzim` binary). The names keep their hyphen
(D-36). Tags and artifacts are immutable: a bad artifact is superseded, never silently
rebuilt under the same tag.

**How packaging is verified before and after publication.** The difference is deliberate and it is recorded rather than
smoothed over:

| When | Command | What it proves |
| --- | --- | --- |
| any time (core) | `cargo package -p auto-tzim-core` | the tarball builds on its own — the core is a leaf crate |
| before core is published (CLI) | `cargo package --no-verify -p auto-tzim` **plus** a grep of the normalised manifest in the `.crate` | the tarball is produced and its manifest is registry-correct: `auto-tzim-core` present, `path` removed. It does **not** prove the tarball builds |
| before core is published (guard) | run `cargo package -p auto-tzim` and assert it fails with the exact message | the failure is *expected*, so a change in it is noticed instead of being confused with a regression |
| after core is published (CLI) | `cargo package -p auto-tzim` | full verification: the tarball builds against the registry version of the core |

The guard row is the one that usually rots. A permanently red step is ignored within a week; a step that asserts the red
is *this* red stays honest, and it flips on the day the core is published — which is the day someone must look at it.

## 3. Definition of done for a release

1. Every acceptance criterion of the stage is met at the final head SHA, with raw evidence archived.
2. Required CI is green on macOS and Linux; no required job is skipped or replaced by a placeholder.
3. `README`, `--help`, the config reference and the safety documents match the binary — checked by test, not by
   reading.
4. No unresolved Critical/High finding; a Medium finding is listed, not hidden.
5. SHA-256 values are published and match the artifacts.
6. If a Homebrew formula exists, it is updated and the real upgrade from the previous installed version was tested.
7. `CHANGELOG.md` is written and does not rewrite earlier entries.
8. The privacy section matches the code's actual network surface, including the absence of any automatic check.

## 4. Who does what

The manager reviews and recommends; the owner merges and publishes. A release is not created by whoever wrote the
code, and a release is never created from a branch that has not been reviewed at the SHA being tagged (charter §14).
`auto-tzim` does not publish itself, and no command in it can produce a release artifact.

## 5. Compatibility with `tzim`

| `auto-tzim` | Tested `tzim` versions | Adapter |
| --- | --- | --- |
| not released | none yet | — |

The table is empty on purpose. Before the first release it will name exact `tzim` versions, not ranges that were never
run, and the adapter table in code will match it.
