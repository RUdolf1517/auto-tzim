# Formats and transformations

> **Status: specification, no implementation.**
> Related: [guarantees](guarantees.md) · [config](config.md) · [safety](safety.md)

`auto-tzim` implements no format. This page exists because "which formats does it support?" is the first question a
user asks, and the honest answer has three parts: `tzim` decides, this program only filters, and the set is
build-dependent.

## 1. Who decides what

| Question | Answer | Where |
| --- | --- | --- |
| Is this file a supported format? | `tzim` | `tzim`'s detector, at run time |
| Is this construct inside the format supported? | `tzim` | per-format contract, refusal lists |
| Is the candidate acceptable? | `tzim` | verification, then `before > after` |
| Is this file *allowed* to be handed over? | `auto-tzim` | discovery + policy |
| Which formats may be offered? | `auto-tzim` filters; `tzim` decides feasibility | `formats` allowlist |

A file that `auto-tzim` allows and `tzim` refuses is not an error: it is `skipped_by_tzim`, exit `0`, with `tzim`'s own
note stored verbatim. Widening our allowlist never widens what `tzim` accepts, and no refusal is ever re-classified from
its wording.

## 2. `formats` semantics

- `formats = []` (default) — `--formats` is **not passed**; the binary decides what "default-enabled" means (D-11). The
  set is deliberately not reconstructed or displayed: there is no machine-readable capability list, and a guess would be a
  fabricated capability (charter §6-18). For reference only, the release binaries of `tzim` `0.14.0` are built with png,
  gzip, zip, ico, jpeg, tiff, zlib and zopfli; `fits` and `jpeg-v1` are outside that default, and JPEG is attempted only
  when named (`--formats jpeg`).
- `formats = ["png"]` — only PNG; everything else is `skipped_policy`, decided by us before any process starts.
- `formats = ["png", "jpeg"]` — PNG and JPEG; naming `jpeg` is the only way JPEG is ever attempted, exactly as in
  `tzim` (`--formats jpeg`).
- An unknown name, or a name the adapter knows is absent from this build, is a validation or policy outcome with a
  stated reason — never a silent pass-through.

**`formats` is an allowlist, not consent to a structural transformation.** Naming a format permits optimisation
inside that format; it does not authorise a change of container, layout or metadata class.

## 3. Transformations (`transforms`)

Some `tzim` transformations change structure visibly — today the example is FITS tile compression (`--fits-tile`),
which turns an `IMAGE` HDU into a `BINTABLE` that readers indexing `NAXIS1` can see. Such a transformation is
confirmation-required in our model: it needs its own opt-in field, separate from `formats`.

In v0.1 the allowlist **must be empty** (charter §6-11, D-05): the field exists, validates, and rejects every value with
a reason. Enabling one is a later change that needs (a) an installed build with the handler, (b) a real-binary
integration test, and (c) an updated safety statement — none of which exists.

| Transform | Requires | State in v0.1 |
| --- | --- | --- |
| `fits-tile` | `tzim` built with `--features fits` **and** `--fits-tile` at run time; release archives do not carry it | rejected by validation |
| `strip-metadata` | a deliberate decision that metadata may be removed (tzim guarantee class G4) | not exposed |
| anything else | does not exist | rejected |

## 4. Format-by-format notes that affect *our* policy

| Format (as `tzim` names it) | What the user should know before adding it to a folder | Our behaviour |
| --- | --- | --- |
| `png` / APNG | safe default; APNG reported as `png` | allowed by default |
| `gzip` (`.gz`, `.tgz`, `.svgz`) | header preserved verbatim | allowed by default |
| `zip` family (`.zip`, `.jar`, `.apk`, `.docx`, `.epub`, `.whl`, …) | signed, ZIP64, encrypted and multi-disk archives are refused by `tzim` | allowed; refusals recorded honestly |
| `ico` / `cur` | embedded PNGs optimised, bitmaps copied | allowed by default |
| `jpeg` | off by default in `tzim`; typically saves 1–3 %; the riskiest handler there | **off** unless named in `formats` |
| `tiff` (classic, strips) | BigTIFF, tiling and LZW refused | allowed by default |
| `zlib` (`.zlib`, `.zz`) | a weak signature is not proof of a bug; `tzim` distinguishes "decodes nothing" from "decodes then breaks" | allowed by default |
| `fits` | not in release builds; tile compression is a structural transformation | rejected in v0.1 |
| PDF, FLAC, WebP, AVIF, MP4, GIF, xz/zstd/7z | refused by `tzim` after measurement; not planned | never enabled; a file of that kind is `skipped_unsupported` with `tzim`'s note |

## 5. What we do with unsupported content

`tzim` reports `unsupported` for files whose signature it does not recognise. `auto-tzim` records that status, keeps
the bytes untouched, and caches the decision against the file's identity and the binary hash so the file is not
re-examined on every run. Changing the `tzim` binary (a new version, or a different build) invalidates the cache — the
new binary may understand what the old one did not. This is why the invalidation key includes the binary SHA-256 and
not only the version (D-06).

## 6. What we will never do with formats

- Convert one format to another (that is transcoding, not optimisation).
- Offer a quality setting, "visually lossless", or a lossy fallback: none of them exist in `tzim` and none may be
  synthesised here.
- Guess a format from a file name extension. The extension is used only for filtering; the decision belongs to
  `tzim`'s detector.
- Retry a refused file with a different set of flags in the same run.
