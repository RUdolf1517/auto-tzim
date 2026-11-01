# Updates

> **Status: specification, no implementation.** `update-check` ships in **v0.2**, not v0.1 (D-28: the MVP makes no
> network request at all). Controlled update is v0.3. Both are described here so the boundary is visible before either
> exists.
> Related: [privacy](privacy.md) · [releases](releases.md) · [roadmap](roadmap.md)

## 1. v0.2 — notify only

> Deferred from v0.1 by D-28. The design below is unchanged; only its version moved, so that the first release a stranger
> installs opens no socket. Exit `5` is reserved for it and is not produced anywhere in v0.1.

`auto-tzim update-check` (v0.2) answers one question: is there a newer `tzim` than the one this installation uses? It:

- makes one documented request (D-07 — a manager default the owner may override; see [privacy](privacy.md) §1);
- compares semantic versions, and reports the state as one of `up_to_date`, `update_available`, `incompatible_major`,
  `error`;
- shows the current version from `tzim --version` and the binary identity from `doctor`, so the user can see *which*
  installation is behind;
- exits `0` when an answer was obtained (including "a newer version exists"), `5` when the check itself failed.

It does **not**: download an archive, verify a checksum on the user's behalf, run an installer, replace a binary,
write outside its own cache, or change `tzim_path`. Those are invariant 14 and are not "small additions".

`allowed_major_versions` (default `[0]`) exists for exactly one purpose: a major version bump of `tzim` is a contract
change — new JSON schema, changed flags, changed defaults — and `auto-tzim` must say "incompatible" rather than
"update available". Nothing is activated automatically.

### 1.1 Failure is not success

| Situation | Report | Exit |
| --- | --- | --- |
| Network unreachable, DNS failure, timeout | `error` with the reason | `5` |
| HTTP 403/429 (rate limit) | `error`, with a note that GitHub limits unauthenticated requests | `5` |
| Malformed body, HTML error page, non-semver tag | `error` | `5` |
| `tzim` missing or its version unparseable | `error`, pointing at `doctor` | `3` |
| Newer version in an allowed major | `update_available` with the upgrade hint for the documented channel | `0` |
| Newer version in a disallowed major | `incompatible_major`, with an explicit "do not upgrade blindly" | `0` |

An error must never be rendered as "you are up to date". That single rule is worth more than the feature.

## 2. v0.3 — controlled update of **both** binaries (D-31)

Two targets: the `tzim` binary this installation depends on, and `auto-tzim` itself. Two modes, chosen once by the user
(D-32): `confirm` asks before replacing anything, `auto` does not ask.

### 2.1 Two modes, one set of checks

| | `update_mode = "confirm"` (default) | `update_mode = "auto"` |
| --- | --- | --- |
| A newer version exists | prints the version, what changed and the command | performs the update |
| Replacing `tzim` | asks | asks nothing |
| Replacing `auto-tzim` | asks | asks nothing |
| Checks, verification, revert | identical | identical |

`auto` removes the question, never a check: a bad checksum is refused in both modes, a contract jump always asks, and a
failed verification reverts and reports it either way. It is a separate setting from the optimisation `mode` (D-29),
because "optimise without asking" is not a statement about "replace the program".

### 2.2 How each target is replaced

**`tzim` — a self-contained binary.** Download to a temporary path beside the target, verify SHA-256 against the published
`SHA256SUMS`, `fsync`, `rename` over the old path, keep the old inode until the new one has answered `--version`. A
`SIGKILL` at any point leaves either the old binary or the new one, never half of either.

**`auto-tzim` — a static binary, updated by the same mechanism** (D-33 makes the two targets the same problem). The one
thing that still differs is *who owns the file*:

| Installation | Who performs it | Why |
| --- | --- | --- |
| a release tarball, unpacked by the user | **we do**, by atomic replacement | we recorded its hash at install time, so its provenance is ours |
| Homebrew formula in a tap | **we ask `brew`**, then verify | the package manager owns the Cellar; replacing files under it is how an installation becomes unrepairable |
| system package, a source checkout, an unknown path | **refused**, exit `6`, with the command to run by hand | provenance unknown — automatic replacement is not ours to perform |

Provenance is recorded at install time and shown in `doctor`; it is never inferred from `PATH` when an update runs.
Replacing a binary we cannot account for is exactly the case where a careful tool should stop and print a command.

### 2.3 Rules that hold in both modes

1. **No update while a run is active**, and no run while an update is in progress: one global lock, both directions.
2. **Recorded before and after:** version, installer, artifact SHA-256, and the reason for the update.
3. **Verified after:** `--version`, `doctor`, the adapter probe against `tzim`, and a read-only smoke run on a bundled
   fixture. A failure runs no compression.
4. **Revert is a tested path, not a claim.** A failed verification reinstalls the previous version through the same
   installer and records `reverted`. If the revert itself fails, both versions and both hashes are reported and nothing
   else is attempted.
5. **Never a downgrade** unless explicitly requested; **never a contract jump in `auto` mode** — a major version, or a
   change to a documented status, exit code or config field, always asks.
6. **`tzim` is updated before `auto-tzim`**, so a mismatched pair is diagnosed by a working half.
7. **No `curl | sh`, no execution of an unverified download, and no redirect to an undocumented host.**
8. **Signing is not claimed.** Integrity is SHA-256 against the published manifest: it protects against corruption and a
   broken mirror, not against a compromised publisher account.
9. **`update_policy = "scheduled"`** only after one successful manual update, and only by explicit user action.

### 2.4 What an update never does

It never touches a file in a user's folder, never resumes or alters a plan, never runs compression as a side effect, and
never replaces a binary whose installation it does not understand.

## 3. The acceptance test that matters

A formula that looks right in a pull request is not evidence. v0.3 is accepted only with a real, isolated test:
install the previous release version, then update the same way a user would, then prove that the version actually
changed, that the binary runs, that the adapter accepts it, and that a dry run on a fixture succeeds. This is the
same rule `tzim` applies to itself, and the reason `auto-tzim` will not ship an update mechanism that has only been
reasoned about.

## 4. How to update `tzim` by hand (v0.1 and v0.2)

There is no `update-check` and no updater: v0.1 opens no socket at all, and v0.2 only tells you a version exists. Read
the release notes, install the new version through the channel you used originally, then run
`auto-tzim doctor` — it will report the new binary's hash and version, and the adapter will either accept it or refuse
with exit `3`. Nothing else changes: history is keyed on the binary hash, so files decided by the old binary are
re-examined by the new one (D-06).
