pub mod commands;
pub mod error;
use std::env;

use error::BotError;
use std::str::FromStr;
use std::time;

use amizone::api::{
    self as amizoneapi,
    client::UserClient,
    types::{AmizoneApiError, AmizoneConnection, DatabaseConnection},
};
use dotenv::dotenv;
use poise::{
    serenity_prelude::{self as serenity, Context as SerenityContext, Ready, UserId},
    Framework,
};

pub type Result<T> = std::result::Result<T, BotError>;
pub type CommandResult = Result<()>;
pub type Context<'a> = poise::Context<'a, Data, BotError>;

pub struct Data {
    pub start_time: time::Instant,
    pub connections: Connections,
    pub dev_user_id: serenity::UserId,
    pub bot_user_id: serenity::UserId,
}

pub struct Connections {
    pub amizone: AmizoneConnection,
    pub db: DatabaseConnection,
}

static IGNORE_CHECK: &[&'static str] = &["login", "help"];

/// Show this menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> CommandResult {
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "\
Type ?help command for more info on a command.
You can edit your message to the bot and the bot will edit its response.",
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

// Initialize user client before every* command
async fn init_client(ctx: Context<'_>) {
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

async fn loggedin_check(ctx: Context<'_>) -> Result<bool> {
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

async fn on_ready<'a>(
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

#[tokio::main]
async fn main() {
    dotenv().ok();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            on_error: |error| Box::pin(error::on_error(error)),
            command_check: Some(|ctx| Box::pin(loggedin_check(ctx))),
            pre_command: |ctx| Box::pin(init_client(ctx)),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                case_insensitive_commands: true,
                ..Default::default()
            },
            commands: vec![help(), commands::login::login(), commands::logout::logout()],
            ..Default::default()
        })
        .token(env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::all())
        .setup(|ctx, ready, framework| Box::pin(on_ready(ctx, ready, framework)));

    framework.run().await.unwrap();
}
