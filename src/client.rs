use crate::constant::VERSION;
use crate::handlers::{dns, http, mtr, ping, tcping};
use futures_util::FutureExt;
use rust_socketio::TransportType::Websocket;
use rust_socketio::{asynchronous::Client, asynchronous::ClientBuilder, Payload};
use serde_json::json;
use serde_json::Value::Null;
use tokio::sync::mpsc;
use tracing::{debug, error};
use tokio::{task, time};

async fn ping_interval(tx: mpsc::Sender<String>, server_type: String, socket: Client) {
    let mut interval = time::interval(time::Duration::from_secs(10));
    loop {
        interval.tick().await;
        match socket.emit("agent", json!(Null)).await {
            Ok(_) => {}
            Err(err) => {
                error!("ping error: {}", err);
                tx.send(server_type).await.unwrap();
                break;
            }
        }
    }
}

pub async fn connect_server(
    tx: mpsc::Sender<String>,
    server_type: String,
    server: String,
    api_key: String,
) -> Result<rust_socketio::asynchronous::Client, rust_socketio::Error> {
    let tx_clone = tx.clone();
    let server_type_clone = server_type.clone();
    ClientBuilder::new(server)
        .transport_type(Websocket)
        .opening_header("Authorization", format!("Bearer {}", api_key))
        .opening_header("x-version", VERSION)
        .namespace("/agent")
        .on("open", move |_, socket| {
            let tx = tx_clone.clone();
            let server_type = server_type_clone.clone();
            async move {
                task::spawn(ping_interval(tx, server_type, socket.clone()));
            }
                .boxed()
        })
        .on("ping", |payload: Payload, socket: Client| {
            async move {
                match ping::ping(payload, socket).await {
                    Ok(_) => {}
                    Err(err) => {
                        debug!("ping error: {}", err);
                    }
                }
            }
                .boxed()
        })
        .on("tcping", |payload: Payload, socket: Client| {
            async move {
                match tcping::tcping(payload, socket).await {
                    Ok(_) => {}
                    Err(err) => {
                        debug!("tcping error: {}", err);
                    }
                }
            }
                .boxed()
        })
        .on("dns", |payload: Payload, socket: Client| {
            async move {
                match dns::dns(payload, socket).await {
                    Ok(_) => {}
                    Err(err) => {
                        debug!("dns error: {}", err);
                    }
                }
            }
                .boxed()
        })
        .on("mtr", |payload: Payload, socket: Client| {
            async move {
                match mtr::mtr(payload, socket).await {
                    Ok(_) => {}
                    Err(err) => {
                        debug!("mtr error: {}", err);
                    }
                }
            }
                .boxed()
        })
        .on("http", |payload: Payload, socket: Client| {
            async move {
                match http::http(payload, socket).await {
                    Ok(_) => {}
                    Err(err) => {
                        debug!("http error: {}", err);
                    }
                }
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
                error!("Server disconnected: {}", data);
                tx.send(server_type).await.unwrap();
            }
                .boxed()
        })
        .connect()
        .await
}
