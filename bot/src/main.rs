pub mod callbacks;
pub mod commands;
pub mod error;
use std::env;

use error::BotError;

use std::time;

use amizone::api::types::{AmizoneConnection, DatabaseConnection};
use dotenv::dotenv;
use poise::serenity_prelude as serenity;

pub type Result<T> = std::result::Result<T, BotError>;
pub type CommandResult = Result<()>;
pub type Context<'a> = poise::Context<'a, Data, BotError>;
pub static IGNORE_CHECK: &[&str] = &["login", "help"];

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
