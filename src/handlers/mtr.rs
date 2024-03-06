use crate::dns_resolve;
use crate::errors::SocketIOError;
use crate::utils::is_ip;
use rust_socketio::{asynchronous::Client as SocketClient, Payload};
use serde_json::{json, Value};
use std::thread;
use tracert::trace::Tracer;
use tracing::debug;
use tracing::error;

pub async fn mtr(payload: Payload, socket: SocketClient) {
    let data: Value = match payload {
        Payload::String(data) => serde_json::from_str(&data).unwrap(),
        Payload::Binary(data) => serde_json::from_slice(&data).unwrap(),
    };
    debug!("receive mtr request: {}", data);
    let job_id = data["job_id"].as_str().unwrap();
    let node_id = data["node_id"].as_u64().unwrap();
    let event = "mtr";
    let host = data["host"].as_str().unwrap();
    let ns = data["ns"].as_str();
    let is_ipv4 = data["is_ipv4"].as_bool().unwrap_or(true);
    let record_type = if is_ipv4 { "A" } else { "AAAA" };
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
                        "error": SocketIOError::ErrDNSLookupFailed,
                    }),
                )
                .await
                .unwrap();
            return;
        }
        res.unwrap()
            .iter()
            .filter(|ip| is_ip(&ip.to_string()))
            .next()
            .unwrap()
            .to_string()
    };
    let tracer: Tracer = Tracer::new(ip.parse().unwrap()).unwrap();
    let handle = thread::spawn(move || tracer.trace());
    match handle.join().unwrap() {
        Ok(r) => {
            for node in r.nodes {
                socket
                    .emit(
                        event,
                        json!({
                            "job_id": job_id,
                            "node_id": node_id,
                            "seq": node.seq,
                            "ip_addr": node.ip_addr,
                            "host_name": node.host_name,
                            "ttl": node.ttl,
                            "hop": node.hop,
                            "node_type": match node.node_type {
                                tracert::node::NodeType::DefaultGateway=> "DefaultGateway",
                                tracert::node::NodeType::Relay=> "Relay",
                                tracert::node::NodeType::Destination=> "Destination",
                            },
                            "rtt": node.rtt.as_millis(),
                        }),
                    )
                    .await
                    .unwrap();
            }
        }
        Err(e) => {
            error!("mtr {} failed: {}", host, e);
            socket
                .emit(
                    event,
                    json!({
                        "job_id": job_id,
                        "node_id": node_id,
                        "error": SocketIOError::ErrMTRFailed,
                    }),
                )
                .await
                .unwrap();
        }
    }
}
