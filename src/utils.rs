pub fn is_ip(ip: &str) -> bool {
    ip.parse::<std::net::IpAddr>().is_ok()
}
