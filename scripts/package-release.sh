#!/usr/bin/env bash
set -euo pipefail

# ============================================================================
# LazyCat420 - Video Editor Packaging Script
# Builds release binary with embedded PE resources and bundles into dist/
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${ROOT_DIR}"

echo "==> Building Video Editor (Release with Windows Resources)..."
cargo build --release

DIST_DIR="${ROOT_DIR}/dist"
APP_DIR="${DIST_DIR}/VideoEditor-Portable"

rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}"

EXE_SRC="${ROOT_DIR}/target/x86_64-pc-windows-gnullvm/release/video-editor.exe"
if [ ! -f "${EXE_SRC}" ]; then
    # Fallback to standard release path if not using cross target directory
    EXE_SRC="${ROOT_DIR}/target/release/video-editor.exe"
fi

echo "==> Copying binary: ${EXE_SRC}"
cp "${EXE_SRC}" "${APP_DIR}/video-editor.exe"

echo "==> Copying assets..."
cp -r "${ROOT_DIR}/assets" "${APP_DIR}/"

echo "==> Copying trust certificate and helper scripts..."
cp "${ROOT_DIR}/scripts/trust-cert.bat" "${APP_DIR}/" 2>/dev/null || true
if [ -f "${ROOT_DIR}/scripts/LazyCat420_Root.cer" ]; then
    cp "${ROOT_DIR}/scripts/LazyCat420_Root.cer" "${APP_DIR}/"
fi

echo "==> Creating portable ZIP..."
cd "${DIST_DIR}"
zip -r "VideoEditor-v0.1.0-Windows-Portable.zip" "VideoEditor-Portable"

echo "==> Done! Output available at:"
echo "    ${DIST_DIR}/VideoEditor-v0.1.0-Windows-Portable.zip"
echo "    ${APP_DIR}/"
