use clap::Parser;

#[derive(Clone, Parser)]
#[command(
    name = "NodeCookAgent",
    version = env!("CARGO_PKG_VERSION"),
    author = "NodeCook <dev@nodecook.com>",
    about = "Agent for NodeCook to run jobs"
)]
pub struct Cli {
    /// IPv4 server address
    #[arg(short = '4', long, env = "NCA_V4_SERVER")]
    pub v4_server: Option<String>,
    /// IPv6 server address
    #[arg(short = '6', long, env = "NCA_V6_SERVER")]
    pub v6_server: Option<String>,
    /// IPv4 only mode
    #[arg(long, default_value_t = false, env = "NCA_V4_ONLY")]
    pub v4_only: bool,
    /// IPv6 only mode
    #[arg(long, default_value_t = false, env = "NCA_V6_ONLY")]
    pub v6_only: bool,
    /// Enable debug mode
    #[arg(short, long, default_value_t = false, env = "NCA_DEBUG")]
    pub debug: bool,
    /// IPv4 node id
    #[arg(long, env = "NCA_V4_NODE_ID")]
    pub v4_node_id: Option<u16>,
    /// IPv6 node id
    #[arg(long, env = "NCA_V6_NODE_ID")]
    pub v6_node_id: Option<u16>,
    /// Advertise title for this node
    #[arg(long, env = "NCA_TITLE")]
    pub title: Option<String>,
    /// Advertise link for this node
    #[arg(long, env = "NCA_LINK")]
    pub link: Option<String>,
}
