use amizone::api::client::UserClient;

use crate::{CommandResult, Context, Result};

static WIFIMACINFO_HELP: &str = "/wifimacinfo - Retrieve information about WiFi MAC addresses registered on amizone.\n\n\
Usage: /wifimacinfo\n\n\
Alias: /wi \n\n\
Example:\n\
/wifimacinfo\n\n\
Note: This command provides information about the registered WiFi MAC addresses, including the addresses, \
the number of free slots, and the total number of slots available.";

///  Retrieves your attendance records for the current semester.
#[poise::command(
    prefix_command,
    slash_command,
    help_text_fn = "wifimacinfo_help",
    aliases("wi")
)]
pub async fn wifimacinfo(ctx: Context<'_>) -> CommandResult {
    ctx.defer().await?;
    let mut invocation_data = ctx.invocation_data::<Result<UserClient>>().await.unwrap();

    let client = invocation_data.as_mut()?;

    let wifimac = client.get_wifi_mac_info().await?;
    let addresses = wifimac.addresses.join("`, `");
    let free_slots = wifimac.free_slots;
    let total_slots = wifimac.slots;

    ctx.say(format!(
        "**Adresses:** `{}`\n**Free Slots:** `{}`\n**Total Slots:** `{}`",
        addresses, free_slots, total_slots
    ))
    .await?;

    Ok(())
}

fn wifimacinfo_help() -> String {
    WIFIMACINFO_HELP.into()
}
