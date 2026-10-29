# Testing strategy

> **Status: specification, no implementation.** No test exists in this repository yet. The table in §6 is a plan,
> deliberately marked as such — `tzim` learned the hard way that a traceability row without a test is a lie, so here
> every row carries an explicit state.
> Related: [safety](safety.md) · [guarantees](guarantees.md) · [manager gates](manager/GATES.md)

## 1. What has to be true before anything is called tested

1. **Behaviour, not compilation.** A test that only checks that a module imports proves nothing about a program whose
   whole value is refusing to touch the wrong file.
2. **Failures are the main subject.** The suite is mostly about what does *not* happen: no process started, no byte
   changed, no prompt, no retry, no second run.
3. **Negative controls.** Every assertion that something did not happen comes with a companion case proving the
   assertion can fail (a file that *should* be processed, a change that *should* be detected).
4. **No sleeping.** `min_file_age_seconds` is driven through an injected clock (proposal D-14). One integration test
   uses a real second of age, and it is marked.
5. **Determinism.** No test depends on enumeration order, locale, or the machine's speed. Timeouts in tests are
   generous and asserted, not raced.

## 2. Layers

| Layer | What it covers | Speed | Runs where |
| --- | --- | --- | --- |
| Unit | config validation, root rules, glob matching, argv construction, status mapping, version comparison, retention maths | milliseconds | every push, every platform |
| Fake `tzim` | every documented status and exit code, malformed/partial JSON, hang, stdin read, version mismatch, file mutation behind our back, temporary-file failure, arbitrary file names | fast, deterministic | every push, every platform |
| Real `tzim`, pinned | the contract we actually depend on: T1–T10 of the [study](manager/TZIM-STUDY-2026-10-29.md#4-exact-checks-for-the-real-binary-input-to-at-08), plus one optimisable PNG and one refused file | seconds | every push, every platform, artifact verified by SHA-256 |
| End-to-end CLI | every command, human and `--json`, exit codes, refusal paths, crash recovery | seconds | every push, every platform |
| Failure injection | kill between steps, full disk (simulated), read-only destination, lock contention, corrupt database, corrupt config | seconds | every push |

## 3. The fake `tzim`

A small program (or a Python entry point invoked through a wrapper script) with a controlled behaviour table selected
by an environment variable or a config file. It must be able to:

- emit each status, with and without a note, with and without an error object;
- exit `0`, `1`, `2`, `3`, `4`, and die by signal;
- print malformed JSON, a truncated line, two summaries, no summary, an unknown `type`, an unknown optional field;
- hang forever, ignore `SIGTERM`, read `stdin` and block;
- mutate the file it was given while reporting success (this is the case that proves our post-run identity check is
  real);
- fail to create its temporary file;
- record the `argv` it received and whether a shell was involved.

The fake exists to test **our** reaction, never to model `tzim`'s correctness; the real binary tests the contract.

## 4. Fixtures

| Corpus | Contents | Why |
| --- | --- | --- |
| Names | spaces, tabs, newline, `;`, `$(…)`, backticks, quotes, leading `-`, `--help`, emoji, CJK, combining characters, a name that is not valid UTF-8 (Unix) | no interpretation of a name, ever |
| Kinds | regular (tiny, empty, 2 GiB sparse), symlink to file/dir/outside, broken symlink, FIFO, socket, device (when permitted), hard-linked pair, hidden file, orphan `.tzim-*.tmp` | discovery and policy |
| Stability | file touched during discovery, during the run, between plan and spawn; file deleted between plan and spawn | races are the interesting part |
| Formats | a 1×1 PNG, a real screenshot PNG, a `.gz`, a `.zip`, a text file, an extension/signature mismatch | real-binary assertions |
| Pathological | a directory depth of 65, a folder with 10 000 entries, a root that is a symlink, a root inside `AUTO_TZIM_HOME` | limits and validation |

Fixtures are created by code, not committed as binary blobs, except the small real files needed for the real-binary
assertions. Every fixture that is committed carries its SHA-256 in the test that uses it.

## 5. What must be asserted about a "nothing happened" outcome

For every skip, refusal, unsupported file, failure and dry run:

1. the bytes are identical before and after (`SHA-256`);
2. the `mtime` is unchanged (unless the platform cannot express it — then it is asserted as "unchanged or explained");
3. no `.tzim-*.tmp` file remains that we created (we never create one);
4. no `optimized` row exists in history;
5. no process was started when the decision was ours (asserted by the fake runner's invocation log).

## 6. Traceability (planned — not one row below has a test yet)

| Row | Test | State |
| --- | --- | --- |
| T-CFG-01 | unknown config field is an error naming the field | planned |
| T-CFG-02 | atomic write survives injection at each step | planned |
| T-CFG-03 | a corrupt config is never overwritten | planned |
| T-ROOT-01 | forbidden roots, relative roots, symlinked roots, state-inside-root | planned |
| T-DISC-01 | hostile names are never interpreted | planned |
| T-DISC-02 | symlinks, special files, hard links skipped with a reason | planned |
| T-DISC-03 | plan is byte-identical across two runs and across enumeration orders | planned |
| T-STAB-01 | modified/deleted file becomes `skipped_unstable` at each stage | planned |
| T-RUN-01 | argv has no shell and no `-r`; golden per profile | planned |
| T-RUN-02 | every status/exit row of the mapping table | planned |
| T-RUN-03 | hang is killed; `SIGTERM`-ignoring child is killed | planned |
| T-RUN-04 | malformed JSON is a failure, never a success | planned |
| T-RUN-05 | child reports success after mutating the file — we detect it | planned |
| T-STATE-01 | crash recovery marks `interrupted`, never `optimized` | planned |
| T-STATE-02 | concurrent run exits `4` and processes nothing | planned |
| T-STATE-03 | second run spawns zero processes | planned |
| T-CLI-01 | every command × exit code, human and `--json` | planned |
| T-UPD-01 | mocked 200/403/429/500/malformed; no socket outside `update-check` | planned |
| T-DOC-01 | documented defaults equal schema defaults | planned |

When a test is written, its row changes state to `exists` and gains the file name and the assertion count. A row may
not be added to this table before the test exists.

## 7. Evidence a handoff must carry

Raw per-case rows, before/after checksums, the pinned `tzim` artifact's hash and the matching `SHA256SUMS` line, CI
run identifiers with the head SHA, the exact commands to reproduce, and the platform matrix. Aggregates must be
recomputable from the rows; a summary number with no rows behind it is not evidence.
