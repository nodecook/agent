mod cli;
mod client;
mod constant;
mod dns_resolve;
mod errors;
mod handlers;
mod utils;

use std::process::exit;
use std::time::Duration;

use crate::constant::V4_SERVER;
use crate::{cli::Cli, constant::V6_SERVER};

use crate::client::connect_server;
use clap::Parser;
use tokio::sync::mpsc;
use tokio::task;
use tokio::time::{self};
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
    let v4_node_id = args.v4_node_id;
    let v6_node_id = args.v6_node_id;
    if !args.v4_only {
        match connect_server(
            tx.clone(),
            "v6".to_string(),
            v6_server.clone(),
            args.api_key.clone(),
            v6_node_id,
        )
        .await
        {
            Ok(_) => {
                v6_ok = true;
            }
            Err(e) => {
                error!("connect ipv6 server failed: {}", e);
            }
        }
    }
    if !args.v6_only {
        match connect_server(
            tx.clone(),
            "v4".to_string(),
            v4_server.clone(),
            args.api_key.clone(),
            v4_node_id,
        )
        .await
        {
            Ok(_) => {
                v4_ok = true;
            }
            Err(e) => {
                error!("connect ipv4 server failed: {}", e);
            }
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
    let mut v4_last_time = time::Instant::now();
    let mut v6_last_time = time::Instant::now();
    while let Some(server_type) = rx.recv().await {
        if server_type == "v4" {
            if v4_last_time.elapsed() < Duration::from_secs(5) {
                continue;
            }
            warn!("ipv4 server disconnected, try to reconnect...");
            v4_last_time = time::Instant::now();
            match connect_server(
                tx.clone(),
                server_type.clone(),
                v4_server.clone(),
                args.api_key.clone(),
                v4_node_id,
            )
            .await
            {
                Ok(_) => {
                    info!("Congratulations, you have successfully reconnected to the ipv4 server!");
                }
                Err(e) => {
                    error!(
                        "reconnect ipv4 server failed: {}, sleep 5 seconds and try again",
                        e
                    );
                    let tx = tx.clone();
                    task::spawn(async move {
                        let _ = tx.send(server_type).await;
                    });
                }
            }
        } else if server_type == "v6" {
            if v6_last_time.elapsed() < Duration::from_secs(5) {
                continue;
            }
            warn!("ipv6 server disconnected, try to reconnect...");
            v6_last_time = time::Instant::now();
            match connect_server(
                tx.clone(),
                server_type.clone(),
                v6_server.clone(),
                args.api_key.clone(),
                v6_node_id,
            )
            .await
            {
                Ok(_) => {
                    info!("Congratulations, you have successfully reconnected to the ipv6 server!");
                }
                Err(e) => {
                    error!(
                        "reconnect ipv6 server failed: {}, sleep 5 seconds and try again",
                        e
                    );
                    let tx = tx.clone();
                    task::spawn(async move {
                        let _ = tx.send(server_type).await;
                    });
                }
            }
        }
    }
}
