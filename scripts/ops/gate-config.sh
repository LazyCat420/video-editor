#!/usr/bin/env bash
# gate-config.sh — video-editor's gate facts. Single crate; the closure logic
# degenerates to "the crate, or nothing". Sourced, never executed.

GATE_REPO_NAME="video-editor"
GATE_CLIPPY_ARGS=""

# Set by MEASUREMENT on main, not preference. Probed 2026-08-19 on main:
#   cargo fmt --check      → RED   (rc=1)
#   cargo clippy           → RED   (rc=101)
#   cargo test             → GREEN (35 tests — see below)
# So both style gates REPORT and do not vote; flip each to 1 in the change that
# lands its cleanup. The test leg is enforcing from day one and is the reason
# this gate is worth having here at all.
GATE_FMT_ENFORCE=0
GATE_CLIPPY_ENFORCE=0

GATE_TRIGGERS=()

# ⚠️ THIS REPO'S DEFAULT BUILD TARGET IS ALREADY WINDOWS. `.cargo/config.toml`
# sets [build] target = x86_64-pc-windows-gnullvm with a win-runner, so the
# ordinary fmt/clippy/test legs ALREADY exercise the shipped target and its
# test exes run through WSL interop. There is therefore no second target to
# add — an extra explicit windows leg would be the same build twice.
#
# What this gate actually buys here: `tests/` holds ten integration test files
# that NO script in the repo ran before. The `test` leg is their first home.
GATE_TARGETS=()

# …but every ordinary leg here IS a windows build, so the llvm-mingw toolchain
# still has to be on PATH for any dependency whose build.rs probes for a C
# compiler. GATE_TARGETS is empty, so say it explicitly.
GATE_NEEDS_WIN_TOOLCHAIN=1

gate_target_entry()     { echo "video-editor"; }
gate__target_list()     { echo "video-editor"; }
gate_target_pkgs()      { shift; echo "video-editor"; }
gate_target_full_pkgs() { echo "video-editor"; }
gate_target_flag()      { echo win; }

gate_extra_legs()       { :; }
gate_extra_legs_names() { echo "(none)"; }
gate_full_extra_legs()  { :; }
