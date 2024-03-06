# NodeCook Agent

[简体中文](./README-zh.md)

This is the agent program of [NodeCook](https://www.nodecook.com). It is responsible for run the jobs like ping, tcping, http, etc from the NodeCook server. Which is written in Rust and need only a few system resources.

## Features

- **Lightweight**: Only a few system resources are needed.
- **Fast**: Written in Rust, it is very fast.
- **Secure**: It is safe to run the agent on your server and protect by api key.
- **Open Source**: All source code is open source and you don't need to worry about the security.

## Installation

You can install the agent by the following methods.

First of all, you need to get the **api key** from the [NodeCook](https://www.nodecook.com/dashboard/apikey) website.

### Prerequisites

- Docker or Docker Compose installed.
- Firewall rules to allow Agent to access the server.

### docker compose (recommended)

```shell
wget https://raw.githubusercontent.com/nodecook/agent/main/compose.yaml -O compose.yaml
export NCA_API_KEY=your_api_key
docker compose up -d
```

### docker

```shell
docker run -d --user=root --name nodecook-agent -e NCA_API_KEY=your_api_key --restart=always --network=host ghcr.io/nodecook/agent
```

## Configuration

There are some environment variables you can use to configure the agent.

### NCA_API_KEY

The api key you get from the [NodeCook](https://www.nodecook.com/dashboard/apikey) website.

### NCA_DEBUG

If set to `true`, the agent will print debug information. Default is `false`.

### NCA_V4_ONLY

If set to `true`, the agent will only use ipv4 to access the server. Default is `false`.

### NCA_V6_ONLY

If set to `true`, the agent will only use ipv6 to access the server. Default is `false`.

## Trubleshooting

### Why I can't see the agent in the dashboard?

Please check your api key and the agent's status. If the agent is running, you can check the logs or set `NCA_DEBUG` to `true` to see the debug information.

### Why the agent run with root user?

The agent requires access to some system resources, such as network interfaces, and therefore needs to be run as root.

### Does the agent collect any data from my server?

No, the agent only run the jobs from the server and send the result back to the server. It doesn't collect any data from your server. You can check the source code to make sure.

### Does the agent need a lot of system resources?

No, the agent is written in Rust and only need a few system resources.

### How can I uninstall the agent?

Just stop the container and remove it.

### How can I update the agent?

If you use `docker compose`, you can just run `docker compose up -d --pull always` to update the agent. If you use docker, you can just pull the latest image and run the container again.

### If you have any other questions, please feel free to [open an issue](https://github.com/nodecook/agent/issues/new)
