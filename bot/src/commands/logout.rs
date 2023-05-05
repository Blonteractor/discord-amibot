use amizone::api::user::User;

use crate::{CommandResult, Context};

static FORGET_HELP: &'static str =
    "/logout - Logs out of Amizone and deletes your stored credentials from the database.

    Usage: /logout
    
    Aliases: /forget
    
    Example:
    /logout
    
    Note: This command will log you out of Amizone and permanently delete your stored credentials from the database.
    After running this command, you will need to re-enter your credentials using the /login command in order to access Amizone.";

/// Make the bot into Amizone with your credentials
#[poise::command(
    prefix_command,
    slash_command,
    help_text_fn = "forget_help",
    aliases("forget")
)]
pub async fn logout(ctx: Context<'_>) -> CommandResult {
    let db_client = &ctx.data().connections.db;
    let caller_id = ctx.author().id.to_string();

    User::forget(caller_id, db_client).await?;

    Ok(())
}

fn forget_help() -> String {
    FORGET_HELP.into()
}
