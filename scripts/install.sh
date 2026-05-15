#!/usr/bin/env bash
set -euo pipefail

DOWNLOAD_BASE_URL="${NODECOOK_AGENT_DOWNLOAD_BASE_URL:-https://dl.nodecook.com}"
BIN_NAME="nodecook-agent"
INSTALL_DIR="${NODECOOK_AGENT_INSTALL_DIR:-/usr/local/bin}"
ENV_FILE="${NODECOOK_AGENT_ENV_FILE:-/etc/nodecook-agent.env}"
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
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os:$arch" in
    Linux:x86_64|Linux:amd64) echo "x86_64-unknown-linux-musl" ;;
    Linux:aarch64|Linux:arm64) echo "aarch64-unknown-linux-musl" ;;
    Darwin:x86_64) echo "x86_64-apple-darwin" ;;
    Darwin:arm64|Darwin:aarch64) echo "aarch64-apple-darwin" ;;
    *)
      echo "Unsupported platform: $os $arch" >&2
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
  mkdir -p "$(dirname "$ENV_FILE")"
  {
    write_systemd_env NCA_DEBUG
    write_systemd_env NCA_V4_ONLY
    write_systemd_env NCA_V6_ONLY
    write_systemd_env NCA_V4_SERVER
    write_systemd_env NCA_V6_SERVER
    write_systemd_env NCA_V4_NODE_ID
    write_systemd_env NCA_V6_NODE_ID
    write_systemd_env NCA_TITLE
    write_systemd_env NCA_LINK
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
  systemctl enable --now "$SERVICE_NAME"
}

plist_env() {
  local key value
  for key in NCA_DEBUG NCA_V4_ONLY NCA_V6_ONLY NCA_V4_SERVER NCA_V6_SERVER NCA_V4_NODE_ID NCA_V6_NODE_ID NCA_TITLE NCA_LINK; do
    value="${!key:-}"
    [ "$value" = "" ] && continue
    value="$(xml_escape "$value")"
    printf '    <key>%s</key>\n    <string>%s</string>\n' "$key" "$value"
  done
}

xml_escape() {
  local value="$1"
  value="${value//&/&amp;}"
  value="${value//</&lt;}"
  value="${value//>/&gt;}"
  value="${value//\"/&quot;}"
  value="${value//\'/&apos;}"
  printf '%s' "$value"
}

install_launchd() {
  local plist="/Library/LaunchDaemons/com.nodecook.agent.plist"
  {
    cat <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.nodecook.agent</string>
  <key>ProgramArguments</key>
  <array>
    <string>${INSTALL_DIR}/${BIN_NAME}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
EOF
    if [ -n "${NCA_DEBUG:-}${NCA_V4_ONLY:-}${NCA_V6_ONLY:-}${NCA_V4_SERVER:-}${NCA_V6_SERVER:-}${NCA_V4_NODE_ID:-}${NCA_V6_NODE_ID:-}${NCA_TITLE:-}${NCA_LINK:-}" ]; then
      echo "  <key>EnvironmentVariables</key>"
      echo "  <dict>"
      plist_env
      echo "  </dict>"
    fi
    cat <<EOF
</dict>
</plist>
EOF
  } > "$plist"
  chown root:wheel "$plist"
  chmod 0644 "$plist"
  launchctl bootout system "$plist" >/dev/null 2>&1 || true
  launchctl bootstrap system "$plist"
  launchctl enable system/com.nodecook.agent
}

main() {
  need_root
  mkdir -p "$INSTALL_DIR"
  install_binary

  case "$(uname -s)" in
    Linux)
      need_cmd systemctl
      write_env_file
      install_systemd
      ;;
    Darwin)
      install_launchd
      ;;
  esac

  echo "NodeCook Agent installed."
}

main "$@"
