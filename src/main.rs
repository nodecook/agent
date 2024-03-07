mod cli;
mod client;
mod constant;
mod dns_resolve;
mod errors;
mod handlers;
mod utils;
use std::process::exit;

use crate::constant::V4_SERVER;
use crate::{cli::Cli, constant::V6_SERVER};

use crate::client::connect_server;
use clap::Parser;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{error, info, warn, Level};
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() {
    let mut level = Level::INFO;
    let args = Cli::parse();
    if args.debug {
        level = Level::DEBUG;
    }
    let collector = fmt().with_max_level(level).finish();
    tracing::subscriber::set_global_default(collector).unwrap();
    if args.v4_only && args.v6_only {
        error!("ipv4_only and ipv6_only can't be true at the same time");
        exit(1);
    }
    let (tx, mut rx) = mpsc::channel::<String>(2);
    let mut v4_ok = false;
    let mut v6_ok = false;
    let v4_server = args
        .v4_server
        .clone()
        .unwrap_or_else(|| V4_SERVER.to_string());
    let v6_server = args
        .v6_server
        .clone()
        .unwrap_or_else(|| V6_SERVER.to_string());
    if !args.v4_only {
        let ret = connect_server(
            tx.clone(),
            "v6".to_string(),
            v6_server.clone(),
            args.api_key.clone(),
        )
        .await;
        if ret.is_ok() {
            v6_ok = true
        } else {
            error!("connect ipv6 server failed: {}", ret.err().unwrap());
        }
    }
    if !args.v6_only {
        let ret = connect_server(
            tx.clone(),
            "v4".to_string(),
            v4_server.clone(),
            args.api_key.clone(),
        )
        .await;
        if ret.is_ok() {
            v4_ok = true
        } else {
            error!("connect ipv4 server failed: {}", ret.err().unwrap());
        }
    }
    if v4_ok {
        info!("Congratulations, you have successfully connected to the ipv4 server!");
    }
    if v6_ok {
        info!("Congratulations, you have successfully connected to the ipv6 server!");
    }
    if !v4_ok && !v6_ok {
        error!("Failed to connect to any server, please check your network and try again");
        exit(1);
    }

    while let Some(server_type) = rx.recv().await {
        if server_type == "v4" && v4_ok {
            warn!("ipv4 server disconnected, try to reconnect...");
            let ret = connect_server(
                tx.clone(),
                "v4".to_string(),
                v4_server.clone(),
                args.api_key.clone(),
            )
            .await;
            if ret.is_err() {
                error!(
                    "reconnect ipv4 server failed: {}, sleep 5 seconds and try again",
                    ret.err().unwrap()
                );
                tx.send("v4".to_string()).await.unwrap();
            }
            sleep(tokio::time::Duration::from_secs(5)).await;
        }
        if server_type == "v6" && v6_ok {
            warn!("ipv6 server disconnected, try to reconnect...");
            let ret = connect_server(
                tx.clone(),
                "v6".to_string(),
                v6_server.clone(),
                args.api_key.clone(),
            )
            .await;
            if ret.is_err() {
                error!(
                    "reconnect ipv6 server failed: {}, sleep 5 seconds and try again",
                    ret.err().unwrap()
                );
                tx.send("v6".to_string()).await.unwrap();
            }
            sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}
