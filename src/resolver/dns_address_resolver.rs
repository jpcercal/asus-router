use std::net::IpAddr;
use std::time::Duration;

use anyhow::anyhow;
use log::info;
use trippy::dns::{Config, DnsResolver, IpAddrFamily, ResolveMethod, Resolver};

pub fn resolve(hostname: &str) -> anyhow::Result<Vec<IpAddr>, anyhow::Error> {
    let resolver = DnsResolver::start(Config::new(
        ResolveMethod::Google,
        IpAddrFamily::Ipv4Only,
        Duration::from_millis(5000),
        Duration::from_secs(300),
    ))?;

    info!("{}: resolving IP addresses", hostname);
    let result: Vec<_> = resolver
        .lookup(&hostname)
        .map_err(|_| anyhow!(format!("unknown host {}", hostname)))?
        .into_iter()
        .collect();

    if result.is_empty() {
        return Err(anyhow!(format!(
            "no DNS matches found for the hostname {}",
            hostname
        )));
    }
    info!(
        "{}: found {} IP addresses ({:?})",
        hostname,
        result.len(),
        result
    );

    Ok(result)
}
