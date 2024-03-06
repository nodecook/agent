use crate::dns_resolve;
use crate::errors::SocketIOError;
use crate::utils::is_ip;
use rust_socketio::{asynchronous::Client as SocketClient, Payload};
use serde_json::{json, Value};
use tracing::debug;
pub async fn dns(payload: Payload, socket: SocketClient) {
    let data: Value = match payload {
        Payload::String(data) => serde_json::from_str(&data).unwrap(),
        Payload::Binary(data) => serde_json::from_slice(&data).unwrap(),
    };
    debug!("receive dns request: {}", data);
    let job_id = data["job_id"].as_str().unwrap();
    let node_id = data["node_id"].as_u64().unwrap() as u16;
    let event = "dns";
    let domain = data["domain"].as_str().unwrap();
    let type_ = data["type"].as_str().unwrap();
    let ns = data["ns"].as_str();
    let start = std::time::Instant::now();
    let res = dns_resolve::resolve(domain, type_, ns).await;
    match res {
        Some(res) => {
            let ms = start.elapsed().as_millis();
            let ips = if type_ == "CNAME" {
                res.iter().map(|ip| ip.to_string()).collect::<Vec<String>>()
            } else {
                res.iter()
                    .filter(|ip| is_ip(&ip.to_string()))
                    .map(|ip| ip.to_string())
                    .collect::<Vec<String>>()
            };
            socket
                .emit(
                    event,
                    json!({
                        "job_id": job_id,
                        "node_id": node_id,
                        "duration": ms,
                        "ips": ips,
                    }),
                )
                .await
                .unwrap();
        }
        None => {
            socket
                .emit(
                    event,
                    json!({
                        "job_id": job_id,
                        "node_id": node_id,
                        "error": SocketIOError::ErrDNSLookupFailed
                    }),
                )
                .await
                .unwrap();
        }
    }
}
