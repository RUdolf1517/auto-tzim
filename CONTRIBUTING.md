# Contributing

> **Status: no code exists yet.** v0.1 is not started; the specification, the safety model and the acceptance matrix
> are in place, and the first implementation follows the process below.

## 1. Read these first

1. [`SPEC.md`](SPEC.md) — the product contract (Russian; the project's specification language).
2. [`docs/safety.md`](docs/safety.md) — the invariants a change must not weaken.
3. [`docs/manager/GATES.md`](docs/manager/GATES.md) — the acceptance matrix a pull request is reviewed against.
4. [`docs/architecture.md`](docs/architecture.md) — where code is allowed to live.

## 2. What this project will not accept

- A codec, a file writer, a repacker, or any code that modifies a user file. `tzim` does that.
- A fallback for a refused file, a `--force`, a "continue on error", or an automatic retry of `failed`.
- A prompt, or a code path that reads `stdin`.
- Telemetry, analytics, or a network call outside `update-check`.
- An undo/backup claim.
- A dependency that runs install-time code or contacts the network.
- A change that widens a default: a bigger root, symlink following, hard links, cloud sync, hidden files.

Each of those is a product decision, not a style preference. If a change needs one, it starts as a decision document
and an issue, not as a patch.

## 3. How work is organised

- Every change maps to an issue (`AT-01`…`AT-09` for the MVP); commits are labelled `AT-0N: …` so the mapping can be
  reconstructed from history.
- The MVP is one pull request. Later stages are separate pull requests; mixing a scheduler, a TUI or a GUI into the MVP
  is refused even when it is already written.
- A pull request carries the handoff block from [`docs/manager/ASSIGNMENT-AT-MVP-DRAFT.md`](docs/manager/ASSIGNMENT-AT-MVP-DRAFT.md)
  §"Final handoff format": head SHA, issue/commit map, CI runs at that SHA, raw evidence locations, tests added,
  documentation updated, known limitations, and `remaining blocker: none` or an exact blocker.
- The author of a change does not merge it, and does not close the issue it fixes.

## 4. Tests

- Behaviour first: what must **not** happen is the main subject (no process started, no byte changed, no retry, no
  second run).
- Every assertion that something did not happen needs a negative control proving the assertion can fail.
- No test sleeps for `min_file_age_seconds`; the clock is injected (proposal D-14). One integration test may use a real
  second of age, and it is labelled.
- A test that only proves a module imports or that compilation succeeded is not evidence for anything in
  [`docs/manager/GATES.md`](docs/manager/GATES.md).
- Real-binary tests use a pinned `tzim` release artifact verified against the release `SHA256SUMS`; never `latest`.

## 5. Documentation

- Documentation is part of acceptance, not a follow-up ticket. A behaviour change that is not in the docs is an
  incomplete change.
- Documents marked "Status: specification, no implementation" describe the contract, not existing behaviour. When the
  first implementation lands, those banners change in the same pull request.
- Do not edit [`SPEC.md`](SPEC.md) or [`MANAGER-SPEC.md`](MANAGER-SPEC.md): they are verbatim copies with a recorded
  provenance ([docs/spec/PROVENANCE.md](docs/spec/PROVENANCE.md)). A specification change happens in the source
  repository, followed by a new copy.

## 6. Reporting a safety problem

A file changed outside the plan, a change during a dry run, or a prompt in a non-interactive run is a Critical finding:
it stops merges and releases until reproduced and fixed. Include the `auto-tzim` version, the `tzim` identity from
`doctor`, the run id, and the raw rows — do not include absolute paths of personal files unless necessary.

## 7. Building and running (once the code exists)

```sh
python3.12 -m venv .venv && . .venv/bin/activate
pip install -e '.[dev]'          # planned; pinned in requirements files
pytest -q
ruff check . && ruff format --check .
```

Until the package exists, these commands are the plan, not instructions.
