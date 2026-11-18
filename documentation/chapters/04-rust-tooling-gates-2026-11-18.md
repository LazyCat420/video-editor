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
  ships the UniBit Font License, which is not an SPDX identifier cargo-deny recognises.
- `private = { ignore = true }` + `publish = false`: the root app declares no
  `license` field; licensing the app itself is an owner decision, not a dependency
  finding. Third-party crates are still fully checked.
- Advisory ignores (documented, no patched releases exist — same family policy as
  drift-king/pkv2 which ignore the rustybuzz/ttf-parser/paste trio): `RUSTSEC-2026-0543`
  (chrono, direct dep), `RUSTSEC-2026-0386` (tokio `AsyncFd` unsoundness — unused on
  this stack), `RUSTSEC-2026-0371/0370/0372` (zbus_names/zvariant stack via
  `rfd -> ashpd`, the Linux file dialog backend).
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

- chrono/tokio/zbus advisory ignores become actionable if upstream ships fixed
  releases; until then they are reported, not silenced without a reason.
