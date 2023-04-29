use amizone::api::user::User;

use crate::{Context, Error};

static LOGIN_HELP: &'static str = "/login - Log into Amizone with your credentials.

Usage: /login [username] [password]

Arguments:
- [username]: Your Amizone username.
- [password]: Your Amizone password.

Example:
/login johnsmith password123

Note: Your credentials are securely stored using encryption.";

/// Log into Amizone with your credentials
#[poise::command(slash_command, help_text_fn = "login_help", check = "login_check")]
pub async fn login(
    ctx: Context<'_>,
    #[description = "Your amizone username"] username: String,
    #[description = "Your amizone password"] password: String,
) -> Result<(), Error> {
    let db_client = &ctx.data().connections.db;
    let amizone_conn = &ctx.data().connections.amizone;
    let caller_id = ctx.author().id.to_string();

    let mut amizone_client = User::new(&caller_id, &username, &password, db_client)
        .await?
        .get_client(amizone_conn.clone())?;

    // Random call to the api to see if the credentials are correct
    if amizone_client.get_user_profile().await.is_err() {
        ctx.say("Incorrect credentials.").await?;
        User::forget(caller_id, db_client).await?;
    } else {
        ctx.say("Login successfull, use the help command to get started.")
            .await?;
    }

    Ok(())
}

fn login_help() -> String {
    LOGIN_HELP.into()
}

async fn login_check(ctx: Context<'_>) -> Result<bool, Error> {
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
