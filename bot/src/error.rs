use amizone::api::types::{AmizoneApiError, DbError, StatusCode as ApiStatusCode};
use log::{debug, info};
use poise::serenity_prelude::{self as serenity, SerenityError};

#[derive(Debug)]
pub enum BotError {
    AmizoneError(AmizoneApiError),
    SerenityError(SerenityError),
    DbError(DbError),
}

impl From<serenity::Error> for BotError {
    fn from(value: serenity::Error) -> Self {
        BotError::SerenityError(value)
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

impl std::fmt::Display for BotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BotError::AmizoneError(err) => write!(f, "Amizone error: {}", err),
            BotError::SerenityError(err) => write!(f, "Serenity error: {}", err),
            BotError::DbError(err) => write!(f, "Database error: {}", err),
        }
    }
}

pub async fn on_error(error: poise::FrameworkError<'_, crate::Data, BotError>) {
    info!("Encountered error => {}", error);

    // Error during a command
    if let poise::structs::FrameworkError::Command { error, ctx } = error {
        debug!("Error during command {}", error);
        match error {
            BotError::AmizoneError(err) => {
                if let ApiStatusCode::Unavailable = err.code() {
                    ctx.say("Operation failed, amizone might be down.")
                        .await
                        .unwrap();
                    return;
                }
            }
            BotError::SerenityError(_) => todo!(),
            BotError::DbError(_) => todo!(),
        }
    }
}
