# NodeCook Agent

[简体中文](./README-zh.md)

This is the agent program of [NodeCook](https://www.nodecook.com). It is responsible for run the jobs like ping, tcping, http, etc from the NodeCook server. Which is written in Rust and need only a few system resources.

**Important: Please keep you agent up to date anytime!**

## Features

- **Lightweight**: Only a few system resources are needed.
- **Fast**: Written in Rust, it is very fast.
- **Open Source**: All source code is open source and you don't need to worry about the security.

## Installation

You can install the agent by the following methods.

### Prerequisites

- Firewall rules to allow Agent to access the server.

### Linux (recommended)

```shell
curl -fsSL https://raw.githubusercontent.com/nodecook/agent/main/scripts/install.sh | sudo bash
```

The script downloads the latest binary from `dl.nodecook.com` and installs it as a service named `nodecook-agent`. It auto-detects the service manager: `systemd` on most distributions, or `procd` on OpenWRT (installing `/etc/init.d/nodecook-agent`). Re-running the same command upgrades the existing installation in place: the binary is replaced and the service is restarted, while the existing environment file is preserved unless you pass new `NCA_*` variables.

You can pass configuration with environment variables:

```shell
curl -fsSL https://raw.githubusercontent.com/nodecook/agent/main/scripts/install.sh | sudo NCA_TITLE="My Node" NCA_LINK="https://example.com" bash
```

### OpenWRT

OpenWRT uses BusyBox (no `bash`/`sudo` by default) and `procd` instead of `systemd`. Run as root with `sh`:

```shell
curl -fsSL https://raw.githubusercontent.com/nodecook/agent/main/scripts/install.sh | sh
```

The script detects `procd`, installs `/etc/init.d/nodecook-agent`, and enables it on boot. Logs go to the system log — view them with `logread -e nodecook-agent`. The musl binary is published for `x86_64` and `aarch64`; on other router architectures use Docker.

### Docker

```shell
docker run -d --user=root --name nodecook-agent --restart=always --network=host ghcr.io/nodecook/agent
```

## Configuration

There are some environment variables you can use to configure the agent.

### NCA_DEBUG

If set to `true`, the agent will print debug information. Default is `false`.

### NCA_V4_ONLY

If set to `true`, the agent will only use ipv4 to access the server. Default is `false`.

### NCA_V6_ONLY

If set to `true`, the agent will only use ipv6 to access the server. Default is `false`.

### NCA_TITLE

The sponsor title displayed for this node.

### NCA_LINK

The sponsor link displayed for this node.

## Auto upgrade

The agent automatically keeps itself up to date — no configuration
required. After a random 1–60 minute delay at startup (to spread fleet
load) it checks `dl.nodecook.com` every hour, and when a new binary is
published it downloads the tarball, verifies the sha256, atomically
swaps the binary in place, and exits so the service manager (`systemd`
`Restart=always` or `procd` `respawn`) restarts the service.
State is persisted under `/var/lib/nodecook-agent/installed.sha256`.
Network errors are logged and the next check is retried at the regular
hourly interval; a failed download never affects the running agent.

## Trubleshooting

### Why I can't see the agent in the dashboard?

Please check the agent's status. If the agent is running, you can check the logs or set `NCA_DEBUG` to `true` to see the debug information.

### Why the agent run with root user?

The agent requires access to some system resources, such as network interfaces, and therefore needs to be run as root.

### Does the agent collect any data from my server?

No, the agent only run the jobs from the server and send the result back to the server. It doesn't collect any data from your server. You can check the source code to make sure.

### Does the agent need a lot of system resources?

No, the agent is written in Rust and only need a few system resources.

### How can I uninstall the agent?

```shell
curl -fsSL https://raw.githubusercontent.com/nodecook/agent/main/scripts/uninstall.sh | sudo bash
```

### How can I update the agent?

Run the install script again. It will replace the binary and restart the service while preserving your environment file. If you use Docker, pull the latest image and run the container again.

### If you have any other questions, please feel free to [open an issue](https://github.com/nodecook/agent/issues/new)
