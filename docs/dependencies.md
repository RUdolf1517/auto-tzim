# Dependencies

> **Status: specification, no implementation.** Nothing is installed yet; this page records what the MVP intends to
> depend on and what it refuses to depend on.
> Related: [architecture](architecture.md) · [privacy](privacy.md) · [releases](releases.md)

The rule: the standard library first, and a new dependency needs a sentence explaining what it does that the standard
library does not. A dependency that phones home, runs install-time code, or drags in a build toolchain is refused
regardless of convenience.

## 1. Runtime

| Dependency | Why | Alternative considered | Refused because |
| --- | --- | --- | --- |
| `tomllib` (stdlib, 3.11+) | reads TOML | `toml`, `tomlkit` | stdlib is available and read-only is all we need for reading |
| `tomli-w` (proposal D-15, unapproved) | writes TOML; `tomllib` cannot | hand-rolled writer | A hand-written TOML serialiser is a correctness risk in the one file that defines what may be touched; `tomli-w` is small, pure Python, no dependencies, no install hook. Alternative: write JSON — refused, because the specification fixes TOML |
| `sqlite3` (stdlib) | state and history | JSON files | transactions, a real schema, and crash recovery are the point of the state layer |
| `urllib.request` (stdlib) | the single `update-check` request | `requests`, `httpx` | the request is one GET with a timeout; a dependency for it is not justified |
| `subprocess`, `signal`, `os`, `stat`, `fnmatch`, `hashlib`, `json`, `logging` (stdlib) | everything else | — | — |

No codec, no image library, no archive library, no `shell=True` wrapper: those belong to `tzim`.

## 2. Development only

| Dependency | Why | Notes |
| --- | --- | --- |
| `pytest` | the test runner | pinned with a hash |
| `pytest-timeout` | fails a hanging test instead of hanging CI | optional; the value is that the suite proves timeouts rather than inheriting the runner's |
| `ruff` | linting and formatting | configuration lives in the repository, not in a comment |
| `mypy` (or `pyright`) | typing | one of them, not both, and only if the annotations are load-bearing |

None of these is imported by the released package.

## 3. Later stages (not now)

| Dependency | Stage | Why then |
| --- | --- | --- |
| `textual` | v0.2 | TUI; optional extra, never required for the CLI |
| `PySide6` | v0.4 | Qt GUI; optional extra, with its own packaging story (signing, notarisation, entitlements) |

Both are deliberately outside the MVP: an optional GUI that becomes a mandatory dependency of the CLI is a regression
in the only thing this tool promises — being a small, auditable orchestrator.

## 4. Pinning and verification

- Runtime dependencies are pinned to exact versions with hashes in a requirements file; a CI job fails when an
  unpinned dependency appears (the `tzim` repository uses the same pattern with a `lockfile` job).
- Test-only dependencies are pinned in a separate file so that a test update cannot silently change the shipped
  package.
- Every dependency is listed in a third-party notice with its licence when the first release happens.
- A dependency update is a reviewed change with a changelog entry; a silent `pip install -U` in CI is not allowed.

## 5. What would make us add a dependency

1. A correctness argument the standard library cannot meet (the TOML writer is the only current example).
2. A cross-platform behaviour the standard library does not abstract (for example reliable file locking, if the
   platform-specific code becomes larger than the dependency).
3. Never: convenience, "it is standard practice", or a transitive dependency of a transitive dependency.
