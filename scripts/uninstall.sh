#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="nodecook-agent"
INSTALL_DIR="${NODECOOK_AGENT_INSTALL_DIR:-/usr/local/bin}"
ENV_FILE="${NODECOOK_AGENT_ENV_FILE:-/etc/nodecook-agent.env}"
STATE_DIR="${NODECOOK_AGENT_STATE_DIR:-/var/lib/nodecook-agent}"
SERVICE_NAME="nodecook-agent"

if [ "$(id -u)" -ne 0 ]; then
  echo "Please run as root, for example: sudo $0"
  exit 1
fi

if command -v systemctl >/dev/null 2>&1; then
  systemctl disable --now "$SERVICE_NAME" >/dev/null 2>&1 || true
  rm -f "/etc/systemd/system/${SERVICE_NAME}.service"
  systemctl daemon-reload >/dev/null 2>&1 || true
fi

rm -f "$INSTALL_DIR/$BIN_NAME"
rm -f "$ENV_FILE"
rm -rf "$STATE_DIR"

echo "NodeCook Agent uninstalled."
