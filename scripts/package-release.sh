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

echo "==> Copying start-here instructions..."
cp "${ROOT_DIR}/scripts/README-FIRST.txt" "${APP_DIR}/"

echo "==> Bundling FFmpeg..."
# The app shells out to ffmpeg/ffprobe for import, thumbnails, preview and export, so a
# machine without them can do nothing at all. find_ffmpeg_executable() checks <exe>/bin/
# before falling back to PATH, so binaries placed there make the folder self-contained.
FFMPEG_BIN_DIR="${FFMPEG_BIN_DIR:-/mnt/c/FFMPEG/bin}"
if [ -f "${FFMPEG_BIN_DIR}/ffmpeg.exe" ] && [ -f "${FFMPEG_BIN_DIR}/ffprobe.exe" ]; then
    mkdir -p "${APP_DIR}/bin"
    cp "${FFMPEG_BIN_DIR}/ffmpeg.exe" "${APP_DIR}/bin/"
    cp "${FFMPEG_BIN_DIR}/ffprobe.exe" "${APP_DIR}/bin/"
    echo "    bundled from ${FFMPEG_BIN_DIR}"
else
    echo "    !! WARNING: no ffmpeg.exe/ffprobe.exe in ${FFMPEG_BIN_DIR}"
    echo "    !! This package will NOT work on a machine without FFmpeg on PATH."
    echo "    !! Set FFMPEG_BIN_DIR to a directory holding both, then re-run."
fi

# trust-cert.bat asks for Administrator and then installs LazyCat420_Root.cer. Shipping it
# without that file gives the recipient an admin prompt that can only fail, so both travel
# together or neither does.
if [ -f "${ROOT_DIR}/scripts/LazyCat420_Root.cer" ]; then
    echo "==> Copying trust certificate and helper scripts..."
    cp "${ROOT_DIR}/scripts/trust-cert.bat" "${APP_DIR}/"
    cp "${ROOT_DIR}/scripts/LazyCat420_Root.cer" "${APP_DIR}/"
else
    echo "==> Skipping trust-cert.bat (no LazyCat420_Root.cer to install)"
fi

echo "==> Creating portable ZIP..."
cd "${DIST_DIR}"
python3 -m zipfile -c "VideoEditor-v0.1.0-Windows-Portable.zip" "VideoEditor-Portable"

echo "==> Done! Output available at:"
echo "    ${DIST_DIR}/VideoEditor-v0.1.0-Windows-Portable.zip"
echo "    ${APP_DIR}/"
