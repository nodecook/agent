#!/usr/bin/env bash
set -euo pipefail

DOWNLOAD_BASE_URL="${NODECOOK_AGENT_DOWNLOAD_BASE_URL:-https://dl.nodecook.com}"
BIN_NAME="nodecook-agent"
INSTALL_DIR="${NODECOOK_AGENT_INSTALL_DIR:-/usr/local/bin}"
STATE_DIR="${NODECOOK_AGENT_STATE_DIR:-/var/lib/nodecook-agent}"
STATE_FILE="${STATE_DIR}/installed.sha256"
SERVICE_NAME="nodecook-agent"

need_root() {
  if [ "$(id -u)" -ne 0 ]; then
    echo "Please run as root, for example: sudo $0"
    exit 1
  fi
}

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1"
    exit 1
  fi
}

target() {
  local arch
  arch="$(uname -m)"
  case "$arch" in
    x86_64|amd64) echo "x86_64-unknown-linux-musl" ;;
    aarch64|arm64) echo "aarch64-unknown-linux-musl" ;;
    *)
      echo "Unsupported architecture: $arch" >&2
      exit 1
      ;;
  esac
}

main() {
  need_root
  need_cmd curl
  need_cmd tar
  need_cmd sha256sum
  need_cmd awk

  if [ ! -x "$INSTALL_DIR/$BIN_NAME" ]; then
    echo "$BIN_NAME is not installed at $INSTALL_DIR/$BIN_NAME; run install.sh first." >&2
    exit 1
  fi

  local target_id asset url sha_url remote_sha local_sha tmp actual_sha
  target_id="$(target)"
  asset="${BIN_NAME}-${target_id}"
  url="${DOWNLOAD_BASE_URL%/}/${asset}.tar.gz"
  sha_url="${url}.sha256"

  remote_sha="$(curl -fsSL "$sha_url" | awk '{print $1}')"
  if [ -z "${remote_sha:-}" ]; then
    echo "Failed to fetch remote sha256 from $sha_url" >&2
    exit 1
  fi

  local_sha=""
  if [ -f "$STATE_FILE" ]; then
    local_sha="$(awk '{print $1}' "$STATE_FILE")"
  fi

  if [ "$remote_sha" = "$local_sha" ]; then
    echo "Already up to date (sha ${remote_sha:0:12}...)."
    exit 0
  fi

  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT

  echo "Downloading $url"
  curl -fsSL "$url" -o "$tmp/${asset}.tar.gz"

  actual_sha="$(sha256sum "$tmp/${asset}.tar.gz" | awk '{print $1}')"
  if [ "$actual_sha" != "$remote_sha" ]; then
    echo "sha256 mismatch: expected $remote_sha got $actual_sha" >&2
    exit 1
  fi

  tar -xzf "$tmp/${asset}.tar.gz" -C "$tmp"
  install -m 0755 "$tmp/$asset/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"

  mkdir -p "$STATE_DIR"
  printf '%s\n' "$remote_sha" > "$STATE_FILE"

  if command -v systemctl >/dev/null 2>&1 \
    && systemctl is-active --quiet "$SERVICE_NAME"; then
    systemctl restart "$SERVICE_NAME"
    echo "NodeCook Agent upgraded (sha ${remote_sha:0:12}...) and service restarted."
  else
    echo "NodeCook Agent binary upgraded (sha ${remote_sha:0:12}...); restart the service manually to apply."
  fi
}

main "$@"
