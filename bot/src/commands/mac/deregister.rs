use super::is_valid_mac;
use crate::{CommandResult, Context, Result};
use amizone::api::client::UserClient;

static WIFIMACDEREGISTER_HELP: &str = "/deregisterwifimac - DeRegister a WiFi MAC address.\n\n\
        Usage: /deregisterwifimac [mac_address]\n\n\
        Alias: /wd [mac_address]\n\n\
        Arguments:\n\
        - [mac_address]: The MAC address to deregister.\n\n\
        Example:\n\
        /registerwifimac 00:11:22:33:44:55\n\
        /wr 00:11:22:33:44:55\n\n\
        Note: The MAC address must be in the following format: `XX:XX:XX:XX:XX:XX` or `XX-XX-XX-XX-XX-XX`. It must also \
        be already registered on amizone, use the `/wifimacinfo` command to see what adresses are registered.";

///  DeRegister a WiFi MAC address.
#[poise::command(
    prefix_command,
    slash_command,
    help_text_fn = "wifimacderegister_help",
    aliases("wd")
)]
pub async fn deregisterwifimac(ctx: Context<'_>, address: String) -> CommandResult {
    ctx.defer().await?;

    if is_valid_mac(&address) {
        ctx.say(format!("`{}` is not a valid MAC address.", address))
            .await?;
        return Ok(());
    };

    let mut invocation_data = ctx.invocation_data::<Result<UserClient>>().await.unwrap();

    let client = invocation_data.as_mut()?;
    client.deregister_wifi_mac(address).await?;
    ctx.say("DeRegistered the MAC succesfully.").await?;

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

fn wifimacderegister_help() -> String {
    WIFIMACDEREGISTER_HELP.into()
}
