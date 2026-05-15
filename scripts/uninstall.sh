#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="nodecook-agent"
INSTALL_DIR="${NODECOOK_AGENT_INSTALL_DIR:-/usr/local/bin}"
ENV_FILE="${NODECOOK_AGENT_ENV_FILE:-/etc/nodecook-agent.env}"
SERVICE_NAME="nodecook-agent"

if [ "$(id -u)" -ne 0 ]; then
  echo "Please run as root, for example: sudo $0"
  exit 1
fi

case "$(uname -s)" in
  Linux)
    if command -v systemctl >/dev/null 2>&1; then
      systemctl disable --now "$SERVICE_NAME" >/dev/null 2>&1 || true
      rm -f "/etc/systemd/system/${SERVICE_NAME}.service"
      systemctl daemon-reload >/dev/null 2>&1 || true
    fi
    ;;
  Darwin)
    plist="/Library/LaunchDaemons/com.nodecook.agent.plist"
    launchctl bootout system "$plist" >/dev/null 2>&1 || true
    rm -f "$plist"
    ;;
  *)
    echo "Unsupported platform: $(uname -s)"
    exit 1
    ;;
esac

rm -f "$INSTALL_DIR/$BIN_NAME"
rm -f "$ENV_FILE"

echo "NodeCook Agent uninstalled."
