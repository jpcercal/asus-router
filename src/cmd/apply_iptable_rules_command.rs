use anyhow::{anyhow, Context, Error, Result};
use clap::{Arg, ArgAction, ArgMatches, Command};
use ipnet::IpNet;
use log::{debug, info};
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::process::{Command as StdProcessCommand, Stdio};

pub const CMD_NAME: &str = "apply-iptable-rules";
const CMD_DESC: &str = "Apply iptable rules";
const CMD_ARG_SUBNETS: &str = "subnets";
const CMD_ARG_IPS: &str = "ips";

const BACKUP_DIR: &str = "/tmp/home/root/backup";
const IPV4_CONF_DIR: &str = "/proc/sys/net/ipv4/conf";
const VPN_ROUTE_TABLE_ID: &str = "900";
const LAN_INTERFACE: &str = "br0";
const VPN_TUNNEL_INTERFACE: &str = "wgc5";
const VPN_MARK: &str = "9";
const VPN_IPSET_NAME: &str = "asusrouter_custom_vpn_ipset";

pub fn init() -> Command {
    Command::new(CMD_NAME).about(CMD_DESC).arg(
        Arg::new(CMD_ARG_SUBNETS)
            .long(CMD_ARG_SUBNETS)
            .required(true)
            .action(ArgAction::Set)
            .default_value(""),
    ).arg(
        Arg::new(CMD_ARG_IPS)
            .long(CMD_ARG_IPS)
            .required(true)
            .action(ArgAction::Set)
            .default_value(""),
    )
}

pub fn run(sub_matches: &ArgMatches) -> Result<()> {
    if let Some(s) = sub_matches.get_one::<String>(CMD_ARG_SUBNETS) {
        let subnets: Vec<IpNet> = s
            .split(',')
            .map(|s| match s.parse() {
                Ok(addr) => addr,
                Err(e) => {
                    info!("failed to parse subnet ({}): {}", s, e);
                    process::exit(1);
                }
            })
            .collect();

        info!(
            "parsed {} subnets successfully ({:?})",
            subnets.len(),
            subnets
        );

        disable_rp_filter().context("Error in disabling rp_filter")?;
        cleanup_previous_applied_configurations()
            .context("Error in cleaning up routes and rules")?;
        configure_ipset(subnets).context("Error in configuring IPSet")?;
        setup_iptables_and_vpn_routing().context("Error in setting up iptables and VPN routing")?;
    }

    Ok(())
}

// Disable Reverse Path Filtering
fn disable_rp_filter() -> Result<()> {
    info!("Disabling Reverse Path Filtering on all interfaces...");

    info!(
        "Creating backup directory (if non-existent yet) on {}",
        BACKUP_DIR
    );
    let path = Path::new(BACKUP_DIR);
    fs::create_dir_all(&path)
        .with_context(|| format!("Failed to create directory: {}", BACKUP_DIR))?;

    // Iterate over all entries in the conf directory.
    for entry in fs::read_dir(IPV4_CONF_DIR)
        .with_context(|| format!("Failed to read directory '{}'", IPV4_CONF_DIR))?
    {
        let entry = entry?;
        let mut rp_filter_path: PathBuf = entry.path();
        rp_filter_path.push("rp_filter");

        if rp_filter_path.exists() {
            // Read the original content before writing "0"
            let old_value = fs::read_to_string(&rp_filter_path)
                .with_context(|| format!("Failed to read content from {:?}", rp_filter_path))?;

            // Create backup file path
            let interface_name = entry.file_name().into_string().unwrap_or_default();
            let backup_path = format!("{}/{}", BACKUP_DIR, interface_name);
            let backup_file = format!("{}/{}", backup_path, "rp_filter");

            // Create interface backup folder
            fs::create_dir_all(&backup_path)
                .with_context(|| format!("Failed to create directory: {}", backup_path))?;
            // Write backup content
            fs::write(&backup_file, &old_value)
                .with_context(|| format!("Failed to write backup to {:?}", backup_file))?;

            // Write "0" to disable reverse path filtering.
            fs::write(&rp_filter_path, "0").with_context(|| {
                format!(
                    "Failed to write 0 as the file content of {:?}",
                    rp_filter_path
                )
            })?;

            info!(
                "Setting rp_filter value from \"{}\" to \"0\" and backing up {:?} onto {:?}",
                old_value.trim(),
                rp_filter_path,
                backup_file
            );
        } else {
            debug!("Skipping {:?} (no rp_filter file)", rp_filter_path);
        }
    }
    Ok(())
}

// Clean Up Existing Routes and Rules
fn cleanup_previous_applied_configurations() -> Result<()> {
    info!(
        "Cleaning up pre-existing configurations installed by this script before {}...",
        VPN_ROUTE_TABLE_ID
    );

    let commands: Vec<String> = vec![
        format!("ip route flush table {}", VPN_ROUTE_TABLE_ID),
        format!("ip route del default table {}", VPN_ROUTE_TABLE_ID),
        format!("ip rule del table {}", VPN_ROUTE_TABLE_ID),
        "ip route flush cache".to_string(),
        format!("iptables -t mangle -D PREROUTING -i {} -m set --match-set {} dst -j MARK --set-mark {}", LAN_INTERFACE, VPN_IPSET_NAME, VPN_MARK),
        format!("ipset destroy {}", VPN_IPSET_NAME),
    ];

    for command in &commands {
        execute_cmd(command).context(format!("Failed to execute {:?}", command))?;
    }

    Ok(())
}

// Configure IPSet for VPN Destinations
fn configure_ipset(subnets: Vec<IpNet>) -> Result<()> {
    info!("Creating and configuring IPSet '{}'...", VPN_IPSET_NAME);

    let mut commands: Vec<String> = vec![format!(
        "ipset create {} hash:net family inet hashsize 1024 maxelem 65536",
        VPN_IPSET_NAME
    )];

    // Add destination IPs to the IPSet (using "-exist" to ignore duplicates).
    for subnet in subnets {
        commands.push(format!("ipset add {} {} -exist", VPN_IPSET_NAME, subnet));
    }

    for command in &commands {
        execute_cmd(command).context(format!("Failed to execute {:?}", command))?;
    }

    Ok(())
}

// Set Up iptables and VPN Routing
fn setup_iptables_and_vpn_routing() -> Result<()> {
    info!("Configuring iptables to mark packets for VPN routing...");

    let commands: Vec<String> = vec![
        format!("iptables -t mangle -A PREROUTING -i {} -m set --match-set {} dst -j MARK --set-mark {}", LAN_INTERFACE, VPN_IPSET_NAME, VPN_MARK),
        format!("ip route add default dev {} table {}", VPN_TUNNEL_INTERFACE, VPN_ROUTE_TABLE_ID),
        format!("ip rule add fwmark {} table {}", VPN_MARK, VPN_ROUTE_TABLE_ID),
        "ip route flush cache".to_string(),
    ];

    for command in &commands {
        execute_cmd(command).context(format!("Failed to execute {:?}", command))?;
    }

    Ok(())
}

fn execute_cmd(command: &String) -> Result<(), Error> {
    match resolve_cmd_and_args(command) {
        Some((cmd, args)) => {
            info!(" $ {} {}", cmd, args.join(" "));
            StdProcessCommand::new(&cmd)
                .args(&args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| format!("Failed to run command: {} {}", cmd, args.join(" ")))?;
        }
        None => {
            return Err(anyhow!("Invalid command: '{}'", command));
        }
    }
    Ok(())
}

fn resolve_cmd_and_args(cmd_and_args_as_str: &str) -> Option<(String, Vec<String>)> {
    let parts: Vec<&str> = cmd_and_args_as_str.split_whitespace().collect();

    if parts.is_empty() {
        return None;
    }

    let cmd = parts[0].trim().to_string();
    let args = parts[1..].iter().map(|s| s.trim().to_string()).collect();

    Some((cmd, args))
}
