use anyhow::{Context, Result};
use clap::{ArgMatches, Command};
use ip_in_subnet::iface_in_subnet;
use ipnet::IpNet;
use log::{debug, info};
use std::process;

pub const CMD_NAME: &str = "discover-youtube-subnets";
const CMD_DESC: &str = "Discover public YouTube subnets and export to text file";

const YOUTUBE_RELATED_HOSTNAMES: [&str; 14] = [
    "youtube.com",
    "www.youtube.com",
    "m.youtube.com",
    "youtube.googleapis.com",
    "youtubei.googleapis.com",
    "redirector.googlevideo.com",
    "manifest.googlevideo.com",
    "i.ytimg.com",
    "s.ytimg.com",
    "ytimg.l.google.com",
    "video.google.com",
    "yt3.ggpht.com",
    "yt.be",
    "youtu.be",
];

pub fn init() -> Command {
    Command::new(CMD_NAME).about(CMD_DESC)
}

pub fn run(_: &ArgMatches) -> Result<()> {
    let mut google_subnets = crate::resolver::google_subnets_resolver::resolve()
        .context("failed to get known subnets")?;
    let mut result: Vec<IpNet> = Vec::new();

    for hostname in YOUTUBE_RELATED_HOSTNAMES.iter() {
        let addrs = match crate::resolver::dns_address_resolver::resolve(hostname) {
            Ok(addrs) => addrs,
            Err(e) => {
                info!("failed to get subnets: {}", e);
                process::exit(1);
            }
        };

        for addr in addrs {
            // Convert addr to a String and keep it in a variable so the reference remains valid
            let addr_as_string = addr.to_string();

            // Retain only the subnets that do not match. If a subnet matches, push it into result.
            google_subnets.retain(|subnet| {
                // Pass the owned strings as references; both variables live long enough.
                let in_subnet =
                    iface_in_subnet(&addr_as_string, subnet.to_string().as_str()).unwrap();
                if in_subnet {
                    result.push(subnet.clone());
                }
                // If the interface is in the subnet, remove it (return false); otherwise, keep it (true)
                !in_subnet
            });
        }
    }

    debug!("google subnets count = {}", google_subnets.len());
    info!("found {} subnets ({:?})", result.len(), result);

    println!(
        "{}",
        serde_json::to_string(
            &result
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
        )?
    );

    Ok(())
}
