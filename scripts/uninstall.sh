#!/bin/sh
set -eu

BIN_NAME="nodecook-agent"
INSTALL_DIR="${NODECOOK_AGENT_INSTALL_DIR:-/usr/local/bin}"
ENV_FILE="${NODECOOK_AGENT_ENV_FILE:-/etc/nodecook-agent.env}"
STATE_DIR="${NODECOOK_AGENT_STATE_DIR:-/var/lib/nodecook-agent}"
INITD_DIR="${NODECOOK_AGENT_INITD_DIR:-/etc/init.d}"
SERVICE_NAME="nodecook-agent"

if [ "$(id -u)" -ne 0 ]; then
  echo "Please run as root, for example: sudo $0"
  exit 1
fi

# systemd
if command -v systemctl >/dev/null 2>&1; then
  systemctl disable --now "$SERVICE_NAME" >/dev/null 2>&1 || true
  rm -f "/etc/systemd/system/${SERVICE_NAME}.service"
  systemctl daemon-reload >/dev/null 2>&1 || true
fi

# procd / OpenWRT
if [ -f "${INITD_DIR}/${SERVICE_NAME}" ]; then
  "${INITD_DIR}/${SERVICE_NAME}" stop >/dev/null 2>&1 || true
  "${INITD_DIR}/${SERVICE_NAME}" disable >/dev/null 2>&1 || true
  rm -f "${INITD_DIR}/${SERVICE_NAME}"
fi

rm -f "$INSTALL_DIR/$BIN_NAME"
rm -f "$ENV_FILE"
rm -rf "$STATE_DIR"

echo "NodeCook Agent uninstalled."
