# Brief — AT-02: the file that decides everything else

Written 2026-11-02, at `main` = `fc17b9b`, after AT-01 was accepted at `017ae80`
([review](reviews/REVIEW-017ae80.md)). This is the coder's brief for **AT-02 only**, the same form as
[HANDOFF-AT-01](HANDOFF-AT-01.md). It is not the handoff [format](HANDOFF-TEMPLATE.md) and does not replace it.

**Status: ready, not started.** AT-01's gate is closed ([D-25](DECISIONS.md)) and nothing in the repository blocks this
brief — but the stage does not begin until the owner says so in the issue. A coder who starts before that word is not
ahead; they are un-reviewable.

Read [ASSIGNMENT-AT-MVP-DRAFT](ASSIGNMENT-AT-MVP-DRAFT.md) for the whole MVP and build **only** what is below. AT-03
through AT-09 stay untouched; delivering part of AT-03 "because the config was in the way" is a review failure, not
initiative.

## 1. What AT-02 is for

AT-01 proved that the workspace builds. AT-02 is the first stage that **writes to a disk the user owns**, and it is the
stage where a mistake is quiet rather than loud:

- a config parser that ignores an unknown key tells the user a rule is active that is not;
- a non-atomic write that is interrupted turns the user's only record of their settings into half a file;
- a permission mask inherited from the umask publishes their folder list to every other account on the machine;
- a validator that forgot to check containment lets the tool plan its own state directory as if it were the user's photos.

None of those produce a crash. All of them produce a bug report eight months later that nobody can reproduce. So this
stage is judged less on features than on **the five ways it is allowed to refuse, and the one way it is allowed to write**.

## 2. Build exactly this

| Item | Requirement |
| --- | --- |
| Contract | [`docs/config.md`](../config.md) is the specification — every field, every default, every refusal and its reason string. Do not invent a field, do not invent a default, do not drop a key because it looks useless |
| Where the code lives | the model, the parser, the validator and the atomic write go in `auto-tzim-core`; the subcommands go in `auto-tzim`. No new crate |
| Schema | `schema = 1`. Any other value is a validation error naming the supported one — not a migration, not a warning |
| Parser | TOML 1.0, strict. Unknown fields are **errors**, in every table, with the field's path and a `did you mean` suggestion taken from the documented names (`unknown field "folder[0].recurse": did you mean "recursive"?`) |
| Errors are complete | `config validate` reports **every** error in the file, not the first. A validator that stops at the first bad field fails acceptance point 1 and there is no negotiation about it |
| Dependencies | at most **two**: `serde` (with `derive`) and `toml`, exact-pinned (`=X.Y.Z`), each in **two commits** — the dependency, then the deny allowlist entry — with `cargo tree -e features` in the pull request ([dependencies](../dependencies.md) §2a). A third dependency is a finding to raise, not a commit to make |
| Location | [`config.md`](../config.md) §1, exactly: macOS `~/Library/Application Support/auto-tzim/`, Linux `${XDG_CONFIG_HOME:-~/.config}` and `${XDG_STATE_HOME:-~/.local/state}`, and `AUTO_TZIM_HOME` for tests, which relocates **everything** — config, state, logs — and wins over both |
| Permissions | directories `0700`, files `0600`, set **explicitly after creation**, so that `umask 000` changes nothing. A `0600` file that is `0666` because the mode was inherited is the bug this point exists for |
| Atomic write | temp file in the **same directory**, write, `sync_all` (file), `rename`, then `fsync` the **directory**. The temp file is never read, never resumed and never renamed into place by a later run; a stale temp is removed only by a successful following write |
| Corrupt config | never overwritten, never "repaired", never reformatted. Every subcommand exits `2` and names the problem |
| Canonicalisation | the stored `root` is the canonical path. `lstat` first: a root that is *itself* a symlink is `root_is_symlink` ([D-45](DECISIONS.md)); intermediate components are resolved, which is why `/tmp/x` on macOS is recorded as `/private/tmp/x` |
| Folder identity | the folder's **own name** ([D-43](DECISIONS.md)): default the canonical root's basename, `--name` to override, 1–120 trimmed characters, unique case-insensitively, and the handle every subcommand takes instead of an id |
| Cloud-sync check | the built-in list and its best-effort semantics in [D-44](DECISIONS.md); `allow_cloud_sync = true` bypasses it, and a path that is not in the list passes with no warning |
| CLI surface | `config validate` and `folder add\|list\|show\|edit\|enable\|disable\|remove`, all taking `NAME`. Exit `0` on success, `2` on any validation, usage or parse error. `--json` is **not** in this stage (AT-06); `scan`, `run`, `status`, `pause`, `resume`, `doctor` still exit `2` as unknown commands |
| `folder add` | validates, canonicalises, creates the profile, prints the resulting TOML block, and **processes nothing** — adding a folder arms a plan, nothing more. A second folder with the same name or the same canonical root is `duplicate_name` / `duplicate_root` |
| `folder show` | prints the profile. The "last run and counts" section that [`cli.md`](../cli.md) §2.4 promises needs state, which is AT-05: print `not recorded yet` there rather than inventing fields |
| `folder remove` | removes the profile and prints what was removed. It never touches a user file, never recurses, and never rewrites history |

**Not in this task, at all:** discovery, the walk, the plan, the `tzim` adapter, the runner, state or history files,
`--json`, `--mode`, `scan`, `run`, the scheduler, `update-check`, `doctor`, and anything that spawns a process or opens
a socket. `auto-tzim` remains a program that cannot touch a user file.

## 3. Acceptance points

Each needs its evidence in the handoff. `not-run` is not `passed`, and a summary number the reviewer cannot recompute
is not evidence.

| # | Point | Evidence required |
| --- | --- | --- |
| 1 | `config validate` reports **every** error, not the first | a file with three independent defects (unknown field, wrong type, `jobs = 2`) exits `2` with all three named; fixing two leaves exactly the third |
| 2 | An unknown field names itself and suggests the real one | the invocation, its stderr and its exit status for `recurse = true`, side by side |
| 3 | A rejected edit leaves the file **byte-identical** | `sha256sum` and `stat` before/after a rejected `folder edit`; the same commands around an accepted edit, which does change the file |
| 4 | A corrupt config is never overwritten | a file of garbage: every subcommand exits `2`, and the file is byte-identical afterwards |
| 5 | The write is atomic and the directory survives a crash | a loop that `SIGKILL`s the process at randomised points and then asserts the config **parses and equals either the old or the new content**, never a truncated third thing; no leftover temp file is ever read |
| 6 | Permissions do not depend on the umask | run under `umask 000`: `stat -c %a` (or `stat -f %Lp`) is `600` for the config and `700` for the directories |
| 7 | Every documented refusal fires with its reason | one row per case: `""`, `"."`, a relative path, `/`, `$HOME`, a file, a symlink, a root inside `AUTO_TZIM_HOME`, a root containing it, a cloud-sync root ([D-44](DECISIONS.md) — a folder under `~/Dropbox` or `~/Yandex.Disk` must be **refused**); each exit `2`, each with the reason string from [`config.md`](../config.md) §5 |
| 7a | A name is the identifier, and a duplicate is refused | add `~/Pictures/proj` (name `project` by default); `folder add` of the same root again is `duplicate_root`; `folder add ~/Other --name Project` is `duplicate_name` (case-insensitive); `folder show project` finds the first and `folder show Project` finds the same one; `folder edit project --name trips` renames it and `folder remove project` then says it does not exist |
| 8 | Reserved keys are rejected, and the accepted-but-inert ones are accepted | `follow_symlinks = true`, `allow_hard_links = true`, `jobs = 2`, `[folder.schedule]`, `transforms = ["fits-tile"]` → `2` with the stage named; `update_policy = "notify"` and `allowed_major_versions = [0]` → accepted and inert (D-28) |
| 9 | `AUTO_TZIM_HOME` relocates everything | with it set, no file is created under `$HOME` or `$XDG_*` — shown by a before/after listing of both trees |
| 10 | `folder remove` deletes no user file | a folder containing a real file: after `remove`, the file is present and byte-identical, and the profile is gone |
| 11 | Two scans of the same config give the same bytes | `folder list` twice is identical, and `folder add` of the same root twice is a duplicate-`root` error, not a second entry |
| 12 | Nothing was added that can reach the network, a process, or the registry | `git grep` for `std::net`, `std::process`, `std::fs::remove_dir_all` outside the store; `cargo tree` lists two new crates and no transitive socket or C build |

## 4. The negative controls are the deliverable

A check that cannot fail is decoration, and it fails silently: it stays green and nobody notices until the day it
matters. Five of the points above are only worth anything if you show them going **red** on purpose, each in its own
commit with the run ID:

1. **all-errors-listed** — make the validator return on the first error; the three-defect file must go from "three
   named" to "one named", and that run must be red;
2. **byte-identical** — let a rejected edit write anyway; the `sha256sum` assertion must fail;
3. **permissions** — drop the explicit `set_permissions` call; the `umask 000` test must fail with `666`/`777`;
4. **corrupt-not-overwritten** — let the loader fall back to defaults and save; the byte-identical assertion must fail;
5. **containment** — disable the `AUTO_TZIM_HOME` check; the `state_inside_root` case must go green when it must be red.

Record the run ID, the job name and the message for each, then revert. This is the same rule AT-01 was held to, and the
reason is the same: a future reader has to be able to tell a live rule from a decoration.

## 5. Six traps that are already in the documents

- **The reason strings are contract, not prose.** `forbidden_root`, `root_not_absolute`, `root_not_a_directory`,
  `root_is_symlink`, `root_missing`, `state_inside_root`, `cloud_sync_root`, `not_in_v01`, `transforms_forbidden` are
  asserted by the corpus later. Reproduce them exactly; if one seems wrong, it is a finding, not an edit.
- **The name is the key, and it is typed.** Every command takes `NAME`, quoted if it has spaces. There is no id to copy
  out of a file ([D-43](DECISIONS.md)); a rename is `folder edit OLD --name NEW` and it does not rewrite history that
  already exists.
- **`2` is the config error, `6` is not.** `6` is the preflight refusal *at run time* — no plan, `pause`, free space
  ([D-03](DECISIONS.md), AC-11). A config that fails to parse is the user typing something wrong: exit `2`.
- **Accepted-but-inert keys exist on purpose.** `update_policy` and `allowed_major_versions` are accepted and do nothing
  in v0.1 (D-28). Silently removing them would change a config file a user already wrote.
- **`transforms = []` is the control, not an afterthought.** The row that matters is the empty array being accepted;
  anyone can make a value fail.
- **Do not "fix" the gate documents.** `GATES.md`, `DECISIONS.md`, `config.md` and the corpus are pre-registered. The
  reflection test in AC-10 will compare your implemented defaults against the documented table, so the table is the
  target and a mismatch is your finding to report, not a row to edit.
- **`min_free_bytes` is validated here, enforced later.** It is an integer with a floor of zero in this stage; the
  refusal it implies belongs to the preflight in AT-06.

## 6. What the pull request must contain

The [handoff block](HANDOFF-TEMPLATE.md) with every line filled, `remaining blocker: none`, and
`Merge/Release performed by coder: no`. Commits labelled `AT-02: …`, dependencies as their own commits, one pull
request. Nothing is published, and no `cargo package` returns to CI ([D-42](DECISIONS.md)).

**Plus one line CI is missing.** [D-42](DECISIONS.md) promised that the manifests' registry metadata would be asserted
rather than eyeballed, and it is not: AT-01's fourth acceptance point passed because I read the four manifests, which
means nothing stops that metadata from being deleted. Add the one-line check — `cargo metadata --no-deps` and an
assertion on the seven fields for both crates — so the promise becomes a gate. It is two lines and it closes F-18.

## 7. What this brief does not settle, and I own

Three things had no answer in the documents, so I decided them rather than let a coder invent them:
[D-43](DECISIONS.md) (a folder is identified by its own name — the owner rejected my ULID, correctly, because an
opaque handle means copy-pasting out of `config.toml` every day), [D-44](DECISIONS.md) (the cloud-sync list is built in,
it **refuses** by default, and one flag overrides it), [D-45](DECISIONS.md) (a symlinked root is rejected, intermediate
components are resolved). The owner confirmed all three on 2026-11-02, so they are adopted, not offered: each one is
baked into stored data and must not be renegotiated by the coder.

One thing this stage deliberately leaves open: `folder list`'s exact column set. Text output, stable enough to test,
`id · name · root · enabled` is the obvious four; the owner may want a fifth. That is a UI question, and `--json` in
AT-06 settles the machine form.
