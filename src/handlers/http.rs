use crate::dns_resolve;
use crate::errors::SocketIOError;
use crate::utils::is_ip;
use rust_socketio::{asynchronous::Client as SocketClient, Payload};
use serde_json::{json, Value};
use std::net::IpAddr;
use std::net::SocketAddr;
use tracing::debug;
use tracing::error;
use url::Url;

pub async fn http(payload: Payload, socket: SocketClient) -> Result<(), rust_socketio::Error> {
    let data: Value = match payload {
        Payload::String(data) => serde_json::from_str(&data).unwrap(),
        Payload::Binary(data) => serde_json::from_slice(&data).unwrap(),
    };
    debug!("receive http request: {}", data);
    let job_id = data["job_id"].as_str().unwrap();
    let node_id = data["node_id"].as_u64().unwrap();
    let event = "http";
    let url = data["url"].as_str().unwrap();
    let ns = data["ns"].as_str();
    let is_ipv4 = data["is_ipv4"].as_bool().unwrap_or(true);
    let record_type = if is_ipv4 { "A" } else { "AAAA" };
    let parsed_url = Url::parse(url).unwrap();
    let host = parsed_url.host_str().unwrap();
    let port = parsed_url.port().unwrap_or(80);
    let start = std::time::Instant::now();
    let ip = if is_ip(host) {
        host.to_string()
    } else {
        let res = dns_resolve::resolve(host, record_type, ns).await;
        if res.is_none() {
            socket
                .emit(
                    event,
                    json!({
                        "job_id": job_id,
                        "node_id": node_id,
                        "error": SocketIOError::ErrDNSLookupFailed
                    }),
                )
                .await?;
            return Ok(());
        }
        res.unwrap()
            .iter()
            .filter(|ip| is_ip(&ip.to_string()))
            .next()
            .unwrap()
            .to_string()
    };
    let dns_duration = start.elapsed().as_millis();
    let client: reqwest::Client;
    if is_ipv4 {
        let localhost_v4 = IpAddr::V4("0.0.0.0".parse().unwrap());
        client = reqwest::Client::builder()
            .local_address(localhost_v4)
            .timeout(std::time::Duration::from_secs(10))
            .resolve(host, SocketAddr::new(ip.parse().unwrap(), port))
            .build()
            .unwrap();
    } else {
        let localhost_v6 = IpAddr::V6("::".parse().unwrap());
        client = reqwest::Client::builder()
            .local_address(localhost_v6)
            .timeout(std::time::Duration::from_secs(10))
            .resolve(host, SocketAddr::new(ip.parse().unwrap(), port))
            .build()
            .unwrap();
    }
    let res = client.get(url).send().await;
    match res {
        Ok(res) => {
            let status = res.status().as_u16();
            socket
                .emit(
                    event,
                    json!({
                        "job_id": job_id,
                        "node_id": node_id,
                        "duration": start.elapsed().as_millis(),
                        "ip": ip,
                        "dns_duration": dns_duration,
                        "status": status,
                    }),
                )
                .await?;
        }
        Err(e) => {
            error!("http {} failed: {}", url, e);
            socket
                .emit(
                    event,
                    json!({
                        "job_id": job_id,
                        "node_id": node_id,
                        "duration": start.elapsed().as_millis(),
                        "dns_duration": dns_duration,
                        "ip": ip,
                        "error": SocketIOError::ErrHTTPFailed,
                    }),
                )
                .await?;
        }
    }
    Ok(())
}
