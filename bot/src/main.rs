pub mod commands;

use std::env;

use std::str::FromStr;
use std::time;

use amizone::api::{
    self as amizoneapi,
    types::{AmizoneConnection, DatabaseConnection},
};
use dotenv::dotenv;
use poise::{
    serenity_prelude::{self as serenity, Context as SerenityContext, Ready, UserId},
    Framework,
};

#[allow(dead_code)]
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

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    println!("Encountered error: {}", error.to_string());
}

/// Show this menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> Result<(), Error> {
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "\
Type ?help command for more info on a command.
You can edit your message to the bot and the bot will edit its response.",
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

async fn on_ready(
    ctx: &SerenityContext,
    ready: &Ready,
    framework: &Framework<Data, Error>,
) -> Result<Data, Error> {
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
            on_error: |error| Box::pin(on_error(error)),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                case_insensitive_commands: true,
                ..Default::default()
            },
            commands: vec![help(), commands::login::login()],
            ..Default::default()
        })
        .token(env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::all())
        .setup(|ctx, ready, framework| Box::pin(on_ready(ctx, ready, framework)));

    framework.run().await.unwrap();
}
