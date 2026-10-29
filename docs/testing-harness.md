# Testing harness — contract

> **Status: specification, no implementation.** The harness is test-only code; it ships in nothing. This page exists
> because [EVIDENCE-PROTOCOL](manager/EVIDENCE-PROTOCOL.md) tells the reviewer to run `harness/run-case <ID>` and to read
> an invocation log — those interfaces were referenced before they were defined. Coder task: AT-08 (#8).
> Related: [testing](testing.md) · [fixture corpus](manager/FIXTURE-CORPUS.md) · [evidence protocol](manager/EVIDENCE-PROTOCOL.md)

The harness has exactly four jobs: start a **fake `tzim`** with a chosen behaviour, **record every invocation** of it,
run **one named case** from the frozen corpus, and fetch and verify the **pinned real binary**. Everything it produces is
raw material for the reviewer, so its output formats are part of the contract, not an implementation detail.

## 1. Layout

```text
tests/
  harness/
    fake_tzim.py            # the fake binary: one script, behaviour from a scenario file
    install_fake.py         # writes a `tzim` shim into a temp bin dir and prepends it to PATH
    run_case.py             # `run-case <ID>`: one corpus case, one JSON result line
    pinned.py               # fetch + checksum + cache of the pinned real tzim artifact
    evidence.py             # bundle writer + manifest with per-file SHA-256
  cases/                    # one YAML/JSON per corpus ID (N-01.yaml, R-10.yaml, …)
  conftest.py               # pytest fixtures: tmp root, AUTO_TZIM_HOME, fake bin dir
```

Nothing here may import `auto_tzim` internals except through its public entry points (`python -m auto_tzim` / the
installed console script), so that a test cannot pass because it called a private function the CLI never reaches.

## 2. The fake `tzim`

One script, installed as `tzim` in a temporary directory that is prepended to `PATH`; `AUTO_TZIM_HOME` and the shim
directory are per-case. Behaviour is data, never code paths in the harness:

```json
{
  "version": "tzim 0.14.0",
  "version_exit": 0,
  "scenario": "verification_failure_on_file_3",
  "steps": [
    {"match": ".*", "emit": "optimized", "exit": 0},
    {"match": "3rd", "emit": "error_verify", "exit": 4}
  ]
}
```

Required behaviours (the corpus in [FIXTURE-CORPUS](manager/FIXTURE-CORPUS.md) §5–§6 names each one):

| Behaviour | Scenario key | Notes |
| --- | --- | --- |
| version output | `version`, `version_exit` | any string, including `garbage` |
| each status | `emit`: `optimized`, `already_optimal`, `unsupported`, `skipped` | with and without `note` |
| error object | `emit: error_io` / `error_verify` / `error_parse` / `error_internal` | `kind` must match |
| malformed output | `emit: truncated_json`, `two_summaries`, `no_summary`, `wrong_type`, `unknown_status`, `extra_field` | |
| hang | `hang: true` | sleeps forever, must be killed by our timeout |
| ignore `SIGTERM` | `ignore_sigterm: true` | requires `SIGKILL` from us |
| read stdin | `read_stdin: true` | blocks; the timeout is the only exit |
| mutate the file | `mutate: true`, `emit: optimized` | proves our post-run identity check |
| tripwire on prose commands | `tripwire: ["--info","--list-formats","--check-deps","--dry-run"]` | exits `99` if invoked with any of them (`A-10`) |
| assert no TTY | `assert_not_tty: true` | exits `98` if `isatty()` on any of the three streams (`R-02`) |
| die by signal | `die: "SIGSEGV"` | |

The fake must never be able to "pass" a test by doing nothing: if the scenario file is missing or unreadable, it exits
`97` and writes the reason to the invocation log.

## 3. Invocation log — the proof that scan is read-only

Every fake invocation appends one JSON object per line to `$ATS_INVOCATION_LOG` before doing anything else:

```json
{"ts":"2026-10-29T19:00:01.123456Z","event":"spawn","pid":4211,"ppid":4200,"argv":["tzim","--version"],
 "cwd":"/tmp/ats/case-X-04","stdin_isatty":false,"stdout_isatty":false,"stderr_isatty":false,
 "stdin_path":"/dev/null","scenario":"version_ok","sha256_self":"…"}
```

| Field | Why it exists |
| --- | --- |
| `argv` | proves no shell, no `-r`, no `--dry-run`, no `--formats` when `formats = []`, and that a filename stayed a single element |
| `stdin_isatty` / `stdout_isatty` / `stderr_isatty` | the no-PTY assertion (`R-02`), asserted from inside the child |
| `stdin_path` | `stdin=DEVNULL` is observable as `/dev/null`, not as a closed fd |
| `cwd` | a run must never execute with the target root as cwd |
| `event: exit` | written by the same process with `exit_code` and `duration_ms`, so a killed child still leaves a spawn row |

**Counting rule used by the reviewer** (`EVIDENCE-PROTOCOL` §5): a *path invocation* is any spawn whose `argv[1:]`
contains an element that is not a flag and not absent. A scan must produce exactly one spawn per run — `--version` — and
zero path invocations. An apply must produce at least one path invocation.

The log is append-only and never truncated by the harness; each case gets its own file so a failure cannot be masked by
another case's rows.

## 4. `run-case <ID>`

```console
$ python -m tests.harness.run_case X-04 --bundle /tmp/evidence/<sha>
{"case":"X-04","expected":{"exit":6,"processed":0},"observed":{"exit":6,"processed":0},
 "pass":true,"artifacts":["X-04/stdout.txt","X-04/stderr.txt","X-04/invocations.jsonl","X-04/root-before.txt","X-04/root-after.txt"]}
```

Contract:

| Aspect | Rule |
| --- | --- |
| arguments | exactly one corpus ID, optional `--bundle DIR`; unknown ID ⇒ exit `2` with the known IDs listed |
| environment | fresh `AUTO_TZIM_HOME`, fresh temp root, fake bin dir on `PATH`, network blocked, `TZ=UTC`, `LC_ALL=C` |
| output | one JSON line on stdout, nothing else; human diagnostics on stderr |
| exit | `0` pass, `1` fail, `2` harness misuse (unknown case, missing fixture), `3` environment cannot run the case |
| determinism | same ID ⇒ same result on the same platform, twice in a row; no reliance on wall-clock, order, or locale |
| cleanup | temp roots removed on success, **kept** on failure and their path printed |
| side effects | only inside its temp dir and the bundle; never touches the repository, the user's home, or the real `tzim` |
| no sleeps | except the single documented real-age case (`S-03`), which sleeps at most 2 s and says so in its output |
| prohibition | must not import `auto_tzim` internals; must drive the CLI |

A case may declare `"platform": "unix-only"` (for `K-08`, `N-17`); on a platform that cannot build it, the result is
`{"pass":false,"skipped":true,"reason":"…","platform":"…"}` and the reviewer treats it as a finding unless the corpus
says otherwise — a skip is never silently a pass.

## 5. Pinned real binary

`pinned.py` fetches **one** named artifact of **one** named version, verifies it against the release `SHA256SUMS`, caches
it under `$ATS_CACHE` (default: the platform cache dir), and prints the resolved path plus its SHA-256:

```console
$ python -m tests.harness.pinned --version 0.14.0 --target x86_64-unknown-linux-musl
{"path":"/cache/tzim-0.14.0-musl/tzim","sha256":"…","version":"0.14.0","sums_line":"<the matching line from SHA256SUMS>"}
```

Rules:

- `latest` is rejected as an argument (D-13): the version is a literal in the repository, bumped only in a reviewed
  change together with the adapter entry.
- A checksum mismatch aborts with exit `4` **before** the binary is executed; the mismatching line and both hashes are
  printed.
- Offline CI is a **failure** with exit `5` and a message naming the URL — never a skip. A skipped real-binary suite is
  how a project convinces itself it tested something.
- The binary is executed only from the cache path, never from a `PATH` lookup, so a system `tzim` cannot be mistaken for
  the pinned one.

## 6. Evidence bundle

`evidence.py` writes the layout the handoff must reference:

```text
evidence/<head-sha>/
  manifest.json            # {"head_sha","created","platform","python","python_version","tzim":{...},"cases":N}
  SHA256SUMS               # one line per file below, generated last, never edited
  <case-id>/
    stdout.txt  stderr.txt  exit.txt
    invocations.jsonl
    root-before.txt  root-after.txt          # recursive listing + hashes of the target root
    stdout.parsed.jsonl                      # only when the case expects valid NDJSON
  summary.json             # counts per status, recomputable from the case files alone
```

`manifest.json` records the head SHA, the OS/arch, the Python version, and the identity of both binaries (ours and
`tzim`'s). `SHA256SUMS` is written after every other file and verified by the reviewer, so a later edit to a case file is
detectable.

## 7. What the harness must never do

- Call `tzim` with a **real** target path in any case that expects a read-only scan — the fake is the only binary those
  cases may see, and the real binary is used only in the `T1`…`T10` contract cases of the study, by the harness itself.
- Use `shell=True`, `os.system`, or a string command anywhere; every process is an argv list.
- Allocate a PTY (no `pty`, no `script`, no `expect`).
- Reach the network except in the `update-check` mocks (`W-02`…`W-04`), which are in-process fakes, not sockets.
- Rely on the developer's `AUTO_TZIM_HOME`, home directory, locale, or clock.
- Treat "no rows" as success: an empty invocation log in a case that expects invocations is a failure (`R-14` expects an
  *empty* log, which is why the expectation is written in the case file, not inferred).
