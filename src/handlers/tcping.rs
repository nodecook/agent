use crate::dns_resolve;
use crate::errors::SocketIOError;
use crate::utils::is_ip;
use rust_socketio::{asynchronous::Client as SocketClient, Payload};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::net;
use tokio::time;
use tracing::debug;
use tracing::error;

pub async fn tcping(payload: Payload, socket: SocketClient) -> Result<(), rust_socketio::Error> {
    let data: Value = match payload {
        Payload::String(data) => serde_json::from_str(&data).unwrap(),
        Payload::Binary(data) => serde_json::from_slice(&data).unwrap(),
    };
    debug!("receive tcping request: {}", data);
    let job_id = data["job_id"].as_str().unwrap();
    let node_id = data["node_id"].as_u64().unwrap();
    let event = "tcping";
    let host = data["host"].as_str().unwrap();
    let domain = host.split(":").next().unwrap();
    let single = data["single"].as_bool().unwrap_or(true);
    let is_ipv4 = data["is_ipv4"].as_bool().unwrap_or(true);
    let ns: Option<&str> = data["ns"].as_str();
    let record_type = if is_ipv4 { "A" } else { "AAAA" };
    let ip = if is_ip(domain) {
        domain.to_string()
    } else {
        let res = dns_resolve::resolve(domain, record_type, ns).await;
        if res.is_none() {
            socket
                .emit(
                    event,
                    json!({
                        "job_id": job_id,
                        "node_id": node_id,
                        "error": SocketIOError::ErrDNSLookupFailed,
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
    let times = if single { 1 } else { 100 };
    let mut interval = time::interval(Duration::from_secs(1));
    for idx in 0..times {
        interval.tick().await;
        let start = std::time::Instant::now();
        let res = net::TcpStream::connect(host).await;
        match res {
            Ok(_) => {
                let ms = start.elapsed().as_millis();
                socket
                    .emit(
                        event,
                        json!({
                            "job_id": job_id,
                            "node_id": node_id,
                            "ip": ip,
                            "duration": ms,
                            "seq": idx+1,
                        }),
                    )
                    .await?;
            }
            Err(e) => {
                error!("tcping {} failed: {}", ip.to_string(), e);
                socket
                    .emit(
                        event,
                        json!({
                            "job_id": job_id,
                            "node_id": node_id,
                            "ip": ip,
                            "seq": idx+1,
                            "error": SocketIOError::ErrTCPingFailed,
                        }),
                    )
                    .await?;
            }
        };
    }
    Ok(())
}
