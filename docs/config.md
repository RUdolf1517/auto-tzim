# Configuration reference

> **Status: specification, no implementation.** Every field below is contracted; no parser exists yet.
> Related: [architecture](architecture.md) · [safety](safety.md) · [cli](cli.md) · [state](state.md)

Configuration is one TOML file. There are no environment-variable overrides of policy: the only environment variable
the program honours is `AUTO_TZIM_HOME`, which relocates *all* files (config, state, logs) at once, so that a test
cannot accidentally touch a user's state.

## 1. Location and permissions

| Platform | Config | State and logs |
| --- | --- | --- |
| macOS | `~/Library/Application Support/auto-tzim/config.toml` | `~/Library/Application Support/auto-tzim/state/` |
| Linux | `${XDG_CONFIG_HOME:-~/.config}/auto-tzim/config.toml` | `${XDG_STATE_HOME:-~/.local/state}/auto-tzim/` |
| any (tests) | `$AUTO_TZIM_HOME/config.toml` | `$AUTO_TZIM_HOME/state/` |

Directories are created `0700` where the platform supports it, the config, the SQLite database and the log files
`0600`. The file is written to a temporary name in the same directory, flushed, `fsync`ed, renamed over the original,
and the directory is `fsync`ed. A config that fails to parse is **never** overwritten or "repaired": commands fail
with exit `2` and a message naming the problem.

## 2. Top-level fields

| Field | Type | Default | Rules |
| --- | --- | --- | --- |
| `schema` | integer | required | must be `1`; any other value is a validation error, not a migration |
| `[installation]` | table | required | see §3 |
| `[[folder]]` | array of tables | `[]` | zero folders is valid: the program then does nothing |

**Unknown fields are errors.** A typo such as `recurse = true` must not be silently ignored, because the user would
then believe a rule is active while it is not. The error names the field and the accepted one it resembles, for
example: `unknown field "folder[0].recurse": did you mean "recursive"?`.

## 3. `[installation]`

| Field | Type | Default | Rules |
| --- | --- | --- | --- |
| `tzim_path` | string | `""` | empty means "find `tzim` on `PATH`"; a path is resolved and its `realpath` recorded. Must be an absolute path or empty; must not be a directory |
| `update_policy` | enum | `notify` | `notify` \| `manual`. `scheduled` is reserved for v0.2 and is rejected in v0.1 with a message saying so |
| `allowed_major_versions` | array of integers | `[0]` | used only by `update-check`; a detected version outside this list is reported as incompatible and never acted on |

## 4. `[[folder]]`

Every field is listed; nothing is implicit.

| Field | Type | Default | Rules |
| --- | --- | --- | --- |
| `id` | string | generated | opaque, immutable, unique; not derived from the path (a renamed folder keeps its history) |
| `name` | string | equal to the folder's basename at add time | free text, 1–120 characters, no control characters |
| `root` | string | required | absolute, existing directory; canonicalised; must not be `/`, `$HOME` itself, or a system directory; must not contain `AUTO_TZIM_HOME` (D-08) |
| `enabled` | bool | `true` | a disabled folder is not scanned and not run |
| `recursive` | bool | `false` | `false` means the top level only |
| `include_hidden` | bool | `false` | hidden files and directories; `.tzim-*.tmp` stays excluded even when this is `true` (R-14) |
| `cross_filesystems` | bool | `false` | whether discovery may cross a mount point; `false` refuses and records the boundary |
| `follow_symlinks` | bool | `false` | **immutable in v0.1**: any value other than `false` is rejected, because the walk is `lstat`-based by design |
| `allow_hard_links` | bool | `false` | `false` means `st_nlink > 1` is `hardlink_skip`; `true` is reserved and rejected in v0.1 until a verified sibling-safety contract exists |
| `allow_cloud_sync` | bool | `false` | `false` rejects a root matching a known sync root; the check is best-effort and documented as such |
| `formats` | array of strings | `[]` | `[]` means **omit `--formats`** and let the installed `tzim` decide (D-11); the default set is neither reconstructed nor displayed (charter §6-18). Otherwise an allowlist over `png`, `gzip`, `zip`, `ico`, `jpeg`, `tiff`, `zlib` |
| `transforms` | array of strings | `[]` | **must be empty in the MVP** (charter §6-11, D-05): any value is a validation error with the reason. `formats` is not consent to a structural transformation |
| `exclude_globs` | array of strings | `[".git/**", ".hg/**", ".svn/**"]` | matched against the POSIX relative path; `**` spans directories; no brace expansion, no negation |
| `min_file_age_seconds` | integer | `60` | 0–86400; a file younger than this is `skipped_unstable` |
| `jobs` | integer | `1` | v0.1 accepts only `1`; any other value is rejected with a message that parallel files are a later stage |
| `min_free_bytes` | integer | `268435456` | checked before the run starts; below it the run refuses (exit code pending, see [P-03](manager/problems/P-03-exit-codes.md)) |
| `timeout_seconds` | integer | `900` | per file, 1–21600 (D-10) |
| `[folder.schedule]` | table | — | v0.2; **rejected in v0.1** with a message rather than silently ignored |
| `[folder.schedule].enabled` | bool | `false` | manual-only default |
| `[folder.schedule].interval_minutes` | integer | `60` | — |

## 5. What validation refuses, with the reason

| Input | Result | Why |
| --- | --- | --- |
| `root = "/"`, `"$HOME"`, `"/usr"`, `"/etc"`, `"/System"`, `"/var"`, `"/bin"`, `"/sbin"`, platform analogues | error `forbidden_root` | the allowlist must be narrow; a whole home directory is the classic "it optimized everything" report |
| `root = ""`, `"."`, `"../photos"`, a relative path | error `root_not_absolute` | the plan must not depend on the working directory |
| a root that is a file, a symlink, or missing | error `root_not_a_directory` / `root_is_symlink` / `root_missing` | canonicalisation must not launder a link |
| a root containing `AUTO_TZIM_HOME`, or inside it | error `state_inside_root` | the program must not plan its own database or logs (D-08) |
| a known cloud-sync root without `allow_cloud_sync = true` | error `cloud_sync_root` | conflicts with a sync client are silent data races |
| a NUL byte, a control character, a duplicate `id`, a duplicate `root` | error | ambiguity is not a feature |
| `follow_symlinks = true`, `allow_hard_links = true`, `jobs > 1`, `[folder.schedule]` | error `not_in_v01` | each names the reason and the stage that will look at it again |
| `transforms = [...]` | error `transforms_forbidden` | charter §6-11: confirmation-required transformations do not run in the MVP. The field exists so the opt-in has a home; it must be empty (D-05) |

## 6. A complete example

```toml
schema = 1

[installation]
tzim_path = ""                 # find `tzim` on PATH
update_policy = "notify"
allowed_major_versions = [0]

[[folder]]
id = "01JEXAMPLE000000000000000000"
name = "Project images"
root = "/Users/example/Pictures/project"
enabled = true
recursive = false
include_hidden = false
cross_filesystems = false
follow_symlinks = false
allow_hard_links = false
allow_cloud_sync = false
formats = []
transforms = []
exclude_globs = [".git/**", ".hg/**", ".svn/**"]
min_file_age_seconds = 60
jobs = 1
min_free_bytes = 268435456
timeout_seconds = 900
```

A folder added with `auto-tzim folder add ~/Pictures/project` produces the same block with a generated `id` and
`name`, and **does not process anything**: adding a folder arms a plan, nothing more.

## 7. Editing rules

- `folder edit` takes the same fields as `folder add`; every change is validated before it is written, and an invalid
  change leaves the file byte-identical (AC-7).
- Removing a folder removes its profile and its cached eligibility records. It never deletes, moves or modifies a
  user file, and it never rewrites history that already happened.
- Disabling a folder keeps its history and its cache; enabling it again does not re-process files whose identity and
  policy are unchanged (D-06).
- The file may be edited by hand: the schema is the contract, and the program will not "helpfully" reorder or
  reformat it beyond the block it changed.
