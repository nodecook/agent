#!/usr/bin/env bash
set -euo pipefail

DOWNLOAD_BASE_URL="${NODECOOK_AGENT_DOWNLOAD_BASE_URL:-https://dl.nodecook.com}"
BIN_NAME="nodecook-agent"
INSTALL_DIR="${NODECOOK_AGENT_INSTALL_DIR:-/usr/local/bin}"
ENV_FILE="${NODECOOK_AGENT_ENV_FILE:-/etc/nodecook-agent.env}"
SERVICE_NAME="nodecook-agent"

ENV_KEYS=(NCA_DEBUG NCA_V4_ONLY NCA_V6_ONLY NCA_V4_SERVER NCA_V6_SERVER NCA_TITLE NCA_LINK)

is_installed() {
  [ -x "$INSTALL_DIR/$BIN_NAME" ]
}

has_env_vars() {
  local key
  for key in "${ENV_KEYS[@]}"; do
    [ -n "${!key:-}" ] && return 0
  done
  return 1
}

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
      echo "Use Docker instead: docker run -d --user=root --name nodecook-agent --restart=always --network=host ghcr.io/nodecook/agent" >&2
      exit 1
      ;;
  esac
}

download_url() {
  local asset="$1"
  echo "${DOWNLOAD_BASE_URL%/}/${asset}.tar.gz"
}

write_env_file() {
  local key
  mkdir -p "$(dirname "$ENV_FILE")"
  {
    for key in "${ENV_KEYS[@]}"; do
      write_systemd_env "$key"
    done
  } > "$ENV_FILE"
}

write_systemd_env() {
  local key="$1"
  local value="${!key:-}"
  [ "$value" = "" ] && return
  value="${value//\\/\\\\}"
  value="${value//\"/\\\"}"
  printf '%s="%s"\n' "$key" "$value"
}

install_binary() {
  local target asset tmp url
  target="$(target)"
  asset="${BIN_NAME}-${target}"
  tmp="$(mktemp -d)"
  url="$(download_url "$asset")"

  need_cmd curl
  need_cmd tar

  echo "Downloading $url"
  curl -fsSL "$url" -o "$tmp/${asset}.tar.gz"
  tar -xzf "$tmp/${asset}.tar.gz" -C "$tmp"
  install -m 0755 "$tmp/$asset/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
  rm -rf "$tmp"
}

install_systemd() {
  cat > "/etc/systemd/system/${SERVICE_NAME}.service" <<EOF
[Unit]
Description=NodeCook Agent
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
EnvironmentFile=-${ENV_FILE}
ExecStart=${INSTALL_DIR}/${BIN_NAME}
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
  systemctl daemon-reload
  systemctl enable "$SERVICE_NAME"
  systemctl restart "$SERVICE_NAME"
}

main() {
  need_root
  need_cmd systemctl
  mkdir -p "$INSTALL_DIR"

  local mode="install"
  if is_installed; then
    mode="upgrade"
    echo "Existing NodeCook Agent detected, upgrading..."
  fi

  install_binary

  if [ "$mode" = "install" ] || has_env_vars; then
    write_env_file
  fi
  install_systemd

  if [ "$mode" = "upgrade" ]; then
    echo "NodeCook Agent upgraded."
  else
    echo "NodeCook Agent installed."
  fi
}

main "$@"
