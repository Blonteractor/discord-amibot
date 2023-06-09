use std::sync::Arc;

use amizone::api::types::{AmizoneApiError, DbError, StatusCode as ApiStatusCode};
use log::debug;
use poise::serenity_prelude::{self as serenity, SerenityError};

use crate::Context;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum BotError {
    AmizoneError(AmizoneApiError),
    SerenityError(Arc<SerenityError>),
    DbError(DbError),
    Custom(String),
}

impl From<serenity::Error> for BotError {
    fn from(value: serenity::Error) -> Self {
        BotError::SerenityError(Arc::new(value))
    }
}

impl From<AmizoneApiError> for BotError {
    fn from(value: AmizoneApiError) -> Self {
        BotError::AmizoneError(value)
    }
}

impl From<DbError> for BotError {
    fn from(value: DbError) -> Self {
        BotError::DbError(value)
    }
}

impl From<&mut BotError> for BotError {
    fn from(value: &mut BotError) -> Self {
        value.to_owned()
    }
}

impl From<&str> for BotError {
    fn from(value: &str) -> Self {
        BotError::Custom(value.to_string())
    }
}

impl From<String> for BotError {
    fn from(value: String) -> Self {
        BotError::Custom(value)
    }
}

impl BotError {
    pub async fn handle(&self, ctx: Context<'_>) {
        match self {
            BotError::AmizoneError(err) => {
                debug!("API Error: {}", err);
                if let ApiStatusCode::Internal = err.code() {
                    ctx.say(format!("Operation failed, {}", err.message()))
                        .await
                        .ok();
                    return;
                }
                ctx.say("Amizone API returned an error.").await.ok();
            }
            BotError::SerenityError(err) => {
                debug!("Discord Error: {}", err);
                ctx.say("Error with the command, file an issue.").await.ok();
            }
            BotError::DbError(err) => {
                debug!("Database Error: {}", err);
                ctx.say("Error retreving database.").await.ok();
            }
            BotError::Custom(err) => {
                debug!("Unhadled Custom Error: {}", err);
                ctx.say("Error with the command, file an issue.").await.ok();
            }
        }
    }
}

impl std::fmt::Display for BotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BotError::AmizoneError(err) => write!(f, "Amizone error: {}", err),
            BotError::SerenityError(err) => write!(f, "Serenity error: {}", err),
            BotError::DbError(err) => write!(f, "Database error: {}", err),
            BotError::Custom(err) => {
                write!(f, "Custom error () really should be handled): {}", err)
            }
        }
    }
}
