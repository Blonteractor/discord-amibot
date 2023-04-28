use std::env;
use std::str::FromStr;
use std::time;

use amizone::api::{
    self as amizoneapi,
    types::{AmizoneConnection, DatabaseConnection},
};
use dotenv::dotenv;
use poise::serenity_prelude::{self as serenity, UserId};

#[allow(dead_code)]
struct Data {
    start_time: time::Instant,
    connections: Connections,
    dev_user_id: serenity::UserId,
    bot_user_id: serenity::UserId,
}

#[allow(dead_code)]
struct Connections {
    pub amizone: AmizoneConnection,
    pub db: DatabaseConnection,
}

impl Connections {
    async fn new() -> Self {
        Self {
            amizone: amizoneapi::new_amizone_connection(env::var("AMIZONE_API_URL").unwrap())
                .await
                .unwrap(),
            db: amizoneapi::new_db_connection(env::var("DATABASE_URL").unwrap())
                .await
                .unwrap(),
        }
    }
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(prefix_command)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                case_insensitive_commands: true,
                ..Default::default()
            },
            commands: vec![register()],
            ..Default::default()
        })
        .token(env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::all())
        .setup(|_, ready, _| {
            Box::pin(async move {
                let connections = Connections::new().await;
                let start_time = time::Instant::now();
                let dev_user_id =
                    UserId::from_str(&env::var("DEV_ID").unwrap_or_default()).unwrap_or_default();

                println!("Amibot is ready");
                Ok(Data {
                    start_time,
                    connections,
                    dev_user_id,
                    bot_user_id: ready.user.id,
                })
            })
        });

    framework.run().await.unwrap();
}
