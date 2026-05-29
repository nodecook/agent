#!/bin/sh
# POSIX sh so it also runs under BusyBox ash (OpenWRT) without requiring bash.
set -eu
# Enable pipefail only when the shell supports it (bash / recent ash); dash does not.
if ( set -o pipefail ) 2>/dev/null; then set -o pipefail; fi

DOWNLOAD_BASE_URL="${NODECOOK_AGENT_DOWNLOAD_BASE_URL:-https://dl.nodecook.com}"
BIN_NAME="nodecook-agent"
INSTALL_DIR="${NODECOOK_AGENT_INSTALL_DIR:-/usr/local/bin}"
ENV_FILE="${NODECOOK_AGENT_ENV_FILE:-/etc/nodecook-agent.env}"
STATE_DIR="${NODECOOK_AGENT_STATE_DIR:-/var/lib/nodecook-agent}"
STATE_FILE="${STATE_DIR}/installed.sha256"
INITD_DIR="${NODECOOK_AGENT_INITD_DIR:-/etc/init.d}"
SERVICE_NAME="nodecook-agent"

ENV_KEYS="NCA_DEBUG NCA_V4_ONLY NCA_V6_ONLY NCA_V4_SERVER NCA_V6_SERVER NCA_TITLE NCA_LINK"

is_installed() {
  [ -x "$INSTALL_DIR/$BIN_NAME" ]
}

has_env_vars() {
  for key in $ENV_KEYS; do
    eval "value=\${$key:-}"
    [ -n "$value" ] && return 0
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

# Detect the service manager so we install the right kind of service.
detect_init() {
  if [ -d /run/systemd/system ] && command -v systemctl >/dev/null 2>&1; then
    echo systemd
  elif [ -x /sbin/procd ] || command -v procd >/dev/null 2>&1; then
    echo procd
  else
    echo none
  fi
}

target() {
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
  asset="$1"
  echo "${DOWNLOAD_BASE_URL%/}/${asset}.tar.gz"
}

write_env_file() {
  mkdir -p "$(dirname "$ENV_FILE")"
  {
    for key in $ENV_KEYS; do
      write_env_line "$key"
    done
  } > "$ENV_FILE"
}

# Emit KEY="value" for one variable. The quoting round-trips correctly both
# through systemd's EnvironmentFile parser and through `.`-sourcing in sh.
write_env_line() {
  key="$1"
  eval "value=\${$key:-}"
  [ -z "$value" ] && return 0
  value="$(printf '%s' "$value" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g')"
  printf '%s="%s"\n' "$key" "$value"
}

install_binary() {
  target="$(target)"
  asset="${BIN_NAME}-${target}"
  tmp="$(mktemp -d)"
  url="$(download_url "$asset")"
  sha_url="${url}.sha256"

  need_cmd curl
  need_cmd tar

  echo "Downloading $url"
  curl -fsSL "$url" -o "$tmp/${asset}.tar.gz"
  tar -xzf "$tmp/${asset}.tar.gz" -C "$tmp"

  # Atomic swap: write to a temp file in the install dir, then rename over the
  # destination. Renaming (not truncating in place) avoids "Text file busy"
  # when upgrading while the old binary is still running.
  mkdir -p "$INSTALL_DIR"
  new_bin="${INSTALL_DIR}/.${BIN_NAME}.new.$$"
  cp "$tmp/$asset/$BIN_NAME" "$new_bin"
  chmod 0755 "$new_bin"
  mv -f "$new_bin" "$INSTALL_DIR/$BIN_NAME"
  rm -rf "$tmp"

  # 写入 sha 基线，让 agent 启动时不会把"已经是最新版"误判为需要升级
  if sha="$(curl -fsSL "$sha_url" 2>/dev/null | awk '{print $1}')" && [ -n "$sha" ]; then
    mkdir -p "$STATE_DIR"
    printf '%s\n' "$sha" > "$STATE_FILE"
  else
    echo "Warning: failed to fetch $sha_url; agent will record baseline on next start." >&2
  fi
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

# OpenWRT init script. `respawn` mirrors systemd's Restart=always, so the
# agent's self-upgrade (which exits and expects to be restarted) keeps working.
install_procd() {
  mkdir -p "$INITD_DIR"
  init_file="${INITD_DIR}/${SERVICE_NAME}"
  {
    cat <<EOF
#!/bin/sh /etc/rc.common

USE_PROCD=1
START=99
STOP=10

ENV_FILE="${ENV_FILE}"

start_service() {
    [ -f "\$ENV_FILE" ] && . "\$ENV_FILE"
    procd_open_instance
    procd_set_param command "${INSTALL_DIR}/${BIN_NAME}"
EOF
    for key in $ENV_KEYS; do
      printf '    [ -n "${%s:-}" ] && procd_append_param env "%s=$%s"\n' "$key" "$key" "$key"
    done
    cat <<'EOF'
    procd_set_param respawn
    procd_set_param stdout 1
    procd_set_param stderr 1
    procd_close_instance
}
EOF
  } > "$init_file"
  chmod 0755 "$init_file"
  "$init_file" enable
  "$init_file" restart
}

main() {
  need_root
  mkdir -p "$INSTALL_DIR"

  init_sys="$(detect_init)"
  if [ "$init_sys" = "none" ]; then
    echo "No supported service manager found (need systemd or procd/OpenWRT)." >&2
    echo "Install the binary manually and run ${INSTALL_DIR}/${BIN_NAME} under your own supervisor." >&2
    exit 1
  fi

  mode="install"
  if is_installed; then
    mode="upgrade"
    echo "Existing NodeCook Agent detected, upgrading..."
  fi

  install_binary

  if [ "$mode" = "install" ] || has_env_vars; then
    write_env_file
  fi

  case "$init_sys" in
    systemd) install_systemd ;;
    procd) install_procd ;;
  esac

  if [ "$mode" = "upgrade" ]; then
    echo "NodeCook Agent upgraded (${init_sys})."
  else
    echo "NodeCook Agent installed (${init_sys})."
  fi
}

main "$@"
