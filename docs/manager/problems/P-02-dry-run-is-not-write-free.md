# P-02 · A `tzim` dry run is not write-free

Status: **analysis only** — no decision, no implementation authorised.
Sources: `tzim` `docs/cli.md` §6 and `docs/safety.md` §2, `src/replace.rs:28` (`source-inspected`); `SPEC.md` §6.2,
§8.1, §15.2; charter invariant 2.

---

## 1. The fact

`--dry-run` in `tzim` is not "look and report". It is "do everything except the final rename":

> "`--dry-run` performs the **complete** pipeline: detection, candidate generation and verification. Only the final
> rename is skipped, and the temporary file is deleted. This makes the reported numbers exact instead of an estimate,
> at the cost of doing the real work. A dry run still creates a candidate in the destination directory (verification
> and exact sizing require the real bytes) and removes it afterwards. It never opens an existing file for writing,
> and it never replaces anything, so every outcome is still G1."

Three consequences are stated outright there and one is not:

1. a **real candidate file is created in the destination directory** — not in `TMPDIR`, because verification compares
   against the original and an atomic replacement must stay on the same filesystem;
2. the directory must be **writable**, and there must be **free space** of roughly the candidate's size;
3. the candidate is deleted afterwards, and the original is untouched — the bytes of user files are identical before
   and after (G1);
4. (not stated, but true) if the process is killed hard, the candidate stays behind as an orphan.

## 2. Why it matters for our contract

`SPEC.md` §15.2 and charter invariant 2 say the first run of a new folder is dry-run only. The point of the gate is
that a user sees a plan before anything is at risk. That still holds. What does *not* hold is the stronger phrasing a
reader may assume — "a dry run changes nothing" — if it is written without qualification:

| Statement | True? | What must be said instead |
| --- | --- | --- |
| "No user file is modified" | yes | — |
| "No user file is opened for writing" | yes | — |
| "The folder is untouched afterwards" | **conditionally** | true if the run completes; false after `SIGKILL`/panic/loss of power (an orphan `.tzim-*.tmp` may remain) |
| "A dry run works on read-only media" | **no** | it needs write permission on each destination directory |
| "A dry run needs no free disk space" | **no** | it needs about one candidate's worth of space, per file, sequentially |
| "A dry run is invisible to sync clients" | **no** | see §4 |

`auto-tzim` can preserve the spirit of the invariant (nothing of the user's is at risk, no permanent change) but it
cannot promise "the directory is never written", and a README that promises it would be the exact kind of false
promise the charter's invariant 16 forbids in the neighbouring case.

## 3. Where the mismatch shows up in our own design

1. **Two products answer to one word.** `auto-tzim scan` (our plan: which files, which argv) and `tzim -n` (exact
   predicted savings, real CPU, real temporary file) are different operations with different costs. `SPEC.md` §8.1
   says a first run needs "a preceding `scan`/dry-run or an explicit `--dry-run` in the same call" — that sentence is
   satisfiable by either, and the difference between them is precisely whether the target directory is written. This
   is Q3/decision D-04, still unapproved.
2. **The refusal path becomes environment-dependent.** A first-run gate that uses `tzim -n` fails on a read-only
   mount, a full disk, a quota-limited account, or a directory the user can read but not write — all situations where
   `auto-tzim` has done nothing wrong and no file is at risk. Without a distinct run-level outcome
   (see [P-03](P-03-exit-codes.md)) that failure is indistinguishable from "a file failed", which is false.
3. **Disk-space policy interacts with `min_free_bytes`.** The profile has a floor (`min_free_bytes`, default
   256 MiB in the example of §5.2). A dry run needs *candidate* space on top of that floor, but nothing in the
   specification says whether the check must account for `tzim`'s temporary file. If it does not, a dry run can
   consume the reserve it was supposed to protect.
4. **Orphaned candidates become our diagnostic problem.** Names follow `.tzim-<pid>-<nanoseconds>-<attempt>.tmp`
   (`src/replace.rs:28`) and `docs/safety.md` §2 states the guard does not survive `SIGKILL`, `abort()` or power
   loss. Therefore: discovery must never plan such a file (including when `include_hidden = true`, since the name is
   dot-prefixed but the user may have enabled them), and it must say something when it sees one. `auto-tzim` must not
   delete it — it is not our data, and deleting a file we did not create is a data-safety action.
5. **History honesty.** A dry run must not write `optimized` rows (§8.2: "не изменяет файлы или history как будто
   обработка произошла"). With `tzim -n` we get `would_optimize`, whose number is exact — attractive evidence, and
   exactly the kind of attractive row that later gets mistaken for a completed run. The row must carry `dry_run`
   prominently.
6. **Time and CPU are not free.** `tzim -n` does the real compression. On the first run of a folder of photographs
   with `--zopfli` enabled, the "preview" can take as long as the real thing. The first-run gate therefore has a cost
   the user must be told about, and a cancel path that matters more than usual.

## 4. The interaction nobody documents: sync clients and indexers

The candidate file appears **inside the user's folder**, with a dot-prefixed name and a 2–10 MB payload, for the
duration of one file's compression. Consequences outside `tzim`'s responsibility but inside `auto-tzim`'s
documentation duty:

- **Cloud sync** (iCloud Drive, Dropbox, OneDrive, Google Drive, Nextcloud): a sync client may upload the temporary
  candidate and then delete it remotely — bandwidth and, in some products, a version-history entry. This is one more
  reason `allow_cloud_sync = false` is the default, and it must be stated where users read about dry runs, not only
  in the config reference.
- **Backup and indexing** (Time Machine, Spotlight, Windows Search, antivirus real-time scanning): each candidate is
  a new file to index and scan, then to forget. On a large first run this can double the visible churn. Nothing we
  can fix; something we must warn about.
- **Network filesystems** (SMB/NFS): `rename` semantics and `fsync` reliability differ from local filesystems, and
  `tzim`'s atomic-rename guarantee is stated for local filesystems. The MVP should name network shares as untested
  rather than quietly inherit an assumption.

## 5. What "invariant 2 is satisfied" can therefore mean, precisely

A phrasing that is both true and useful:

> The first run of a new folder writes nothing permanent: no original file is opened for writing, no file is
> replaced, and no history row claims a completed optimization. To produce exact numbers, `tzim`'s own dry run builds
> and verifies a real candidate in the destination directory and deletes it; the directory must therefore be writable
> and have free space, and after a hard kill an orphaned `.tzim-*.tmp` may remain. `auto-tzim` never creates such a
> candidate itself and never deletes one it did not create.

That paragraph is a documentation obligation, not a code change, and it is the kind of sentence the review gate
(AC-10 "documentation matches behaviour") exists to check.

## 6. Options (described, not chosen)

| Option | What it means | Cost |
| --- | --- | --- |
| A. Both layers (Q3 option A) | `scan` = our plan, no `tzim`; `run --once --dry-run` = plan + `tzim -n --json` | Exact numbers; needs the temporary-file wording everywhere; read-only targets refuse rather than preview |
| B. Plan only (Q3 option B) | `tzim` is never invoked in a dry run; "how much would I save" is an estimate | Zero directory writes; the first real run is the first real measurement; the §15.1 promise "exact deterministic plan" becomes "exact *list*", not "exact *saving*" |
| C. Plan plus an explicit probe flag (Q3 option C) | a third mode that asks `tzim` for numbers | Extra surface in the MVP CLI, one more pair of tests |
| D. Ask upstream for a no-write estimator | `tzim -n --no-temp` that estimates rather than measures, or a documented note that estimation is not offered | Not available today; `tzim`'s whole design says exact numbers require the bytes |

None is adopted. The owner asked for the problem, not a solution.

## 7. What would settle it

1. An owner answer to Q3 (which of A/B/C, and whether either satisfies the first-run gate).
2. Pre-registered documentation wording for the chosen option, including the orphan-file caveat.
3. Tests that assert the *right* thing: final-state equality of the directory (no extra entries), no byte change to
   any original, and an explicit test that a dry run on a read-only destination directory produces the documented
   outcome instead of a traceback.
4. A decision on whether `min_free_bytes` must be evaluated against candidate space as well (P-03 lists this as a
   refusal condition).

## 8. Deliberately not done

- No decision recorded as adopted; D-04 stays proposed.
- No change to `SPEC.md` §15.2 or to invariant 2's text in the charter.
- No code, no fixtures, no tests.
