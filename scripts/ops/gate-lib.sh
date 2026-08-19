#!/usr/bin/env bash
# gate-lib.sh — shared library for the Rust gate scripts (scoped-gate.sh,
# full-suite.sh). Vendored IDENTICALLY into every Rust repo under sun/
# (pinball-knight, drift-king, video-editor, spritefusion-pixel-snapper);
# repo-specific facts live in gate-config.sh beside it, never here.
# If you fix something here, carry the fix to every copy and bump the version.
GATE_LIB_VERSION="3"

# ── hard rules this file encodes (each learned from a real incident) ──────────
# 1. A leg's command runs BARE — never `cmd | grep PASS`, never `cmd | tail`.
#    A trailing pipe makes $? the filter's, and a full suite once shipped four
#    real reds as exit 0 that way. Filter a captured log AFTER rc is recorded.
# 2. cargo test always gets --no-fail-fast: plain fail-fast reports 1 of N test
#    binaries as if it were the whole suite.
# 3. Legs run in SEQUENCE. Concurrent halves oversubscribe the shared box and
#    manufacture phantom timeout failures that read as real reds.
# 4. The shared CARGO_TARGET_DIR is safe for exactly ONE worktree at a time
#    (measured: a test in worktree A failed with worktree B's sabotage
#    signature). lease_target_dir turns that rule into a mechanism.
# 5. Exit 75 means "the box was full, NOTHING RAN" — it is not a red suite.

set -u -o pipefail
# Deliberately NO `set -e` here: legs must be able to fail while the script
# keeps collecting statuses. Setup steps guard themselves with explicit `|| die`.

# Per-target extra cargo args. Default: none. A repo whose shipped artifact is
# narrower than its crate overrides this — spritefusion's wasm leg must be
# --lib, because the [[bin]] drags in the native-only CLI path.
gate_target_extra_args() { :; }

# Cross-compiling to windows-gnullvm needs the user-local llvm-mingw toolchain
# on PATH — not for the LINKER (.cargo/config.toml names that by absolute path)
# but for any dependency whose build.rs probes for a C compiler. `ring` does,
# and without this dk-game's windows leg dies in cc-rs with "failed to find
# tool x86_64-w64-mingw32-clang" — an ENVIRONMENTAL failure that reads exactly
# like a code defect. A gate that goes red for a reason the diff cannot cause
# is as corrosive as one that goes green over a real break.
gate_win_toolchain_path() {
  local bin="${LLVM_MINGW_BIN:-$HOME/.local/opt/llvm-mingw/bin}"
  if [ -x "$bin/x86_64-w64-mingw32-clang" ]; then
    case ":$PATH:" in *":$bin:"*) ;; *) export PATH="$bin:$PATH" ;; esac
    return 0
  fi
  echo "gate: WARNING: windows target configured but no llvm-mingw at $bin — run scripts/setup-win-toolchain.sh; the windows leg will fail for a TOOLCHAIN reason, not a code one." >&2
  return 1
}

gate_die()  { echo "gate: FATAL: $*" >&2; exit 10; }
gate_note() { echo "gate: $*"; }

# ── repo facts ────────────────────────────────────────────────────────────────
gate_default_branch() {  # $1 = repo dir
  git -C "$1" symbolic-ref --quiet --short refs/remotes/origin/HEAD 2>/dev/null \
      | sed 's|^origin/||' \
    || { git -C "$1" show-ref --verify --quiet refs/heads/main && echo main || echo master; }
}

gate_primary_checkout() {  # first worktree listed is always the main checkout
  git worktree list --porcelain | awk '/^worktree /{print $2; exit}'
}

# ── changed set: committed-vs-merge-base ∪ staged/unstaged ∪ untracked ────────
# Emits "STATUS<TAB>PATH", repo-relative. Renames contribute BOTH paths (R).
# Untracked files count as adds — a brand-new .rs file is the classic
# structural change and must be seen. Caller must be in the repo root.
gate_changed_set() {  # $1 = base ref
  {
    git diff --name-status -M "$1" HEAD 2>/dev/null
    git diff --name-status -M HEAD
  } | awk -F'\t' '{
        if ($1 ~ /^R/)      { printf "R\t%s\nR\t%s\n", $2, $3 }
        else if (NF >= 2)   { printf "%s\t%s\n", substr($1,1,1), $2 }
      }'
  git status --porcelain=v1 | awk '$1=="??"{ line=$0; sub(/^\?\? /,"",line); printf "A\t%s\n", line }'
}

# ── leg runner: every gate's status reaches the exit code ─────────────────────
LEG_NAMES=(); LEG_STATUS=()
run_leg() {  # run_leg <name> <cmd…>   — command runs BARE, never behind a pipe
  local name="$1"; shift
  printf '\n── LEG %s ──\n  $ %s\n' "$name" "$*"
  local t0=$SECONDS rc
  "$@"
  rc=$?
  LEG_NAMES+=("$name"); LEG_STATUS+=("$rc")
  printf -- '── LEG %s: %s (rc=%d, %ds)\n' "$name" \
    "$([ "$rc" -eq 0 ] && echo PASS || echo FAIL)" "$rc" $((SECONDS - t0))
}

run_leg_advisory() {  # like run_leg, but a red is REPORTED and never counted.
  # For gates that are not yet green on the default branch (e.g. pre-existing
  # rustfmt drift): a gate that starts red on main trains people to ignore
  # reds. Promote to run_leg the moment the branch is clean.
  local name="$1"; shift
  printf '\n── LEG %s (ADVISORY) ──\n  $ %s\n' "$name" "$*"
  local t0=$SECONDS rc
  "$@"
  rc=$?
  LEG_NAMES+=("$name(advisory)"); LEG_STATUS+=(0)
  printf -- '── LEG %s: %s (rc=%d, %ds) — ADVISORY, not counted\n' "$name" \
    "$([ "$rc" -eq 0 ] && echo PASS || echo FAIL)" "$rc" $((SECONDS - t0))
}

gate_finish() {  # the ONLY green exit path; exits 1 if any leg was red
  local fails=0 i
  echo; echo "══ SUMMARY (${#LEG_NAMES[@]} legs) ══"
  for i in "${!LEG_NAMES[@]}"; do
    if [ "${LEG_STATUS[$i]}" -eq 0 ]; then
      echo "  PASS ${LEG_NAMES[$i]}"
    else
      echo "  FAIL ${LEG_NAMES[$i]} (rc=${LEG_STATUS[$i]})"
      fails=$((fails + 1))
    fi
  done
  if [ "$fails" -ne 0 ]; then echo "══ $fails LEG(S) FAILED ══"; exit 1; fi
  echo "══ ALL LEGS PASS ══"
  exit 0
}
trap 'echo "gate: INTERRUPTED with ${#LEG_NAMES[@]} legs recorded — this run is NOT green" >&2' INT TERM

# ── CPU meter: same machine-global budget as pk-run.sh, wire-compatible ───────
# Same lock files ($LOCKDIR/thread-NN.lock), same fds (300+i), same <> open
# (never > — probing must not truncate a holder's label), same v1|class|pid|
# cwd|label META written through the held fd, so `pk-run.sh --status` in
# braindeadbot-client reports these gates alongside everything else. Budget is
# derived from the SAME cached topology file — this vendored copy must never
# mint a pool of a different size.
GATE_LOCKDIR="${BDB_SLOT_LOCKDIR:-$HOME/.cache/bdb-cpu-slots}"
GRANT=0
GATE_GOT=()
meter_grab() {  # elastic, class test; sets $GRANT; exit 75 = never ran
  local ask="${1:-}" reserve="${BDB_SLOT_RESERVE:-2}"
  local phys logical smt pool budget min timeout deadline i fd meta
  mkdir -p "$GATE_LOCKDIR"
  # shellcheck disable=SC1091
  [ -r "$GATE_LOCKDIR/topology" ] && . "$GATE_LOCKDIR/topology"
  logical="${LOGICAL:-$(nproc)}"; phys="${PHYS:-$(( logical / 2 ))}"
  smt=$(( logical / phys )); (( smt >= 1 )) || smt=1
  pool=$(( phys - reserve )); (( pool >= 1 )) || pool=1
  budget=$(( pool * smt ))
  [ -n "$ask" ] || ask=$(( budget / 2 > 1 ? budget / 2 : 2 ))
  (( ask > budget )) && ask=$budget
  min="${GATE_MIN_THREADS:-2}"; (( min > ask )) && min=$ask
  timeout="${GATE_METER_TIMEOUT:-300}"
  meta="v1|test|$$|$PWD|${GATE_LABEL:-rust-gate}"
  deadline=$(( SECONDS + timeout ))
  while :; do
    GATE_GOT=()
    for ((i = 0; i < budget && ${#GATE_GOT[@]} < ask; i++)); do
      fd=$((300 + i))
      eval "exec $fd<>'$GATE_LOCKDIR/thread-$(printf '%02d' "$i").lock'"
      if flock -n "$fd"; then GATE_GOT+=("$i"); else eval "exec $fd>&-"; fi
    done
    (( ${#GATE_GOT[@]} >= min )) && break
    for i in ${GATE_GOT[@]+"${GATE_GOT[@]}"}; do eval "exec $((300 + i))>&-"; done
    GATE_GOT=()
    (( SECONDS < deadline )) || {
      echo "gate: could not get $min of the $budget-thread budget in ${timeout}s — the box is full, NOTHING RAN (exit 75 is not a red suite)" >&2
      exit 75
    }
    sleep 0.5
  done
  GRANT=${#GATE_GOT[@]}
  for i in "${GATE_GOT[@]}"; do
    truncate -s 0 "$GATE_LOCKDIR/thread-$(printf '%02d' "$i").lock" 2>/dev/null || true
    eval "printf '%s\n' \"\$meta\" >&$((300 + i))"
  done
  if (( GRANT < ask )); then
    echo "gate: [test] $GRANT thread(s) of $budget — SHRUNK from $ask; slower, not wrong; not a timing datapoint" >&2
  else
    echo "gate: [test] $GRANT thread(s) of $budget ($((budget - GRANT)) left for everyone else)" >&2
  fi
}

# ── cargo target-dir lease: the 141.5s→13.5s trick, made safe ─────────────────
# flock on one per-repo lock file in the same machine-global dir. Win → share
# the primary's warm target/. Lose → this tree's own target/ (cold but SAFE).
# The choice line ALWAYS prints before any cargo runs — a silently confounded
# timing is the failure mode this exists to kill. fd 250 is inherited by every
# cargo child; the kernel releases the lease however this script dies.
# Never bare-execute a binary out of the shared target dir — another tree may
# have built it with its own paths baked in; always go through cargo.
lease_target_dir() {
  local primary repo_base lockfile holder
  primary="$(gate_primary_checkout)"
  repo_base="$(basename "$primary")"
  lockfile="$GATE_LOCKDIR/cargo-target-${repo_base}.lock"
  mkdir -p "$GATE_LOCKDIR"
  if [ "${GATE_NO_LEASE:-0}" = 1 ]; then
    export CARGO_TARGET_DIR="$PWD/target"
    gate_note "CARGO_TARGET_DIR=$CARGO_TARGET_DIR (--no-lease — own dir, unconditionally)"
    return 0
  fi
  exec 250<>"$lockfile"
  if flock -n 250; then
    truncate -s 0 "$lockfile" 2>/dev/null || true
    printf 'v1|cargo-target|%s|%s|%s\n' "$$" "$PWD" "${GATE_LABEL:-rust-gate}" >&250
    export CARGO_TARGET_DIR="$primary/target"
    gate_note "CARGO_TARGET_DIR=$CARGO_TARGET_DIR (lease WON — warm shared cache)"
  else
    holder="$(head -1 "$lockfile" 2>/dev/null || true)"
    exec 250>&-
    if [ "$(pwd -P)" = "$(cd "$primary" && pwd -P)" ]; then
      # the primary lost: its own target/ IS the contested dir — use a sibling
      export CARGO_TARGET_DIR="$primary/target-gate"
    else
      export CARGO_TARGET_DIR="$PWD/target"
    fi
    gate_note "CARGO_TARGET_DIR=$CARGO_TARGET_DIR (lease HELD by [${holder:-?}] — own dir, cold but SAFE; do not read this run as a warm-path timing)"
  fi
}

# ── self-test: prove the exit-status plumbing, not the gates ──────────────────
# Entry scripts call gate_selftest_maybe "$0" "$@" FIRST. `--self-test` re-execs
# the script with GATE_SELFTEST=1, which swaps the real legs for three synthetic
# ones: green, red (rc=42), and red-behind-a-pipe (proves pipefail is live in
# leg context). The parent asserts the child failed AND named both reds.
gate_selftest_legs() {
  run_leg selftest-green true
  run_leg selftest-red bash -c 'exit 42'
  run_leg selftest-pipe-red gate__pipe_red
  gate_finish
}
gate__pipe_red() { bash -c 'exit 7' | cat; }

gate_selftest_maybe() {  # $1 = the entry script's own path; $2.. = its args
  local self="$1"; shift
  if [ "${GATE_SELFTEST:-0}" = 1 ]; then gate_selftest_legs; fi   # never returns
  local want=0 a
  for a in "$@"; do [ "$a" = "--self-test" ] && want=1; done
  [ "$want" = 1 ] || return 0
  local out rc fail=0
  out="$(GATE_SELFTEST=1 bash "$self" 2>&1)"; rc=$?
  [ "$rc" -ne 0 ] || { echo "self-test: child exited 0 over a red leg — plumbing BROKEN"; fail=1; }
  grep -qF 'FAIL selftest-red (rc=42)' <<<"$out"  || { echo "self-test: red leg not reported"; fail=1; }
  grep -qF 'FAIL selftest-pipe-red' <<<"$out"     || { echo "self-test: pipe-red leg not reported (pipefail dead?)"; fail=1; }
  grep -qF 'PASS selftest-green' <<<"$out"        || { echo "self-test: green leg not reported"; fail=1; }
  if [ "$fail" -eq 0 ]; then
    echo "self-test: exit-status plumbing PROVEN (child rc=$rc, both reds named)"
    exit 0
  fi
  printf '%s\n' "$out" | tail -20
  exit 1
}
