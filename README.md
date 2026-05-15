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

### Linux / macOS (recommended)

```shell
curl -fsSL https://raw.githubusercontent.com/nodecook/agent/main/scripts/install.sh | sudo bash
```

The script downloads the latest binary from `dl.nodecook.com` and installs it as:

- Linux: `systemd` service named `nodecook-agent`
- macOS: `launchd` daemon named `com.nodecook.agent`

You can pass configuration with environment variables:

```shell
curl -fsSL https://raw.githubusercontent.com/nodecook/agent/main/scripts/install.sh | sudo NCA_TITLE="My Node" NCA_LINK="https://example.com" bash
```

### Windows

Run PowerShell as Administrator:

```powershell
iwr https://raw.githubusercontent.com/nodecook/agent/main/scripts/install.ps1 -useb | iex
```

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

Linux / macOS:

```shell
curl -fsSL https://raw.githubusercontent.com/nodecook/agent/main/scripts/uninstall.sh | sudo bash
```

Windows PowerShell as Administrator:

```powershell
iwr https://raw.githubusercontent.com/nodecook/agent/main/scripts/uninstall.ps1 -useb | iex
```

### How can I update the agent?

Run the install script again. It will replace the binary and restart the service. If you use Docker, pull the latest image and run the container again.

### If you have any other questions, please feel free to [open an issue](https://github.com/nodecook/agent/issues/new)
