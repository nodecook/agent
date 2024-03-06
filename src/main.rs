mod cli;
mod client;
mod constant;
mod dns_resolve;
mod errors;
mod handlers;
mod utils;
use crate::constant::V4_SERVER;
use crate::{cli::Cli, constant::V6_SERVER};

use crate::client::connect_server;
use clap::Parser;
use tokio::signal;
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
        std::process::exit(1);
    }
    let mut v4_ok = false;
    let mut v6_ok = false;
    if !args.v4_only {
        let server = args.v6_server.unwrap_or_else(|| V6_SERVER.to_string());
        let ret = connect_server(server, args.api_key.clone()).await;
        if ret.is_ok() {
            v4_ok = true
        } else {
            debug!("connect ipv6 server failed: {}", ret.err().unwrap());
        }
    }
    if !args.v6_only {
        let server = args.v4_server.unwrap_or_else(|| V4_SERVER.to_string());
        let ret = connect_server(server, args.api_key).await;
        if ret.is_ok() {
            v6_ok = true
        } else {
            debug!("connect ipv4 server failed: {}", ret.err().unwrap());
        }
    }
    if !v4_ok && !v6_ok {
        error!("connect ipv4 and ipv6 server failed, please check your network or try again later");
        std::process::exit(1);
    }
    info!("Congratulations, you have successfully started the agent!");

    shutdown_signal().await;
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
