use amizone::api::user::User;

use crate::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn login(ctx: Context<'_>, username: String, password: String) -> Result<(), Error> {
    let db_client = &ctx.data().connections.db;
    let amizone_conn = &ctx.data().connections.amizone;
    let caller_id = ctx.author().id.to_string();

    let mut amizone_client = User::new(&caller_id, &username, &password, db_client)
        .await?
        .get_client(amizone_conn.clone())?;

    // Random call to the api to see if the credentials are correct
    if amizone_client.get_user_profile().await.is_err() {
        ctx.say("Incorrect credentials.").await?;
        User::forget(&caller_id, db_client).await?;
    } else {
        ctx.say("Login successfull, use the help command to get started.")
            .await?;
    }

    Ok(())
}
