#!/usr/bin/env bash
set -euo pipefail
EXE="$1"; shift
exec "$EXE" "$@"
