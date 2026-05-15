# NodeCook Agent

[English](./README.md)

这是 [NodeCook](https://www.nodecook.com) 的代理程序。它负责从 NodeCook 服务器运行 ping、tcping、http 等作业。它是用 Rust 编写的，只需要很少的系统资源。

**重要提示: 任何时候请保持您的 agent 为最新版本!**

## 特性

- **轻量级**：仅需要少量系统资源。
- **快速**：用 Rust 编写，非常快。
- **开源**：所有源代码都是开源的，您无需担心安全性。

## 安装

您可以通过以下方法安装代理。

### 前置依赖

- 防火墙规则允许 Agent 访问服务器。

### Linux / macOS（推荐）

```shell
curl -fsSL https://raw.githubusercontent.com/nodecook/agent/main/scripts/install.sh | sudo bash
```

脚本会从 `dl.nodecook.com` 下载最新二进制，并安装为：

- Linux：名为 `nodecook-agent` 的 `systemd` 服务
- macOS：名为 `com.nodecook.agent` 的 `launchd` 守护进程

可以通过环境变量传入配置：

```shell
curl -fsSL https://raw.githubusercontent.com/nodecook/agent/main/scripts/install.sh | sudo NCA_TITLE="My Node" NCA_LINK="https://example.com" bash
```

### Windows

以管理员身份运行 PowerShell：

```powershell
iwr https://raw.githubusercontent.com/nodecook/agent/main/scripts/install.ps1 -useb | iex
```

### Docker

```shell
docker run -d --user=root --name nodecook-agent --restart=always --network=host ghcr.io/nodecook/agent
```

## 配置

有一些环境变量可以用来配置代理程序。

### NCA_DEBUG

如果设置为 `true`，代理程序将打印调试信息，默认为 `false`。

### NCA_V4_ONLY

如果设置为 `true`，代理程序将只使用 ipv4 访问服务器，默认为 `false`。

### NCA_V6_ONLY

如果设置为 `true`，代理程序将只使用 ipv6 访问服务器，默认为 `false`。

### NCA_TITLE

该节点展示的赞助标题。

### NCA_LINK

该节点展示的赞助链接。

## 故障排除

### 为什么我在仪表板中看不到代理？

请检查代理的状态。如果代理正在运行，您可以检查日志或将 `NCA_DEBUG` 设置为 `true` 以查看调试信息。

### 为什么代理以 root 用户运行？

代理需要访问一些系统资源，例如网络接口，因此需要以 root 用户运行。

### 代理是否从我的服务器收集任何数据？

不，代理只是从服务器运行作业并将结果发送回服务器。它不会从您的服务器收集任何数据。您可以查看源代码确认。

### 代理是否需要大量系统资源？

不，代理是用 Rust 编写的，只需要少量系统资源。

### 如何卸载代理程序？

Linux / macOS：

```shell
curl -fsSL https://raw.githubusercontent.com/nodecook/agent/main/scripts/uninstall.sh | sudo bash
```

Windows 请以管理员身份运行 PowerShell：

```powershell
iwr https://raw.githubusercontent.com/nodecook/agent/main/scripts/uninstall.ps1 -useb | iex
```

### 如何更新代理程序？

重新运行安装脚本即可。它会替换二进制并重启服务。如果您使用 Docker，拉取最新镜像并重新运行容器即可。

### 如果您有任何其他问题，请随时 [打开一个 issue](https://github.com/nodecook/agent/issues/new)
