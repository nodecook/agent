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
    #[arg(short = '4', long, env = "NCA_IPV4_SERVER")]
    pub ipv4_server: Option<String>,
    /// IPv6 server address
    #[arg(short = '6', long, env = "NCA_IPV6_SERVER")]
    pub ipv6_server: Option<String>,
    /// IPv4 only mode
    #[arg(long, default_value_t = false, env = "NCA_IPV4_ONLY")]
    pub ipv4_only: bool,
    /// IPv6 only mode
    #[arg(long, default_value_t = false, env = "NCA_IPV6_ONLY")]
    pub ipv6_only: bool,
    /// API key comes from nodecook to know this node belongs to you
    #[arg(short, long, env = "NCA_API_KEY")]
    pub api_key: String,
    /// Enable debug mode
    #[arg(short, long, default_value_t = false, env = "NCA_DEBUG")]
    pub debug: bool,
}
