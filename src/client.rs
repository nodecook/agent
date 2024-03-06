use crate::constant::VERSION;
use crate::handlers::{dns, http, mtr, ping, tcping};
use futures_util::FutureExt;
use rust_socketio::TransportType::Websocket;
use rust_socketio::{asynchronous::Client, asynchronous::ClientBuilder, Payload};
use tracing::error;

pub async fn connect_server(
    server: String,
    api_key: String,
) -> Result<rust_socketio::asynchronous::Client, rust_socketio::Error> {
    ClientBuilder::new(server)
        .transport_type(Websocket)
        .opening_header("Authorization", format!("Bearer {}", api_key))
        .opening_header("x-version", VERSION)
        .namespace("/agent")
        .on("ping", |payload: Payload, socket: Client| {
            async move {
                ping::ping(payload, socket).await;
            }
            .boxed()
        })
        .on("tcping", |payload: Payload, socket: Client| {
            async move {
                tcping::tcping(payload, socket).await;
            }
            .boxed()
        })
        .on("dns", |payload: Payload, socket: Client| {
            async move {
                dns::dns(payload, socket).await;
            }
            .boxed()
        })
        .on("mtr", |payload: Payload, socket: Client| {
            async move {
                mtr::mtr(payload, socket).await;
            }
            .boxed()
        })
        .on("http", |payload: Payload, socket: Client| {
            async move { http::http(payload, socket).await }.boxed()
        })
        .on("error", |err, _| {
            async move {
                let data = match err {
                    Payload::String(data) => data,
                    Payload::Binary(data) => String::from_utf8_lossy(&data).to_string(),
                };
                error!("Connect server error: {}", data);
            }
            .boxed()
        })
        .connect()
        .await
}
