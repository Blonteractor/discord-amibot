use super::error::BotError;
use super::{Connections, Context, Data, Result, IGNORE_CHECK};
use std::env;
use std::str::FromStr;
use std::time;

use amizone::api::{self as amizoneapi, client::UserClient, types::AmizoneApiError};
use poise::{
    serenity_prelude::{Context as SerenityContext, Ready, UserId},
    Framework,
};

pub async fn on_ready<'a>(
    ctx: &SerenityContext,
    ready: &Ready,
    framework: &Framework<Data, BotError>,
) -> Result<Data> {
    println!("Registering commands...");

    #[cfg(not(debug_assertions))]
    poise::builtins::register_globally(ctx, framework.options().commands.as_slice())
        .await
        .unwrap();

    #[cfg(debug_assertions)]
    poise::builtins::register_in_guild(
        ctx,
        framework.options().commands.as_slice(),
        std::env::var("DEV_SERVER_ID")
            .unwrap()
            .parse::<u64>()
            .unwrap()
            .into(),
    )
    .await
    .unwrap();

    let connections = Connections {
        amizone: amizoneapi::new_amizone_connection(env::var("AMIZONE_API_URL").unwrap())
            .await
            .unwrap(),
        db: amizoneapi::new_db_connection(env::var("DATABASE_URL").unwrap())
            .await
            .unwrap(),
    };
    let start_time = time::Instant::now();
    let dev_user_id = UserId::from_str(&env::var("DEV_ID").unwrap_or_default()).unwrap_or_default();

    println!("Amibot is ready");
    Ok(Data {
        start_time,
        connections,
        dev_user_id,
        bot_user_id: ready.user.id,
    })
}

// Initialize user client before every* command
pub async fn init_client(ctx: Context<'_>) {
    if IGNORE_CHECK.contains(&ctx.invoked_command_name()) {
        return;
    }

    let db_client = &ctx.data().connections.db;
    let amizone_conn = &ctx.data().connections.amizone;
    let caller_id = ctx.author().id.to_string();

    let invocation_data = match amizoneapi::user::User::from_id(caller_id, db_client).await {
        Ok(user) => match user {
            Some(user) => match user.get_client(amizone_conn.clone()) {
                Ok(user_client) => Ok(user_client),
                Err(amizone_error) => Err(amizone_error.into()),
            },
            None => Err(BotError::AmizoneError(AmizoneApiError::not_found(
                "User not logged in",
            ))),
        },
        Err(dberror) => Err(dberror.into()),
    };

    ctx.set_invocation_data::<Result<UserClient>>(invocation_data)
        .await;
}

pub async fn loggedin_check(ctx: Context<'_>) -> Result<bool> {
    if IGNORE_CHECK.contains(&ctx.invoked_command_name()) {
        return Ok(true);
    }

    if amizoneapi::user::User::from_id(ctx.author().id.to_string(), &ctx.data().connections.db)
        .await?
        .is_none()
    {
        ctx.say("Not logged in, login using `/login` to get started.")
            .await?;

        Ok(false)
    } else {
        Ok(true)
    }
}
