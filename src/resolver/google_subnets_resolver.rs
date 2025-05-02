use anyhow::{Context, Result};
use ipnet::IpNet;
use log::{debug, info};
use reqwest::blocking::get;
use serde::Deserialize;

// URL of Google's JSON file with official IPv4 subnets.
const GOOGLE_KNOWN_IP_RANGES_URL: &str = "https://www.gstatic.com/ipranges/goog.json";

// Structures to parse the JSON file.
#[derive(Deserialize)]
struct GoogleIPRanges {
    prefixes: Vec<GooglePrefix>, // JSON contains an array "prefixes"
}

#[derive(Deserialize)]
struct GooglePrefix {
    #[serde(rename = "ipv4Prefix")]
    ipv4_prefix: Option<String>, // Some entries might not have an IPv4 prefix.
}

pub fn resolve() -> Result<Vec<IpNet>, anyhow::Error> {
    info!(
        "fetching google known IP ranges from {}",
        GOOGLE_KNOWN_IP_RANGES_URL
    );
    let response = get(GOOGLE_KNOWN_IP_RANGES_URL).context("Failed to fetch JSON URL")?;
    let json_text = response.text().context("Failed to read response text")?;
    let ip_ranges: GoogleIPRanges =
        serde_json::from_str(&json_text).context("Failed to parse JSON")?;

    // Extract subnets from JSON.
    let mut result: Vec<IpNet> = Vec::new();
    for prefix in ip_ranges.prefixes {
        if let Some(ipv4) = prefix.ipv4_prefix {
            let addr = IpNet::from(
                ipv4.parse::<IpNet>()
                    .context("Failed to parse IPv4 address")?,
            );
            debug!("adding the IPv4 subnet {}", addr);
            result.push(addr);
        }
    }
    result.sort();
    result.dedup();
    info!("found {} subnets", result.len());

    Ok(result)
}
