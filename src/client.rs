use crate::handlers::{dns, http, mtr, ping, tcping};
use futures_util::FutureExt;
use rust_socketio::TransportType::Websocket;
use rust_socketio::{asynchronous::Client, asynchronous::ClientBuilder, Payload};
use serde_json::json;
use serde_json::Value::Null;
use tokio::sync::mpsc;
use tokio::{task, time};
use tracing::error;

pub async fn ping_interval(tx: mpsc::Sender<String>, server_type: String, socket: Client) {
    let mut interval = time::interval(time::Duration::from_secs(10));
    loop {
        interval.tick().await;
        match socket.emit("agent", json!(Null)).await {
            Ok(_) => {}
            Err(err) => {
                error!("ping {server_type} error: {err}");
                send_server_error(tx, server_type).await;
                break;
            }
        }
    }
}

pub async fn send_server_error(tx: mpsc::Sender<String>, server_type: String) {
    task::spawn(async move {
        match tx.try_send(server_type) {
            Ok(_) => {}
            Err(err) => {
                error!("send_server_error error: {}", err);
            }
        }
    });
}

pub async fn connect_server(
    tx: mpsc::Sender<String>,
    server_type: String,
    server: String,
    api_key: String,
    node_id: Option<u16>,
) -> Result<rust_socketio::asynchronous::Client, rust_socketio::Error> {
    ClientBuilder::new(server)
        .transport_type(Websocket)
        .opening_header("Authorization", format!("Bearer {}", api_key))
        .opening_header("x-node-id", node_id.unwrap_or(0).to_string())
        .namespace("/agent")
        .on("ping", |payload: Payload, socket: Client| {
            async move {
                task::spawn(ping::ping(payload, socket));
            }
            .boxed()
        })
        .on("tcping", |payload: Payload, socket: Client| {
            async move {
                task::spawn(tcping::tcping(payload, socket));
            }
            .boxed()
        })
        .on("dns", |payload: Payload, socket: Client| {
            async move {
                task::spawn(dns::dns(payload, socket));
            }
            .boxed()
        })
        .on("mtr", |payload: Payload, socket: Client| {
            async move {
                task::spawn(mtr::mtr(payload, socket));
            }
            .boxed()
        })
        .on("http", |payload: Payload, socket: Client| {
            async move {
                task::spawn(http::http(payload, socket));
            }
            .boxed()
        })
        .on("error", move |err, _| {
            let tx = tx.clone();
            let server_type = server_type.clone();
            async move {
                let data = match err {
                    Payload::String(data) => data,
                    Payload::Binary(data) => String::from_utf8_lossy(&data).to_string(),
                };
                error!("Server {server_type} disconnected: {data}");
                send_server_error(tx, server_type).await;
            }
            .boxed()
        })
        .connect()
        .await
}
