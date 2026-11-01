# Third-party notices

What the shipped binary links, and under what licence. The reasoning — why each crate exists and what it replaces — is in
[dependencies](docs/dependencies.md); this page is the flat list a distribution packager needs.

**Status: nothing is built yet.** The table is the intended set, fixed before the code so that the first release does not
add a dependency quietly. Exact versions and checksums are recorded in `Cargo.lock` when the workspace is created (AT-01).

## Runtime dependencies

| Crate | Used for | Licence | Ships a `NOTICE`? |
| --- | --- | --- | --- |
| `clap` | argv parsing, help text | MIT OR Apache-2.0 | no |
| `serde` | typed config and JSON output | MIT OR Apache-2.0 | no |
| `toml` | reading the TOML config | MIT OR Apache-2.0 | no |
| `serde_json` | the documented NDJSON output | MIT OR Apache-2.0 | no |
| `rusqlite` (feature `bundled`) | state and history; compiles SQLite into the binary | MIT | no |
| `sha2` | binary identity, record hashes | MIT OR Apache-2.0 | no |
| `thiserror` | typed errors that map to exit codes | MIT OR Apache-2.0 | no |

The bundled SQLite is **public domain** — the one entry here that is not a permissive licence but no licence at all, which
is worth stating plainly because packagers ask about it.

## External program, not a dependency

`auto-tzim` **runs** [`tzim`](https://github.com/RUdolf1517/tzim); it does not import, vendor or link it. `tzim` is a
separate MIT-licensed executable installed by the user, with its own
[THIRD-PARTY.md](https://github.com/RUdolf1517/tzim/blob/main/THIRD-PARTY.md) covering its crates. A user who installs
`auto-tzim` acquires nothing from that tree; a user who already has `tzim` keeps it where it is.

This is deliberate and it is the licence boundary: because the two programs meet at a process boundary, neither project's
dependency tree becomes the other's.

## Build and test only

| Crate | Used for | Licence | Notes |
| --- | --- | --- | --- |
| `assert_cmd` | spawning the CLI in black-box tests | MIT OR Apache-2.0 | not in the shipped binary |
| `predicates` | assertions on stdout/stderr | MIT OR Apache-2.0 | — |
| `tempfile` | isolated test trees | MIT OR Apache-2.0 | — |
| `insta` (optional) | snapshots of human output and the proposal | Apache-2.0 | optional |

## Transitive dependencies

`clap` brings `clap_builder` and `clap_derive`; `serde` brings `serde_derive`; `rusqlite` brings `libsqlite3-sys` (the
bundled C build) and `hashlink`; `toml` brings `toml_edit` and `winnow`; `serde_json` brings `itoa`, `ryu` and `memchr`;
`sha2` brings `digest`, `generic-array`, `typenum`, `block-buffer` and `crypto-common`. All are MIT, Apache-2.0, Zlib or
0BSD; `cargo deny` enforces the allow-list in CI, and the exact versions and checksums live in `Cargo.lock`.

## This project

`auto-tzim` is [MIT](LICENSE).

## A note for anyone packaging this

The binary is statically linked against musl on Linux and needs no runtime dependency beyond the C library contract of the
target. The Qt front door (v0.4) is a **separate artifact** with its own dependency on the Qt runtime (D-34) — a packager
who ships the CLI does not inherit Qt, which is the reason the split exists.
