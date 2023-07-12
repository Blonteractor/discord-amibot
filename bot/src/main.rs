pub mod callbacks;
pub mod commands;
pub mod error;
pub mod util;
use std::{collections::HashMap, sync::Arc};

use anyhow::Context as _;
use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;

use error::BotError;
use tokio::sync::Mutex;

use std::time;

use amizone::api::{
    client::UserClient,
    types::{AmizoneConnection, DatabaseConnection},
};

use poise::serenity_prelude::{self as serenity, Colour, UserId};

pub type Result<T> = std::result::Result<T, BotError>;
pub type CommandResult = Result<()>;
pub type Context<'a> = poise::Context<'a, Data, BotError>;
type ApplicationContext<'a> = poise::ApplicationContext<'a, Data, BotError>;
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
            tertiary: Colour::from_rgb(247, 143, 70),
        }
    }
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

#[shuttle_runtime::main]
async fn poise(
    #[shuttle_secrets::Secrets] secrets_store: SecretStore,
) -> ShuttlePoise<Data, BotError> {
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
                commands::meta::ping(),
                commands::meta::help(),
                commands::meta::source(),
                commands::authentication::login::login(),
                commands::authentication::logout::logout(),
                commands::attendance::attendance(),
                commands::exam::datesheet(),
                commands::courses::courses(),
                commands::mac::wifimac(),
                commands::profile::profile(),
                commands::faculty_feedback::facultyfeedback(),
                commands::schedule::schedule(),
            ],
            ..Default::default()
        })
        .token(
            secrets_store
                .get("DISCORD_TOKEN")
                .context("'DISCORD_TOKEN' was not found")?,
        )
        .intents(serenity::GatewayIntents::all())
        .setup(|ctx, ready, framework| {
            Box::pin(callbacks::on_ready(ctx, ready, framework, secrets_store))
        })
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(framework.into())
}
