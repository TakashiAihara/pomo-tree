#!/usr/bin/env bash
# pomo-tree installer for macOS (Apple Silicon)
#
#   curl -fsSL https://raw.githubusercontent.com/TakashiAihara/pomo-tree/main/scripts/install.sh | bash
#
# Downloads the latest release .app tarball and installs it into /Applications.

set -euo pipefail

REPO="TakashiAihara/pomo-tree"
ASSET="pomo-tree_macos_aarch64.app.tar.gz"
APP_NAME="pomo-tree.app"
INSTALL_DIR="/Applications"
URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: this installer is for macOS. On Windows, use scripts/install.ps1." >&2
  exit 1
fi

if [[ "$(uname -m)" != "arm64" ]]; then
  echo "error: only Apple Silicon (arm64) builds are published for now." >&2
  echo "       Intel mac users: build from source (see README.md Development)." >&2
  exit 1
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo "Downloading ${ASSET} ..."
curl -fL --progress-bar "$URL" -o "${tmp}/${ASSET}"

echo "Extracting ..."
tar -xzf "${tmp}/${ASSET}" -C "$tmp"

if [[ ! -d "${tmp}/${APP_NAME}" ]]; then
  echo "error: ${APP_NAME} not found in the downloaded archive." >&2
  exit 1
fi

if [[ -d "${INSTALL_DIR}/${APP_NAME}" ]]; then
  echo "Replacing existing ${INSTALL_DIR}/${APP_NAME} ..."
  rm -rf "${INSTALL_DIR}/${APP_NAME}"
fi

mv "${tmp}/${APP_NAME}" "${INSTALL_DIR}/"

# 未署名配布のため quarantine を外す (curl 経由では通常付かないが念のため)
xattr -dr com.apple.quarantine "${INSTALL_DIR}/${APP_NAME}" 2>/dev/null || true

echo
echo "Installed to ${INSTALL_DIR}/${APP_NAME}"
echo "Launch it with: open ${INSTALL_DIR}/${APP_NAME}"
echo "The timer lives in the menu bar (look for the tomato)."
