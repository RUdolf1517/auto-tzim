# P-01 · The machine interface `tzim` does not have

Status: **analysis only** — no decision, no upstream issue filed, no implementation authorised.
Sources: `tzim` at `d4e45974b32fc2bb696eee346892846fe3e0a5d1` (`source-inspected`), `SPEC.md` §2.1, §6.3, §7.1, §7.3,
§7.4, §11.1.

---

## 1. What the problem is, in one paragraph

`auto-tzim` is required to know two things about the `tzim` it drives: **what that build can do** (which formats and
opt-ins are compiled in, which JSON schema it emits) and **why a file was not touched** (refused, unsupported, over a
limit, or a failed replacement). `tzim` publishes both facts, but only as English prose for humans and as a
free-text `note` field. `SPEC.md` §2.1 forbids parsing human-readable output, so `auto-tzim` cannot read either
fact. Every design that survives that prohibition has to reconstruct partial knowledge from the outside, and the
reconstruction is lossy in ways that touch retry policy, status history and the update path.

## 2. What `tzim` actually exposes today

| Surface | Shape | Machine-usable? |
| --- | --- | --- |
| `tzim --version` | one line, `tzim <semver>`, stdout, exit `0` (`src/main.rs:22`) | **yes** — the only one |
| `tzim --info` | prose: a tagline, a paragraph, then `Formats in this build: …` and two conditionally printed sentences about JPEG and Zopfli (`src/main.rs:93-120`) | no: sentences, wrapped, no keys |
| `tzim --list-formats` | a three-column fixed-width table (`FORMAT`, `TRANSFORMATION`, `PRESERVED`) followed by two explanatory paragraphs (`src/main.rs:196-221`) | no: column widths depend on content; prose footer |
| `tzim --json` (result stream) | NDJSON, one `file` object per file then one `summary` with `schema: 1` | **yes** — for outcomes |
| per-file `note` | free text, the only place a refusal reason appears | no: prose |
| per-file `error` | `{kind, message}` with `kind` ∈ `io`, `parse`, `verify`, `internal` (`src/error.rs:69-76`) | yes, but only for `failed` |
| exit codes | `0`/`1`/`2`/`3`/`4` + signal codes | yes, coarse |
| docs | `docs/cli.md` promises "object keys and enum values are stable; new optional fields may be added in minor versions" | this is the contract we are allowed to lean on |

`grep -rn capabilit src/ docs/cli.md` finds no capability API: the word appears only in an unrelated comment in the
JPEG handler. There is no `--capabilities`, no `--json` variant of `--info` or `--list-formats`, and no report of
compiled features anywhere in the JSON stream.

## 3. Why human output cannot be the fallback

`SPEC.md` §2.1 and §7.1 are explicit: no parsing of human-readable output, no guessing of flags, no inference from
prose. The mechanical reasons are worth writing down, because "just parse the table" is the obvious first idea:

1. **Formatting is a presentation decision, not a contract.** `--list-formats` pads columns with `{:<6}`/`{:<46}`;
   adding a longer transformation string reflows the table. Colour is disabled for these commands today
   (`docs/cli.md` §10), but the guarantee is about appearance, not parsing.
2. **The table has a prose footer** ("Tzim removes no metadata unless …"), so a naive line parser will treat
   sentences as rows.
3. **Feature visibility is conditional in the text itself** — the JPEG and Zopfli sentences are compiled in or out
   (`src/main.rs:114-120`), so the presence or absence of a sentence is the signal. That is exactly the
   "parse the human text" failure mode: a reworded sentence becomes a silent capability change.
4. **Nothing pins the wording.** `tzim`'s own documentation tracks its `--help` text with `tests/packaging.rs`, but
   `--info` and `--list-formats` carry no stability promise in `docs/cli.md`, unlike the JSON stream.
5. **Locales, terminals, widths** are not a factor for these commands today, but they are one config change away
   from being one — and a parser would inherit that risk without a promise to rely on.

## 4. Consequence A — the "default-enabled set" is unknowable

`SPEC.md` §6.3: `formats = []` means "only the default-enabled processing of the current compatible `tzim`". That set
is **build-dependent**:

- `Cargo.toml:40` — `default = ["png", "gzip", "zip", "ico", "jpeg", "tiff", "zlib", "zopfli"]`.
- `fits` is documented as deliberately **not** in `default`; the release archives therefore have no FITS handler
  (`docs/releases.md` §2).
- `jpeg-v1` (independent JPEG verification) is also not in `default` (`Cargo.toml`, feature block).
- `zip`/`zlib`/`tiff` share the DEFLATE path, so a custom build can lack them independently.

So "default" is not one thing. Two users with "the same version" can have different capabilities, and `auto-tzim`
cannot see the difference. The consequences inside our contract:

| Where | What breaks |
| --- | --- |
| §7.1 (`doctor` and every run record supported formats/opt-ins) | The record would be a version number plus an assumption. Two runs on differently built binaries of the same version would carry identical capability records — a false statement in the evidence trail. |
| §6.3 (`formats` stays an allowlist, not a consent to transformation) | A profile that names `fits` on a FITS-less build produces a per-file `unsupported` from `tzim` — safe, but indistinguishable in our history from "the user's file happened to be unsupported". |
| §7.4 (skip-cache invalidation) | The cache key proposed in D-06 includes the binary SHA-256 exactly because the version cannot express this. That is a workaround, not knowledge: it detects a different binary but cannot say *how* it differs. |
| §11.1 (an incompatible schema is never activated automatically) | Only the JSON `schema` field can be checked, and only after a run has started. There is no pre-flight statement about it. |
| §6.3 (confirmation-required transformations) | We cannot enumerate which transformations a build offers, so an "empty allowlist in v0.1" is a policy decision rather than a fact we could verify. |

## 5. Consequence B — a refusal has no type

`tzim` collapses several different outcomes into `status: "skipped"` with a prose `note`:

| Real situation inside `tzim` | Reported as | Distinguishable? |
| --- | --- | --- |
| Progressive/arithmetic JPEG, LZW TIFF, signed ZIP, ZIP64, preset-dictionary zlib, BigTIFF | `skipped` + note | only by reading the note |
| Over a limit (`--max-pixels`, `--max-uncompressed`, `--max-ratio`, `--memory-budget`) | `skipped` + note | only by reading the note |
| Hard-linked file (`tzim` refuses by default, `--allow-hardlinks` needed) | `skipped` + note | only by reading the note |
| Symlink encountered by `tzim` itself | `skipped` + note | only by reading the note |
| Candidate accepted but the `rename`/`fsync` failed | `failed` (`io`) | yes |
| Content unsupported (no signature, unknown container) | `unsupported` (± note) | yes |

`SPEC.md` §7.3 wants twelve normalized statuses, and §7.4 builds retry policy on them: `already_optimal`,
unsupported and policy/refusal skips are not retried while identity, version and policy hold; `failed` is retried
only manually. Because we cannot type a refusal, three things follow:

1. **Most of our statuses must be produced by us, before `tzim` runs.** That is the fail-closed answer and it is
   also the honest one: symlinks, special files, hard links, instability, excluded paths and opt-in gates are our
   decisions (§6), so `hardlink_skip`, `skipped_unstable`, `skipped_policy` never depend on `tzim` prose.
2. **What remains unmapped is the refusal class we did not predict.** A profile allowing a format that this build
   refuses (see Consequence A) lands in a generic bucket. Choosing one bucket is a policy question with visible
   consequences: map every `skipped` to `skipped_refused` and a limit skip looks like a refusal; map it to
   `skipped_policy` and a genuine refusal looks like our decision, which invites the user to relax policy instead of
   looking at the file.
3. **History and filtering degrade.** `history --status` and the TUI filters (v0.2) would show one bucket plus
   verbatim prose, so "why did this file not shrink" becomes a text search rather than a query.

A fourth, subtler effect: `note` is the only channel for the zlib "weak signature" nuance that `tzim` decided
deliberately (a weak signature that decodes nothing is `unsupported`, a stream that decodes and then breaks stays
`failed`). Any prose matching would have to encode that distinction, which is precisely the drift risk.

## 6. Consequence C — "can this build prompt?" is not answerable from outside

`tzim` today has no prompt at all: `grep -rn stdin src/` returns nothing, so `stdin=DEVNULL` can never deadlock
(`SPEC.md` §7.2). Issue #83 proposes interactive confirmation for opt-in format handling. There is no way for
`auto-tzim` to ask a binary "do you ever read stdin?" — the only signal would be a version, and the only protection
is a pinned version range. Charter invariant 12 (scheduler/non-TTY/JSON never waits for a prompt) is therefore
enforced by a version constraint now, and will need a documented non-TTY refusal contract upstream the moment a
prompting version exists.

## 7. Consequence D — the update check has no in-tree machine endpoint

`docs/releases.md` §1 states the binary never checks the network. There is no documented "latest version" endpoint
owned by the project; the only documented, stable artefacts are the release asset URLs (`…/releases/latest/download/
<asset>`) and `SHA256SUMS`. So the update-check surface is invented by `auto-tzim` (GitHub REST in D-07), not
inherited from `tzim` — a privacy and dependency decision that exists only because the upstream interface does not.

## 8. What would remove each gap (described, **not filed**)

| Gap | Upstream shape that would close it | Size |
| --- | --- | --- |
| Capabilities | `tzim --capabilities --json` → `{version, schema: 1, formats: [...], features: [...], opt_ins: [...], confirmation_required: [...]}` , same key-stability promise as the result stream | small, no behaviour change |
| Refusal typing | an extra field beside `note`, e.g. `"reason": "refused_lzw_tiff"` / `"limit_max_pixels"` / `"skipped_hardlink"`, with a documented enum | small, additive |
| Prompt contract | a written guarantee that with a non-TTY stdin a confirmation-required invocation refuses with a documented status and exit code | documentation plus one check |
| Update source | a `releases/latest.json` in the repository, or a documented statement that the GitHub API is the supported endpoint | trivial but a maintenance promise |

None of these was filed. The owner's instruction for this question was to describe the problem, not to act on it,
and `SPEC.md` §2.2 keeps `tzim`'s contract out of `auto-tzim`'s scope.

## 9. What `auto-tzim` would do while the gap stays open

Fully described in D-01/D-02, marked there as unapproved:

- a version-keyed adapter table with an explicit JSON-schema guard;
- an optional runtime probe (`tzim -n --json` on a 1×1 PNG inside a private temp directory) that proves "the png
  handler exists and emits `schema: 1`" without reading prose;
- recording the binary path, resolved realpath, size, SHA-256, `--version` and the observed `schema` per run;
- fail-closed `incompatible_tzim` (exit `3`) for unknown versions, missing schema, or a failed probe;
- one mapping table from `tzim` statuses/exit codes to ours, with `skipped` → `skipped_refused` and the raw `note`
  stored verbatim.

Costs of that workaround, stated plainly:

1. Every `tzim` release needs a reviewed adapter entry, even when nothing changed for us.
2. The probe proves that *png* works; it cannot prove that every format we allow works.
3. Capability evidence in history is version + binary hash, not a real capability list.
4. Refusal typing stays lossy for cases our policy layer did not predict.

## 10. What would settle the question

1. An owner decision on whether to keep the adapter-only approach, to request the upstream interface, or to wait.
2. If the adapter stays: a test matrix proving every row of the mapping table, including an unknown status, an
   unknown exit code, a missing `schema`, an extra optional field, and a probe failure.
3. If upstream is requested: agreed field names and enum values, written down before either side implements them,
   so that "stable keys" means the same thing in both repositories.
4. For the prompt contract: any concrete statement from upstream about non-TTY behaviour, plus a test that a
   prompting version is refused by the adapter instead of being driven blind.

## 11. Deliberately not done

- No issue opened in `tzim`; no pull request anywhere.
- No adapter code, no probe code, no fixtures.
- No change to `SPEC.md`, no relaxation of §2.1 or §7.1.
- No status committed as "adopted" in [DECISIONS.md](../DECISIONS.md): D-01 and D-02 remain proposals.
