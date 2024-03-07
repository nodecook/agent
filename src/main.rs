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
use tracing::{debug, error, info, Level};
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
    if !args.v4_only {
        let server = args
            .v6_server
            .clone()
            .unwrap_or_else(|| V6_SERVER.to_string());
        let ret = connect_server(tx.clone(), "v4".to_string(), server, args.api_key.clone()).await;
        if ret.is_ok() {
            v4_ok = true
        } else {
            debug!("connect ipv6 server failed: {}", ret.err().unwrap());
        }
    }
    if !args.v6_only {
        let server = args
            .v4_server
            .clone()
            .unwrap_or_else(|| V4_SERVER.to_string());
        let ret = connect_server(tx.clone(), "v6".to_string(), server, args.api_key).await;
        if ret.is_ok() {
            v6_ok = true
        } else {
            debug!("connect ipv4 server failed: {}", ret.err().unwrap());
        }
    }
    if !v4_ok && !v6_ok {
        error!("connect ipv4 and ipv6 server failed, please check your network or try again later");
        exit(1);
    }
    info!("Congratulations, you have successfully started the agent!");

    while let Some(server_type) = rx.recv().await {
        if server_type == "v4" && v4_ok {
            error!("ipv4 server disconnected, try to reconnect...");
            exit(1);
        }
        if server_type == "v6" && v6_ok {
            error!("ipv6 server disconnected, try to reconnect...");
            exit(1);
        }
    }
}
