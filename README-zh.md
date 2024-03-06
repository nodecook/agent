# NodeCook Agent

[English](./README.md)

这是 [NodeCook](https://www.nodecook.com) 的代理程序。它负责从 NodeCook 服务器运行 ping、tcping、http 等作业。它是用 Rust 编写的，只需要很少的系统资源。

## 特性

- **轻量级**：仅需要少量系统资源。
- **快速**：用 Rust 编写，非常快。
- **安全**：在您的服务器上运行代理并通过 api 密钥进行保护是安全的。
- **开源**：所有源代码都是开源的，您无需担心安全性。

## 安装

您可以通过以下方法安装代理。

首先，您需要从 [NodeCook](https://www.nodecook.com/dashboard/apikey) 网站获取 **api 密钥**。

### 前置依赖

- 安装了 Docker 或 Docker Compose。
- 防火墙规则允许 Agent 访问服务器。

### docker compose（推荐）

```shell
wget https://raw.githubusercontent.com/nodecook/agent/main/compose.yaml -O compose.yaml
export NCA_API_KEY=your_api_key
docker compose up -d
```

### docker

```shell
docker run -d --user=root --name nodecook-agent -e NCA_API_KEY=your_api_key --restart=always --network=host ghcr.io/nodecook/agent
```

## 配置

有一些环境变量可以用来配置代理程序。

### NCA_API_KEY

您从 [NodeCook](https://www.nodecook.com/dashboard/apikey) 网站获取的 api 密钥。

### NCA_DEBUG

如果设置为 `true`，代理程序将打印调试信息，默认为 `false`。

### NCA_V4_ONLY

如果设置为 `true`，代理程序将只使用 ipv4 访问服务器，默认为 `false`。

### NCA_V6_ONLY

如果设置为 `true`，代理程序将只使用 ipv6 访问服务器，默认为 `false`。

## 故障排除

### 为什么我在仪表板中看不到代理？

请检查您的 api 密钥和代理的状态。如果代理正在运行，您可以检查日志或将 `NCA_DEBUG` 设置为 `true` 以查看调试信息。

### 为什么代理以 root 用户运行？

代理需要访问一些系统资源，例如网络接口，因此需要以 root 用户运行。

### 代理是否从我的服务器收集任何数据？

不，代理只是从服务器运行作业并将结果发送回服务器。它不会从您的服务器收集任何数据。您可以查看源代码确认。

### 代理是否需要大量系统资源？

不，代理是用 Rust 编写的，只需要少量系统资源。

### 如何卸载代理程序？

只需要停止并且删除容器即可。

### 如何更新代理程序？

如果您使用 `docker compose`，则只需运行 `docker compose up -d --pull always` 即可更新代理。如果您使用 docker，您只需拉取最新的镜像并再次运行容器即可。

### 如果您有任何其他问题，请随时 [打开一个 issue](https://github.com/nodecook/agent/issues/new)
