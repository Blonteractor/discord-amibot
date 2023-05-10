use crate::{CommandResult, Context};
use deregister::deregister;
use info::info;
use regex::Regex;
use register::register;

mod deregister;
mod info;
mod register;

static WIFIMAC_HELP: &str = "/wifimac [subcommands] - Commands to work with the WiFi MAC interface on amizone.\n\n\
Example:\n\
/wifimac info\n\n\
Note: This command provides information about the registered WiFi MAC addresses, including the addresses, \
the number of free slots, and the total number of slots available.";

// https://stackoverflow.com/questions/4260467/what-is-a-regular-expression-for-a-mac-address
pub fn is_valid_mac(addr: impl AsRef<str>) -> bool {
    let mac_regex = Regex::new(r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$").unwrap();
    mac_regex.is_match(addr.as_ref())
}

#[poise::command(
    prefix_command,
    slash_command,
    help_text_fn = "wifimac_help",
    aliases("wm"),
    subcommands("register", "deregister", "info")
)]
///Commands to work with the WiFi MAC interface on amizone.
pub async fn wifimac(ctx: Context<'_>) -> CommandResult {
    ctx.say(WIFIMAC_HELP).await?;
    Ok(())
}

fn wifimac_help() -> String {
    WIFIMAC_HELP.into()
}
