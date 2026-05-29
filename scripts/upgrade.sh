#!/bin/sh
set -eu
if ( set -o pipefail ) 2>/dev/null; then set -o pipefail; fi

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

short_sha() {
  printf '%s' "$1" | cut -c1-12
}

# Restart the service through whichever supervisor currently manages it.
restart_service() {
  if command -v systemctl >/dev/null 2>&1 \
    && systemctl is-active --quiet "$SERVICE_NAME"; then
    systemctl restart "$SERVICE_NAME"
    return 0
  fi
  if [ -x "/etc/init.d/${SERVICE_NAME}" ]; then
    "/etc/init.d/${SERVICE_NAME}" restart
    return 0
  fi
  return 1
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
    echo "Already up to date (sha $(short_sha "$remote_sha")...)."
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

  # Atomic swap to avoid "Text file busy" while the old binary runs.
  new_bin="${INSTALL_DIR}/.${BIN_NAME}.new.$$"
  cp "$tmp/$asset/$BIN_NAME" "$new_bin"
  chmod 0755 "$new_bin"
  mv -f "$new_bin" "$INSTALL_DIR/$BIN_NAME"

  mkdir -p "$STATE_DIR"
  printf '%s\n' "$remote_sha" > "$STATE_FILE"

  if restart_service; then
    echo "NodeCook Agent upgraded (sha $(short_sha "$remote_sha")...) and service restarted."
  else
    echo "NodeCook Agent binary upgraded (sha $(short_sha "$remote_sha")...); restart the service manually to apply."
  fi
}

main "$@"
