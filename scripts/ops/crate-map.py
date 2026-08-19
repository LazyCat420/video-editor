#!/usr/bin/env python3
"""crate-map.py — changed files → workspace crates → reverse-dependency closure.

Vendored identically into every Rust repo under sun/ (GATE_LIB_VERSION lives in
gate-lib.sh; keep the copies in step). Stdlib only — this box has no jq.

    crate-map.py --manifest <repo>/Cargo.toml [--trigger GLOB=member ...]

Reads "STATUS<TAB>PATH" records on stdin (repo-relative paths; R rows carry one
path each, renames having been expanded upstream). Writes line-oriented facts
for bash to consume:

    SEED <member>              a directly-touched crate (one line per seed)
    FULL <reason>              closure widened to every member; reason printed
    CLOSURE <m1> <m2> ...      the reverse-dependency closure (may be empty)
    UNMAPPED <status> <path>   changed file owned by no crate
    SHAPE code-mapped|docs-only|structural-unmapped

Exit 10 when cargo metadata itself fails — a broken Cargo.toml is a red gate,
never a skip.
"""
import fnmatch
import json
import os
import subprocess
import sys

FULL_TRIGGER_FILES = {
    "Cargo.toml", "Cargo.lock", "rust-toolchain", "rust-toolchain.toml",
    "Trunk.toml", "build.rs",
}


def main() -> int:
    manifest = None
    triggers = []  # (glob, member)
    args = sys.argv[1:]
    i = 0
    while i < len(args):
        if args[i] == "--manifest":
            manifest = args[i + 1]; i += 2
        elif args[i] == "--trigger":
            glob, _, member = args[i + 1].partition("=")
            triggers.append((glob, member)); i += 2
        else:
            print(f"crate-map: unknown arg {args[i]!r}", file=sys.stderr)
            return 64
    if not manifest:
        print("crate-map: --manifest is required", file=sys.stderr)
        return 64

    try:
        out = subprocess.run(
            ["cargo", "metadata", "--format-version", "1", "--no-deps",
             "--manifest-path", manifest],
            capture_output=True, text=True, check=True,
        ).stdout
        meta = json.loads(out)
    except (subprocess.CalledProcessError, json.JSONDecodeError, OSError) as e:
        detail = getattr(e, "stderr", "") or str(e)
        print(f"crate-map: cargo metadata FAILED — a broken manifest is a red "
              f"gate, not a skip:\n{detail}", file=sys.stderr)
        return 10

    root = os.path.dirname(os.path.abspath(manifest))
    members = {}  # name -> crate root dir, repo-relative ('' for a root crate)
    redges = {}   # member -> set of members that depend on it
    for pkg in meta["packages"]:
        rel = os.path.relpath(os.path.dirname(pkg["manifest_path"]), root)
        members[pkg["name"]] = "" if rel == "." else rel
    for pkg in meta["packages"]:
        for dep in pkg.get("dependencies", []):
            if dep["name"] in members:
                redges.setdefault(dep["name"], set()).add(pkg["name"])

    # longest-prefix wins, so a nested crate beats its parent dir
    roots_by_len = sorted(members.items(), key=lambda kv: -len(kv[1]))

    def owner(path: str):
        for name, crate_root in roots_by_len:
            if crate_root == "":
                continue  # a root crate owns leftovers only via FULL/unmapped rules
            if path == crate_root or path.startswith(crate_root + "/"):
                return name
        # single-crate repo: the root crate owns src/, tests/, benches/, build.rs
        for name, crate_root in members.items():
            if crate_root == "" and (
                path.startswith(("src/", "tests/", "benches/", "examples/"))
                or path == "build.rs"
            ):
                return name
        return None

    seeds, unmapped, full_reasons = set(), [], []
    for raw in sys.stdin:
        raw = raw.rstrip("\n")
        if not raw:
            continue
        status, _, path = raw.partition("\t")
        if not path:
            status, path = "M", raw
        path = path.strip().strip('"')
        if path in FULL_TRIGGER_FILES or path.startswith(".cargo/"):
            full_reasons.append(f"workspace-level file changed: {path}")
            continue
        hit = owner(path)
        if hit is None:
            for glob, member in triggers:
                if fnmatch.fnmatch(path, glob) and member in members:
                    hit = member
                    break
        if hit is not None:
            seeds.add(hit)
            continue
        # unmapped: code-shaped files force the full closure — this is exactly
        # the "new crate dir not yet in [workspace.members]" hole
        base = os.path.basename(path)
        if path.endswith(".rs") or base in ("Cargo.toml", "build.rs") \
                or path.startswith("crates/"):
            full_reasons.append(f"unmapped code file: {path}")
        else:
            unmapped.append((status, path))

    if full_reasons:
        closure = set(members)
        print(f"FULL {full_reasons[0]}"
              + (f" (+{len(full_reasons) - 1} more)" if len(full_reasons) > 1 else ""))
    else:
        closure = set()
        frontier = list(seeds)
        while frontier:
            m = frontier.pop()
            if m in closure:
                continue
            closure.add(m)
            frontier.extend(redges.get(m, ()))

    for s in sorted(seeds):
        print(f"SEED {s}")
    print("CLOSURE " + " ".join(sorted(closure)) if closure else "CLOSURE")
    for status, path in unmapped:
        print(f"UNMAPPED {status} {path}")

    # Shape classification. The warning exists for a STRUCTURAL change that no
    # gate would notice — a file added or deleted that nothing compiles. But an
    # added .md is not that: nothing compiling a document is the correct and
    # expected state, and shouting about it is a false red. A false red costs a
    # gate its authority exactly as fast as a false green does, so documents
    # and assets are classified as docs-only even when added or deleted.
    # Anything else structural (a new .sh, .py, an asset the build consumes, an
    # unknown extension) still warrants the banner.
    def is_doc_like(path: str) -> bool:
        if path.startswith(("documentation/", "docs/", "reports/", ".agents/")):
            return True
        return os.path.splitext(path)[1].lower() in {
            ".md", ".markdown", ".rst", ".txt", ".html", ".css",
            ".svg", ".png", ".jpg", ".jpeg", ".gif", ".webp", ".ico",
        }

    if closure:
        shape = "code-mapped"
    elif any(s in ("A", "D", "R", "?") and not is_doc_like(p) for s, p in unmapped):
        shape = "structural-unmapped"
    else:
        shape = "docs-only"
    print(f"SHAPE {shape}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
