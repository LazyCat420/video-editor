---
part: Runtime
status: shipped
updated: 2026-10-22
---
# Dependency security 2026-10-22: `zip` 0.6.6 to 9.0.5, and a committed `Cargo.lock`

`cargo audit` reported three vulnerabilities, all on `zip 0.6.6`: RUSTSEC-2026-0367, -0368 and
-0369 (SUID bits, symlink and permission handling when extracting an archive). All three are fixed
in `zip >= 9.0.0`.

## Who uses `zip`
Only this crate, directly (`Cargo.toml`); no other dependency pulls it in. The app **writes** one
archive format, `.pptx`, through `ZipWriter` in `src/export/pptx_exporter.rs`. Nothing in `src/`
extracts an archive; `ZipArchive` is used only by two test files to read back the export. So the
advisories were not reachable from this app. The bump is for a clean audit and to avoid shipping the
affected code. As [chapter 02](02-what-is-actually-open.md) notes, no UI path currently calls the
PPTX exporter at all.

## What changed
| Item | From | To |
|---|---|---|
| `zip` | 0.6.6, feature `deflate` | 9.0.5, feature `deflate-flate2` |
| `src/export/pptx_exporter.rs` | `zip::write::FileOptions::default()` | `zip::write::SimpleFileOptions::default()` |
| `Cargo.lock` | untracked, gitignored, dated 2026-08-19 | committed (zip bump applied on top of it) |

`FileOptions` became generic in zip 2 and later; `SimpleFileOptions` is the non-generic alias, and
it is the only code change. In zip 9 the `deflate` feature adds `zopfli` and `zlib-rs`; the
`deflate-flate2` feature keeps the existing `flate2` backend, so the lockfile gains only `typed-path`
and `nix 0.31.3` (a macOS and BSD target dependency, not built here) and loses `byteorder`. The
required Rust is 1.88; the toolchain here is 1.94.

## Why `Cargo.lock` is now committed
This is an application (`[[bin]]`, Windows `.exe`, `installer/`), and Cargo guidance is to commit
the lockfile of a binary so builds and `cargo audit` see the same versions. `Cargo.lock` was in
`.gitignore`, so the audit result depended on whichever lock a machine happened to have. The
committed lock is the existing one with only the `zip` change applied; no other crate moved.

## Validation (primary checkout, through `testrun`)
- Baseline on `main` before the change: `cargo test --no-fail-fast` gave 148 tests passed, 0 failed.
- After the change: see the validation section of the commit log (same test command).
- `cargo audit` after: the three `zip` advisories are gone. The remaining `unmaintained` and
  `unsound` warnings (paste, ttf-parser, zbus_names, zvariant, zvariant_derive) are only reported.

## Resolved tickets
| Ticket | Root cause | Commit |
|---|---|---|
| RUSTSEC-2026-0367/0368/0369 | `zip 0.6.6` below the fixed release 9.0.0 | see git log (`fix(deps): zip 0.6.6 -> 9.0.5`) |
