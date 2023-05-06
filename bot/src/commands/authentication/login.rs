use amizone::api::user::User;

use crate::{BotError, CommandResult, Context};

static LOGIN_HELP: &str = "/login - Log into Amizone with your credentials.

Usage: /login [username] [password]

Arguments:
- [username]: Your Amizone username.
- [password]: Your Amizone password.

Example:
/login johnsmith password123

Note: Your credentials are securely stored using encryption.";

/// Log into Amizone with your credentials
#[poise::command(
    prefix_command,
    slash_command,
    help_text_fn = "login_help",
    check = "login_check"
)]
pub async fn login(
    ctx: Context<'_>,
    #[description = "Your amizone username"] username: String,
    #[description = "Your amizone password"] password: String,
) -> CommandResult {
    ctx.defer_ephemeral().await?;
    let db_client = &ctx.data().connections.db;
    let amizone_conn = &ctx.data().connections.amizone;
    let caller_id = ctx.author().id.to_string();

    let mut amizone_client = User::new(&caller_id, &username, &password, db_client)
        .await?
        .get_client(amizone_conn.clone())?;

    if let Ok(profile) = amizone_client.get_user_profile().await {
        ctx.say(format!(
            "Logged in as `{}` of `{}`, use the help command to get started.",
            profile.name, profile.batch
        ))
        .await?;
    } else {
        ctx.say("Incorrect credentials.").await?;
        User::forget(caller_id, db_client).await?;
    }

    Ok(())
}

fn login_help() -> String {
    LOGIN_HELP.into()
}

async fn login_check(ctx: Context<'_>) -> Result<bool, BotError> {
    if User::from_id(ctx.author().id.to_string(), &ctx.data().connections.db)
        .await?
        .is_some()
    {
        ctx.say("You are already logged in and ready to go.")
            .await?;

        Ok(false)
    } else {
        Ok(true)
    }
}
