use amizone::api::client::UserClient;

use crate::{CommandResult, Context, Result};

static DATESHEET_HELP: &str = "/datesheet - Retrieve your datesheet for upcoming examinations.\n\n\
Usage: /datesheet\n\n\
Example:\n\
/datesheet\n\n\
Note: This command fetches and displays your datesheet for upcoming examinations.\
If you have any scheduled exams, it will show the course name, course code, date, and exam mode.\
If you don't have any upcoming exams, it will not return any information.";

/// Retrieves your datesheet for upcoming examination
#[poise::command(prefix_command, slash_command, help_text_fn = "datesheet_help")]
pub async fn datesheet(ctx: Context<'_>) -> CommandResult {
    ctx.defer().await?;
    let mut invocation_data = ctx.invocation_data::<Result<UserClient>>().await.unwrap();

    let client = invocation_data.as_mut()?;

    let (title, datesheet) = client.get_exam_schedule().await?;

    if datesheet.is_empty() {
        return Ok(());
    }

    let mut message = format!("**{}**```", title);
    // let mut message = String::new;

    for record in datesheet {
        let (code, name) = match &record.course {
            Some(course) => (course.code.as_str(), course.name.as_str()),
            _ => ("", ""),
        };

        let time = record
            .time
            .unwrap_or_default()
            .to_string()
            .replace('T', " ")
            .replace(":00Z", "");
        let mode = record.mode;

        message.push_str(&format!("📚 {} ({})\n", name, code));
        message.push_str(&format!("📅 {}\n", time));
        message.push_str(&format!("✍🏼 {}\n\n", mode));
    }

    message.push_str("```");

    ctx.say(message).await?;

    Ok(())
}

fn datesheet_help() -> String {
    DATESHEET_HELP.into()
}
