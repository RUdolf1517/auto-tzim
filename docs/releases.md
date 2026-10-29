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
| Python source archive + wheel | PyPI and GitHub releases | the package is pure Python; `python -m auto_tzim` and the `auto-tzim` entry point are equivalent |
| `SHA256SUMS` | GitHub releases | one line per artifact; every published file is listed |
| source archives | GitHub | generated from the tag |
| Homebrew formula | tap, from the first usable release | must be updated for **every** usable release, and verified by a real upgrade test |
| container image | not planned | a tool that rewrites user files in place in a container is a category error |

Release notes state: the version, the supported `tzim` versions, the platforms actually tested, any change to defaults,
and any change to the network surface. Tags and artifacts are immutable: a bad artifact is superseded, never silently
rebuilt under the same tag.

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
