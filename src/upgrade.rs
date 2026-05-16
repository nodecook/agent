use anyhow::{anyhow, bail, Context, Result};
use flate2::read::GzDecoder;
use rand::Rng;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tar::Archive;
use tracing::{info, warn};

const DOWNLOAD_BASE: &str = "https://dl.nodecook.com";
const STATE_FILE: &str = "/var/lib/nodecook-agent/installed.sha256";
const CHECK_INTERVAL: Duration = Duration::from_secs(3600);
// HTTP 总超时：sha256 文件几十字节，给短超时；tarball 几 MB，给宽超时
const SHA_FETCH_TIMEOUT: Duration = Duration::from_secs(15);
const TARBALL_FETCH_TIMEOUT: Duration = Duration::from_secs(120);
// 防御：dl.nodecook.com 如果错配置返回 HTML 错误页，或被劫持成大文件，限上限
const MAX_TARBALL_BYTES: usize = 50 * 1024 * 1024;

fn detect_target() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Some("x86_64-unknown-linux-musl"),
        ("linux", "aarch64") => Some("aarch64-unknown-linux-musl"),
        _ => None,
    }
}

fn http_client(timeout: Duration) -> Result<Client> {
    Client::builder()
        .timeout(timeout)
        .connect_timeout(Duration::from_secs(10))
        .build()
        .context("build reqwest client")
}

async fn fetch_remote_sha(target: &str) -> Result<String> {
    let url = format!("{}/nodecook-agent-{}.tar.gz.sha256", DOWNLOAD_BASE, target);
    let body = http_client(SHA_FETCH_TIMEOUT)?
        .get(&url)
        .send()
        .await
        .with_context(|| format!("GET {url}"))?
        .error_for_status()
        .with_context(|| format!("status for {url}"))?
        .text()
        .await
        .with_context(|| format!("read body of {url}"))?;
    // sha256sum 输出格式：`<hash>  <filename>`
    let hash = body
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow!("empty sha256 body"))?
        .to_lowercase();
    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("malformed sha256: {hash:?}");
    }
    Ok(hash)
}

fn read_state(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn write_state(path: &Path, sha: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create dir {}", parent.display()))?;
    }
    std::fs::write(path, sha).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

async fn download_and_verify(target: &str, expected_sha: &str) -> Result<Vec<u8>> {
    let url = format!("{}/nodecook-agent-{}.tar.gz", DOWNLOAD_BASE, target);
    let response = http_client(TARBALL_FETCH_TIMEOUT)?
        .get(&url)
        .send()
        .await
        .with_context(|| format!("GET {url}"))?
        .error_for_status()
        .with_context(|| format!("status for {url}"))?;
    // 预检大小，防御误返回 HTML/巨大文件
    if let Some(len) = response.content_length() {
        if len as usize > MAX_TARBALL_BYTES {
            bail!(
                "tarball too large: declared {len} bytes (max {MAX_TARBALL_BYTES})"
            );
        }
    }
    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("read body of {url}"))?;
    if bytes.len() > MAX_TARBALL_BYTES {
        bail!(
            "tarball too large: received {} bytes (max {MAX_TARBALL_BYTES})",
            bytes.len()
        );
    }
    let computed = format!("{:x}", Sha256::digest(&bytes));
    if computed != expected_sha {
        bail!(
            "sha256 mismatch: remote claims {} but downloaded blob is {}",
            expected_sha,
            computed
        );
    }
    Ok(bytes.to_vec())
}

fn extract_binary(tarball: &[u8], target: &str) -> Result<Vec<u8>> {
    let asset_dir = format!("nodecook-agent-{}", target);
    let entry_path = format!("{}/nodecook-agent", asset_dir);
    let mut archive = Archive::new(GzDecoder::new(tarball));
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_string_lossy().to_string();
        if path == entry_path {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            if buf.is_empty() {
                bail!("extracted binary is empty");
            }
            return Ok(buf);
        }
    }
    bail!("binary `{entry_path}` not found in tarball")
}

fn swap_binary(new_bytes: &[u8]) -> Result<PathBuf> {
    let current = std::env::current_exe().context("current_exe")?;
    let parent = current
        .parent()
        .ok_or_else(|| anyhow!("current_exe has no parent"))?;
    let file_name = current
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("current_exe has no filename"))?;
    let tmp = parent.join(format!(".{file_name}.new"));
    std::fs::write(&tmp, new_bytes)
        .with_context(|| format!("write tmp binary {}", tmp.display()))?;
    std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))
        .with_context(|| format!("chmod tmp binary {}", tmp.display()))?;
    // Linux 允许 atomic rename 替换正在运行的 ELF，进程的内存映射不受影响
    std::fs::rename(&tmp, &current)
        .with_context(|| format!("rename {} -> {}", tmp.display(), current.display()))?;
    Ok(current)
}

async fn check_and_upgrade(target: &'static str, state_path: &Path) -> Result<()> {
    let remote_sha = fetch_remote_sha(target).await?;
    match read_state(state_path) {
        None => {
            // 首次启动：信任 installer 装的就是最新版，仅记录当前指纹
            write_state(state_path, &remote_sha)?;
            info!(
                "recorded baseline binary sha: {}…",
                &remote_sha[..remote_sha.len().min(12)]
            );
        }
        Some(local) if local == remote_sha => {
            // 已是最新
        }
        Some(local) => {
            info!(
                "new binary detected (local {}… remote {}…), upgrading...",
                &local[..local.len().min(12)],
                &remote_sha[..remote_sha.len().min(12)]
            );
            let tarball = download_and_verify(target, &remote_sha).await?;
            let binary = extract_binary(&tarball, target)?;
            let path = swap_binary(&binary)?;
            // 状态文件在 binary 替换成功后再写，否则中途失败下次仍会重试
            write_state(state_path, &remote_sha)?;
            info!(
                "upgraded {}; exiting so systemd can restart with the new binary",
                path.display()
            );
            // systemd 配置了 Restart=always，进程退出会被自动拉起
            std::process::exit(0);
        }
    }
    Ok(())
}

pub fn spawn() {
    let Some(target) = detect_target() else {
        warn!(
            "auto upgrade skipped: unsupported platform {}/{}",
            std::env::consts::OS,
            std::env::consts::ARCH
        );
        return;
    };
    let state_path = PathBuf::from(STATE_FILE);
    tokio::spawn(async move {
        // 初次检查前随机延迟 1~60 分钟，避免所有节点同时拉 dl.nodecook.com
        let jitter_secs = rand::thread_rng().gen_range(60..=3600);
        info!(
            "auto upgrade enabled; first check in {}s, then hourly",
            jitter_secs
        );
        tokio::time::sleep(Duration::from_secs(jitter_secs)).await;
        loop {
            // 网络抖动、DNS 失败、SSL 失败等都被吃在 warn 里，
            // 主流程继续，下一小时自动重试。
            if let Err(e) = check_and_upgrade(target, &state_path).await {
                warn!("auto upgrade check failed (will retry next hour): {e:#}");
            }
            tokio::time::sleep(CHECK_INTERVAL).await;
        }
    });
}
