use hickory_resolver::config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts};
use hickory_resolver::lookup::Lookup;
use hickory_resolver::TokioAsyncResolver;
use std::net::SocketAddr;
use std::str::FromStr;
use tracing::error;

pub async fn resolve(domain: &str, record_type: &str, nameserver: Option<&str>) -> Option<Lookup> {
    let mut config = ResolverConfig::default();
    if !nameserver.is_none() {
        let mut ns = nameserver.unwrap().to_string();
        if !ns.ends_with(":53") {
            if record_type == "AAAA" {
                ns = format!("[{}]:53", ns)
            } else {
                ns = format!("{}:53", ns);
            }
        }
        config.add_name_server(NameServerConfig {
            socket_addr: SocketAddr::from_str(&ns).unwrap(),
            protocol: Protocol::Udp,
            tls_dns_name: None,
            trust_negative_responses: false,
            bind_addr: None,
        });
    }

    let resolver = TokioAsyncResolver::tokio(config, ResolverOpts::default());
    match resolver.lookup(domain, record_type.parse().unwrap()).await {
        Ok(res) => Some(res),
        Err(e) => {
            error!(
                "dns resolve failed: {}, domain: {}, record_type: {}",
                e, domain, record_type
            );
            None
        }
    }
}
