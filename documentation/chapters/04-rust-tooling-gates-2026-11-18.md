---
part: Rust tooling gates 2026-11-18
status: shipped
updated: 2026-11-18
type: record
evidence: 16efcd6 (tooling wave) and 0d5146a (deny ignore cleanup) on main; all six files below present and `[lints.rust]`/`[lints.clippy]` in Cargo.toml (docs audit 2026-11-18)
---
# Rust tooling gates 2026-11-18: toolchain pin, `[lints]`, nextest, deny, smoke

Single wave that put this repo on the same boring baseline as drift-king's engine
workspace: a pinned toolchain, one lint table, one test-gate profile, one supply-chain
policy, and one deterministic end-to-end smoke test.

## What landed

| File | Change |
|---|---|
| `rust-toolchain.toml` | pins `1.94.1` + `x86_64-pc-windows-gnullvm` target (the default target set by `.cargo/config.toml`) so fresh clones do not drift |
| `Cargo.toml` | `[lints.rust]` / `[lints.clippy]` mirror of drift-king's engine table at **warn** level (`unsafe_code = "forbid"`, `clippy::all`/`pedantic`/`nursery`/`unwrap_used`/`expect_used` at warn, the same allow-list); `publish = false` added — the app ships as a Windows installer, never to crates.io |
| `.config/nextest.toml` | `[profile.gate]` with `fail-fast = false` so one red test never hides the rest |
| `deny.toml` | cargo-deny policy copied from drift-king with repo-specific amendments below |
| `tests/smoke.rs` | 2 deterministic headless tests (no FFmpeg, no GPU, no window): timeline `Track::duration` math and save → load round-trip through `Project::save_to_file` / `Project::load_from_file` |
| `clippy-baseline.txt` | recorded clippy warning counts (see below) |

## deny policy notes

`cargo deny check` is **green** (advisories, bans, licenses, sources). Deviations from
the drift-king baseline, each commented in `deny.toml`:

- `LicenseRef-UFL-1.0` allowed: `epaint_default_fonts 0.29.1` (via `egui`/`eframe`)
  ships the Ubuntu Font License 1.0, which is not an SPDX identifier cargo-deny recognises.
- `private = { ignore = true }` + `publish = false`: the root app declares no
  `license` field; licensing the app itself is an owner decision, not a dependency
  finding. Third-party crates are still fully checked.
- Advisory ignores (unmaintained crates with no replacement on this stack, each
  commented in `deny.toml`): `RUSTSEC-2026-0192` (rustybuzz, via the font stack),
  `RUSTSEC-2026-0543` (chrono, direct dep; replacing it is production work outside
  this slice), `RUSTSEC-2026-0371/0370/0372` (zbus_names/zvariant stack via
  `rfd -> ashpd`, the Linux file dialog backend). Vulnerabilities are never ignored.
- Tokio `RUSTSEC-2026-0386` (`AsyncFd` unsoundness) was first ignored, then fixed
  properly in `0d5146a` by the semver-compatible bump tokio 1.53.1 → 1.53.3 and
  removed from the list. The same commit dropped three ignores
  (`RUSTSEC-2026-0206`, `RUSTSEC-2024-0436`, `RUSTSEC-2023-0089`) that matched
  nothing in this tree.
- Two yanked crates were fixed properly instead of ignored: `cargo update -p
  async-executor` (1.14.0 → 1.14.1) and `cargo update -p clipboard-win` (via `arboard`).

## Test counts (this wave)

- Before: 134 tests. After: **136** (134 + 2 smoke). Full suite green via
  `TESTRUN_FULL_SUITE=1 testrun -- cargo nextest run --profile gate` (gnullvm target,
  wine runner).

## Clippy baseline

`cargo clippy --all-targets` (1.94.1, warn-level lint table): **652 warning lines**;
top lints `cast_possible_truncation` (865 raw messages incl. multi-target),
`cast_lossless` (619), `too_many_lines` (136), `float_cmp` (121). All are pre-existing;
nothing fails (warn level). Counts recorded in `clippy-baseline.txt`.

## Smoke-test failure proof

The smoke test was proven to actually bite: breaking `Track::duration` arithmetic made
`timeline_duration_is_the_max_clip_end` fail; restored afterwards.

## Follow-ups (not in this slice)

- rustybuzz/chrono/zbus advisory ignores become actionable if upstream ships fixed
  releases; until then they are reported, not silenced without a reason.
