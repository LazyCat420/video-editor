#!/usr/bin/env bash
# scoped-gate.sh — run exactly the gates that cover this tree's changes.
#
#   scripts/ops/scoped-gate.sh [--base <ref>] [--allow-empty] [--no-lease]
#                              [--dry-run] [--self-test]
#
# Computes the changed set (committed vs the merge-base with the default branch
# PLUS staged/unstaged/untracked), maps it to workspace crates, takes the
# REVERSE-dependency closure, and runs: fmt → clippy → one `cargo test
# --no-fail-fast` over the closure → `cargo check` for EVERY configured
# cross-target → the repo's extra legs. The cross-target legs run whenever the
# closure is non-empty — a cfg-gated function is covered by no gate that skips
# its cfg (a wasm-only compile break once shipped under 877 green native tests).
#
# Exit: 0 all legs green (or docs-only diff) · 1 a leg failed · 3 STRUCTURAL
# change selected no gate (add/delete/rename nothing compiles — the dangerous
# shape; --allow-empty downgrades to a warning) · 10 setup failure · 75 the box
# was full and NOTHING RAN (not a red suite).
#
# This is the quick/train gate the landing queue's `predict --gate-rc` consumes;
# `full-suite.sh` beside it is the batched full run. It is not a full suite and
# must not be grown into one.
set -u -o pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/../.." && pwd)"
# shellcheck source=gate-lib.sh
. "$HERE/gate-lib.sh"
# shellcheck source=gate-config.sh
. "$HERE/gate-config.sh"

gate_selftest_maybe "${BASH_SOURCE[0]}" "$@"

BASE_REF="" ALLOW_EMPTY=0 DRY=0
while [ $# -gt 0 ]; do
  case "$1" in
    --base)        BASE_REF="$2"; shift 2 ;;
    --allow-empty) ALLOW_EMPTY=1; shift ;;
    --no-lease)    GATE_NO_LEASE=1; shift ;;
    --dry-run)     DRY=1; shift ;;
    --self-test)   shift ;;  # handled above
    -h|--help)     sed -n '2,20p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) gate_die "unknown arg '$1'" ;;
  esac
done

cd "$REPO_ROOT" || gate_die "cannot cd to $REPO_ROOT"
export GATE_LABEL="${GATE_REPO_NAME}-scoped-gate"

DEFAULT="$(gate_default_branch .)"
if [ -z "$BASE_REF" ]; then
  BASE_REF="$(git merge-base HEAD "origin/$DEFAULT" 2>/dev/null \
           || git merge-base HEAD "$DEFAULT" 2>/dev/null \
           || echo HEAD)"
fi

# ── changed set → closure ─────────────────────────────────────────────────────
TRIGGER_ARGS=()
for t in ${GATE_TRIGGERS[@]+"${GATE_TRIGGERS[@]}"}; do TRIGGER_ARGS+=(--trigger "$t"); done
MAP_OUT="$(gate_changed_set "$BASE_REF" | sort -u \
           | python3 "$HERE/crate-map.py" --manifest "$REPO_ROOT/Cargo.toml" \
               ${TRIGGER_ARGS[@]+"${TRIGGER_ARGS[@]}"})" || gate_die "crate-map failed (rc=$?)"
printf '%s\n' "$MAP_OUT" | sed 's/^/gate: /'

read -r -a CLOSURE <<<"$(printf '%s\n' "$MAP_OUT" | awk '/^CLOSURE/{ $1=""; print; exit }')"
SHAPE="$(printf '%s\n' "$MAP_OUT" | awk '/^SHAPE /{ print $2; exit }')"

if [ ${#CLOSURE[@]} -eq 0 ]; then
  case "$SHAPE" in
    docs-only)
      gate_note "NOTE: diff touches only docs/config outside every crate — no Rust gate selected, by design."
      exit 0 ;;
    *)
      echo "gate: ██ WARNING: STRUCTURAL change (add/delete/rename) selected NO gate ██" >&2
      printf '%s\n' "$MAP_OUT" | awk '/^UNMAPPED/{ print "gate:   " $0 }' >&2
      echo "gate: this is the dangerous shape — a new/removed file nothing compiles is how holes ship." >&2
      if [ "$ALLOW_EMPTY" = 1 ]; then
        gate_note "--allow-empty given — continuing as a warning only"
        exit 0
      fi
      exit 3 ;;
  esac
fi

# ── leg list ──────────────────────────────────────────────────────────────────
PKG_ARGS=()
for m in "${CLOSURE[@]}"; do PKG_ARGS+=(-p "$m"); done

if [ "$DRY" = 1 ]; then
  gate_note "dry-run — closure: ${CLOSURE[*]}"
  gate_note "  fmt / clippy / test over: ${CLOSURE[*]}"
  for t in ${GATE_TARGETS[@]+"${GATE_TARGETS[@]}"}; do
    gate_note "  check --target $t over: $(gate_target_pkgs "$t" "${CLOSURE[@]}")"
  done
  gate_note "  extra legs: $(gate_extra_legs_names "${CLOSURE[@]}")"
  exit 0
fi

# The toolchain is needed when a windows target is configured, and ALSO when
# the repo's own .cargo/config.toml already builds windows by default (in
# which case every ordinary leg is a windows build) — video-editor's shape.
NEED_WIN="${GATE_NEEDS_WIN_TOOLCHAIN:-0}"
for t in ${GATE_TARGETS[@]+"${GATE_TARGETS[@]}"}; do
  case "$t" in *windows*) NEED_WIN=1; break ;; esac
done
[ "$NEED_WIN" = 1 ] && { gate_win_toolchain_path || true; }

meter_grab
lease_target_dir

if [ "${GATE_FMT_ENFORCE:-1}" = 1 ]; then
  run_leg fmt cargo fmt --check "${PKG_ARGS[@]}"
else
  run_leg_advisory fmt cargo fmt --check "${PKG_ARGS[@]}"
fi
if [ "${GATE_CLIPPY_ENFORCE:-1}" = 1 ]; then
  # shellcheck disable=SC2086
  run_leg clippy cargo clippy "${PKG_ARGS[@]}" --all-targets --jobs "$GRANT" ${GATE_CLIPPY_ARGS:-}
else
  # shellcheck disable=SC2086
  run_leg_advisory clippy cargo clippy "${PKG_ARGS[@]}" --all-targets --jobs "$GRANT" ${GATE_CLIPPY_ARGS:-}
fi
run_leg test cargo test "${PKG_ARGS[@]}" --no-fail-fast --jobs "$GRANT" -- --test-threads="$GRANT"

for t in ${GATE_TARGETS[@]+"${GATE_TARGETS[@]}"}; do
  read -r -a TPKGS <<<"$(gate_target_pkgs "$t" "${CLOSURE[@]}")"
  TARGS=(); for m in "${TPKGS[@]}"; do TARGS+=(-p "$m"); done
  if [ ${#TARGS[@]} -eq 0 ]; then
    gate_note "target $t: no configured package in or under the closure — skipped (entry package list is empty?)"
    continue
  fi
  read -r -a TXTRA <<<"$(gate_target_extra_args "$t")"
  run_leg "check-$t" cargo check --target "$t" "${TARGS[@]}" ${TXTRA[@]+"${TXTRA[@]}"} --jobs "$GRANT"
done

gate_extra_legs "${CLOSURE[@]}"

gate_finish
