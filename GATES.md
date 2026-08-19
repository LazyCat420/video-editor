# Gates (`scripts/ops/`)

Landed 2026-08-19.

```bash
scripts/ops/scoped-gate.sh    # gates covering this tree's changes
scripts/ops/full-suite.sh     # everything, in sequence
scripts/ops/scoped-gate.sh --self-test    # prove the exit-status plumbing
```

## What this actually bought: `tests/` had never been run

This repo had **no test or check script of any kind**, and `tests/` holds ten
integration test files — `timeline_tests.rs`, `filter_graph_tests.rs`,
`export_stability_and_visual_parity_tests.rs`, `playback_sequence_tests.rs`,
`text_overlay_tests.rs`, `slide_dnd_tests.rs`, `sidebar_width_tests.rs`,
`envelope_tests.rs`, `effects_and_stickers_tests.rs`, `pptx_position_test.rs` —
that nothing invoked.

First run, 2026-08-19: **35 tests, all passing.** They were correct and
unwatched. The `test` leg is now their home and it is enforcing.

## No separate cross-target leg, and that is deliberate

`.cargo/config.toml` sets `[build] target = "x86_64-pc-windows-gnullvm"`, so
the ordinary `fmt`/`clippy`/`test` legs are **already** building the shipped
target, and the test executables run through the WSL interop runner. An extra
explicit windows leg would be the same build twice.

`GATE_NEEDS_WIN_TOOLCHAIN=1` is still set in `gate-config.sh`, because the
llvm-mingw toolchain has to be on `PATH` for any dependency whose `build.rs`
probes for a C compiler. Without it a leg dies in `cc-rs` with "failed to find
tool x86_64-w64-mingw32-clang" — an **environmental** failure that reads
exactly like a code defect. If you see that, run
`scripts/setup-win-toolchain.sh`; do not go looking through the diff.

## Enforcement, set by measurement

Measured on `main` before choosing:

| leg | on `main` 2026-08-19 | enforcing? |
|---|---|---|
| `cargo fmt --check` | RED (rc=1) | no — advisory |
| `cargo clippy --all-targets` | RED (rc=101) | no — advisory |
| `cargo test --no-fail-fast` | GREEN (35 tests) | **yes** |

A gate that is red on `main` the day it ships trains everyone to ignore its
reds, so the two style gates report without voting (`run_leg_advisory`). Flip
`GATE_FMT_ENFORCE` / `GATE_CLIPPY_ENFORCE` to `1` in the same change that lands
each cleanup.

## The exit code is the product

Every leg's command runs **bare** — never `cmd | grep`, never `cmd | tail`, so
`$?` is always the command's own. `--self-test` proves it in one command: three
synthetic legs (green, `exit 42`, and a red **behind a pipe**), asserting the
child failed and named both reds.

Exit codes: `0` green · `1` a leg failed · `3` a structural change selected no
gate · `10` setup failure · `75` the box was full and **nothing ran**, which is
not a red suite.

## Shared machinery

`gate-lib.sh`, `crate-map.py`, `scoped-gate.sh` and `full-suite.sh` are
identical copies across four sibling Rust repos; only `gate-config.sh` is
local. `GATE_LIB_VERSION` (currently **3**) is the drift tripwire — fix the
library, carry it to all four, bump the version. The library also meters CPU
against a machine-global pool (the box is shared) and leases a warm
`CARGO_TARGET_DIR` when one is free, always printing which it got.

## Open items

- fmt and clippy are both red on `main` and unscheduled; two of three legs do
  not vote until that is fixed.
- The 35 tests are a baseline recorded on the day the gate landed, not a
  coverage claim. Nobody has checked what they *don't* cover.
