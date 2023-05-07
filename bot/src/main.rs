pub mod callbacks;
pub mod commands;
pub mod error;
pub mod util;
use std::{collections::HashMap, env, sync::Arc};

use error::BotError;
use tokio::sync::Mutex;

use std::time;

use amizone::api::{
    client::UserClient,
    types::{AmizoneConnection, DatabaseConnection},
};
use dotenv::dotenv;
use poise::serenity_prelude::{self as serenity, Colour, UserId};

pub type Result<T> = std::result::Result<T, BotError>;
pub type CommandResult = Result<()>;
pub type Context<'a> = poise::Context<'a, Data, BotError>;
pub static IGNORE_CHECK: &[&str] = &["login", "help", "ping"];

pub struct ColourScheme {
    pub primary: Colour,
    pub secondary: Colour,
    pub tertiary: Colour,
}

impl ColourScheme {
    pub fn amity_colours() -> Self {
        Self {
            primary: Colour::from_rgb(245, 194, 44),
            secondary: Colour::from_rgb(14, 46, 78),
            tertiary: Colour::from_rgb(189, 189, 189),
        }
    }
}

/// Returns the ping of the heartbeat in ms
#[poise::command(prefix_command, slash_command)]
pub async fn ping(ctx: Context<'_>) -> CommandResult {
    let shard_manager = ctx.framework().shard_manager.lock().await;
    let runners = shard_manager.runners.lock().await;

    let ping = runners
        .iter()
        .filter(|(id, _)| id.0 == ctx.serenity_context().shard_id)
        .next()
        .unwrap()
        .1
        .latency
        .unwrap_or_default()
        .as_millis();

    drop(runners);
    drop(shard_manager);

    ctx.say(&format!("**{}ms**", ping)).await?;

    Ok(())
}

pub struct Data {
    pub start_time: time::Instant,
    pub connections: Connections,
    pub dev_user_id: serenity::UserId,
    pub bot_user_id: serenity::UserId,
    pub colourscheme: ColourScheme,
    pub users_cache: Arc<Mutex<HashMap<UserId, UserClient>>>,
}

pub struct Connections {
    pub amizone: AmizoneConnection,
    pub db: DatabaseConnection,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            on_error: |error| Box::pin(callbacks::on_error(error)),
            command_check: Some(|ctx| Box::pin(callbacks::loggedin_check(ctx))),
            pre_command: |ctx| Box::pin(callbacks::init_client(ctx)),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                case_insensitive_commands: true,
                ..Default::default()
            },
            commands: vec![
                ping(),
                commands::help::help(),
                commands::authentication::login::login(),
                commands::authentication::logout::logout(),
                commands::attendance::attendance(),
                commands::exam::datesheet(),
            ],
            ..Default::default()
        })
        .token(env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::all())
        .setup(|ctx, ready, framework| Box::pin(callbacks::on_ready(ctx, ready, framework)));

    framework.run().await.unwrap();
}
