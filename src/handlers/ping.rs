use crate::dns_resolve;
use crate::errors::SocketIOError;
use crate::utils::is_ip;
use rand::random;
use rust_socketio::{asynchronous::Client as SocketClient, Payload};
use serde_json::{json, Value};
use std::time::Duration;
use surge_ping::{Client, Config, IcmpPacket, PingIdentifier, PingSequence, ICMP};
use tokio::time;
use tracing::debug;
use tracing::error;

pub async fn ping(payload: Payload, socket: SocketClient) -> Result<(), rust_socketio::Error> {
    let data: Value = match payload {
        Payload::String(data) => serde_json::from_str(&data).unwrap(),
        Payload::Binary(data) => serde_json::from_slice(&data).unwrap(),
    };
    debug!("receive ping request: {}", data);
    let job_id = data["job_id"].as_str().unwrap();
    let node_id = data["node_id"].as_u64().unwrap();
    let event = "ping";
    let host = data["host"].as_str().unwrap();
    let single = data["single"].as_bool().unwrap_or(true);
    let is_ipv4 = data["is_ipv4"].as_bool().unwrap_or(true);
    let ns: Option<&str> = data["ns"].as_str();
    let record_type: &str = if is_ipv4 { "A" } else { "AAAA" };
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
                        "error":SocketIOError::ErrDNSLookupFailed,
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
    let mut config_builder = Config::builder();
    if is_ipv4 {
        config_builder = config_builder.kind(ICMP::V4);
    } else {
        config_builder = config_builder.kind(ICMP::V6);
    }
    let config = config_builder.build();
    let client = Client::new(&config).unwrap();
    let payload = [0; 56];
    let mut pinger = client
        .pinger(ip.parse().unwrap(), PingIdentifier(random()))
        .await;
    let mut interval = time::interval(Duration::from_secs(1));
    pinger.timeout(Duration::from_secs(1));
    let times = if single { 1 } else { 100 };
    for idx in 0..times {
        interval.tick().await;
        match pinger.ping(PingSequence(idx), &payload).await {
            Ok((IcmpPacket::V4(packet), dur)) => {
                socket
                    .emit(
                        event,
                        json!({
                            "job_id": job_id,
                            "node_id": node_id,
                            "ip": packet.get_source(),
                            "duration": Some(dur).map(|d| d.as_millis()),
                            "seq": packet.get_sequence().0+1,
                        }),
                    )
                    .await?
            }
            Ok((IcmpPacket::V6(packet), dur)) => {
                socket
                    .emit(
                        event,
                        json!({
                            "job_id": job_id,
                            "node_id": node_id,
                            "ip": packet.get_source(),
                            "duration": Some(dur).map(|d| d.as_millis()),
                            "seq": packet.get_sequence().0+1,
                        }),
                    )
                    .await?
            }
            Err(e) => {
                error!("ping {} failed: {}", host, e);
                socket
                    .emit(
                        event,
                        json!({
                            "job_id": job_id,
                            "node_id": node_id,
                            "ip": ip,
                            "duration": None::<u64>,
                            "seq": idx+1,
                            "error": SocketIOError::ErrPingFailed,
                        }),
                    )
                    .await?;
            }
        }
    }
    Ok(())
}
