use super::error::BotError;
use super::{Connections, Context, Data, Result, IGNORE_CHECK};
use log::{debug, info, trace};
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time;
use tokio::sync::Mutex;

use amizone::api::{self as amizoneapi, client::UserClient, types::AmizoneApiError};
use poise::{
    serenity_prelude::{Context as SerenityContext, Ready, UserId},
    Framework,
};

use poise::structs::FrameworkError;

pub async fn on_ready<'a>(
    ctx: &SerenityContext,
    ready: &Ready,
    framework: &Framework<Data, BotError>,
) -> Result<Data> {
    trace!("Registering commands");

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

    trace!("Setting up connections");
    let connections = Connections {
        amizone: amizoneapi::new_amizone_connection(
            env::var("AMIZONE_API_URL").expect("missing AMIZONE_API_URL"),
        )
        .await
        .unwrap(),
        db: amizoneapi::new_db_connection(env::var("DATABASE_URL").expect("missing DATABASE_URL"))
            .await
            .unwrap(),
    };
    let start_time = time::Instant::now();
    let dev_user_id = UserId::from_str(&env::var("DEV_ID").unwrap_or_default()).unwrap_or_default();

    info!("Amibot is ready");
    Ok(Data {
        start_time,
        connections,
        dev_user_id,
        bot_user_id: ready.user.id,
        users_cache: Arc::new(Mutex::new(HashMap::new())),
    })
}

pub async fn on_error(error: poise::FrameworkError<'_, crate::Data, BotError>) {
    info!("Encountered error => {}", error);

    // Error during a command
    match error {
        FrameworkError::Command { error, ctx } => error.handle(ctx).await,
        FrameworkError::CommandPanic { payload, ctx } => {
            debug!(
                "Command Panic: {}",
                payload.unwrap_or("Payload missing".to_string())
            );
            ctx.say("Critical error in command.").await.ok();
        }
        FrameworkError::CommandCheckFailed {
            error: Some(error),
            ctx,
        } => error.handle(ctx).await,
        FrameworkError::UnknownCommand {
            ctx,
            msg,
            prefix: _,
            msg_content,
            framework: _,
            invocation_data: _,
            trigger: _,
        } => {
            msg.reply(ctx, "Unkown command").await.ok();
            debug!("Unkown command: {}", msg_content);
        }
        FrameworkError::UnknownInteraction {
            ctx: _,
            framework: _,
            interaction,
        } => {
            debug!("Unkown interaction: {:?}", interaction);
        }
        _ => (),
    }
}

// Initialize user client before every* command
pub async fn init_client(ctx: Context<'_>) {
    if IGNORE_CHECK.contains(&ctx.invoked_command_name()) {
        return;
    }
    trace!(
        "Running pre_command for command {} for {}",
        ctx.command().qualified_name,
        ctx.author().id
    );

    let users_cache = ctx.data().users_cache.lock().await;
    let invocation_data =
        // Check the cache for a client, being a one liner immediately drops the lock
        if let Some(client) = users_cache.get(&ctx.author().id) {
            trace!("Loaded invocation data from cache");
            Ok(client.to_owned())
        } else {
            // Drop the lock as we are going to make some db requests, will re acquire it later when needed
            drop(users_cache);
            trace!("Failed to load invocation data from cache, requesting from database");
            let db_client = &ctx.data().connections.db;
            let amizone_conn = &ctx.data().connections.amizone;
            let caller_id = ctx.author().id.to_string();

            match amizoneapi::user::User::from_id(caller_id, db_client).await {
                Ok(user) => match user {
                    Some(user) => match user.get_client(amizone_conn.clone()) {
                        Ok(user_client) => {
                            trace!(
                                "User {} is logged in, pre_command succeeded.",
                                ctx.author().id
                            );
                            // Insert the found data in the cache, its scoped to immediately drop the lock (the scope is probably not needed)
                            {
                                ctx.data()
                                    .users_cache
                                    .lock()
                                    .await
                                    .insert(ctx.author().id, user_client.clone());
                            }
                            Ok(user_client)
                        }
                        Err(amizone_error) => {
                            debug!("Error in retrieving the client for {}", ctx.author().id);
                            Err(amizone_error.into())
                        }
                    },
                    None => {
                        trace!("User not logged in.");
                        Err(BotError::AmizoneError(AmizoneApiError::not_found(
                            "User not logged in",
                        )))
                    }
                },
                Err(dberror) => Err(dberror.into()),
            }
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
