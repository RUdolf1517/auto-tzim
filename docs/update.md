# Updates

> **Status: specification, no implementation.** v0.1 is notify-only; controlled update is a later stage and is
> described here so that the boundary is visible.
> Related: [privacy](privacy.md) · [releases](releases.md) · [roadmap](roadmap.md)

## 1. v0.1 — notify only

`auto-tzim update-check` answers one question: is there a newer `tzim` than the one this installation uses? It:

- makes one documented request (proposal D-07, unapproved — see [P-01](manager/problems/P-01-machine-interface.md)
  §7 and [privacy](privacy.md) §1);
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

## 2. v0.3 — controlled update (not designed in detail, not implemented)

The v0.3 stage may update `tzim` through an explicitly configured official Homebrew tap/formula. Requirements are
fixed now, because they are the parts people get wrong:

1. **Explicit configuration.** The tap and formula names are configured; they are never guessed, and a local tap is
   refused.
2. **No update while a run is active.** The global run lock is held for the whole update, so a compression run and an
   update cannot interleave. The reverse also holds: a run refuses to start while an update is in progress.
3. **Recorded before and after.** Old version, new version, and the binary SHA-256 on both sides.
4. **Verified after.** Re-run the version check, the adapter probe, and a read-only/dry-run smoke test on a bundled
   fixture. A failure is reported and does not trigger compression.
5. **No automatic rollback claim** until rollback is implemented and tested. An update that fails must leave a clear
   log and a working previous binary — never a half-replaced one we cannot restore.
6. **`update_policy = "scheduled"`** only after a successful manual update, and only by explicit user action.
7. **`auto-tzim` never updates itself** by this mechanism.

## 3. The acceptance test that matters

A formula that looks right in a pull request is not evidence. v0.3 is accepted only with a real, isolated test:
install the previous release version, then update the same way a user would, then prove that the version actually
changed, that the binary runs, that the adapter accepts it, and that a dry run on a fixture succeeds. This is the
same rule `tzim` applies to itself, and the reason `auto-tzim` will not ship an update mechanism that has only been
reasoned about.

## 4. What v0.1 users should do instead

Read the release notes, install the new version through the channel they used originally, then run
`auto-tzim doctor` — it will report the new binary's hash and version, and the adapter will either accept it or refuse
with exit `3`. Nothing else changes: history is keyed on the binary hash, so files decided by the old binary are
re-examined by the new one (proposal D-06).
