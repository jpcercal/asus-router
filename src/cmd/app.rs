use anyhow::Result;
use clap::Command;
use log::info;

pub const CMD_NAME: &str = "asus-router";
const CMD_DESC: &str = "AsusRouter Software";
const CMD_VERSION: &str = "1.0.0";

pub fn boot() -> Result<()> {
    let matches = Command::new(CMD_NAME)
        .about(CMD_DESC)
        .version(CMD_VERSION)
        .arg_required_else_help(true)
        .subcommand(crate::cmd::discover_youtube_subnets_command::init())
        .subcommand(crate::cmd::apply_iptable_rules_command::init())
        .get_matches();

    match matches.subcommand() {
        Some((crate::cmd::discover_youtube_subnets_command::CMD_NAME, sub_matches)) => {
            return crate::cmd::discover_youtube_subnets_command::run(sub_matches);
        }
        Some((crate::cmd::apply_iptable_rules_command::CMD_NAME, sub_matches)) => {
            return crate::cmd::apply_iptable_rules_command::run(sub_matches);
        }
        _ => {
            info!("no subcommand matches were found");
        }
    }

    Ok(())
}
