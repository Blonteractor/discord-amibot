use crate::{CommandResult, Context};

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

/// Show this menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> CommandResult {
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "\
Type ~help command for more info on a command.
You need to use /login before you can use most commands.",
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

/// Returns the link to the github repositary for this bot
#[poise::command(prefix_command, slash_command)]
pub async fn source(ctx: Context<'_>) -> CommandResult {
    ctx.say("https://github.com/blonteractor/discord-amibot")
        .await?;
    Ok(())
}
