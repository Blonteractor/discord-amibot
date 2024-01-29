use amizone::api::client::UserClient;

use crate::{CommandResult, Context, Result};

static ATTENDANCE_HELP: &str ="/attendance - Retrieves your attendance records for the current semester.

Usage: /attendance
    
Example: /attendance
    
Note: This command requires you to be logged in using the /login command. If you are not logged in, you will be prompted to do so first.";

///  Retrieves your attendance records for the current semester.
#[poise::command(prefix_command, slash_command, help_text_fn = "attendance_help")]
pub async fn attendance(ctx: Context<'_>) -> CommandResult {
    ctx.defer_ephemeral().await?;
    let mut invocation_data = ctx.invocation_data::<Result<UserClient>>().await.unwrap();

    let client = invocation_data.as_mut()?;

    let records = client.get_attendance().await?;

    let mut message = String::from("```");
    // let mut message = String::new;

    for record in records {
        let (code, name) = match &record.course {
            Some(course) => (course.code.as_str(), course.name.as_str()),
            _ => ("", ""),
        };

        let (attended, held) = match &record.attendance {
            Some(attendance) => (attendance.attended, attendance.held),
            _ => (-1, -1),
        };

        let percentage = (attended as f64 / held as f64) * 100.0;
        let percentage_str = format!("{:.2}%", percentage);

        let emoji;
        if percentage >= 85.0 {
            emoji = "👍";
        } else if percentage >= 75.0 {
            emoji = "🚨";
        } else {
            emoji = "👎";
        }

        // message.push_str(&format!("📚 **{} ({})**\n", name, code));
        message.push_str(&format!("📚 {} ({})\n", name, code));
        message.push_str(&format!("✅ Attended: {}\n", attended));
        // message.push_str(&format!(
        //     "📅 {}% ({}/{}) {}\n\n",
        //     percentage_str, attended, held, emoji
        // ));
        message.push_str(&format!("📅 Held: {}\n", held));
        message.push_str(&format!("{} Percentage: {}\n\n", emoji, percentage_str));
    }

    message.push_str("```");

    ctx.say(message).await?;

    Ok(())
}

fn attendance_help() -> String {
    ATTENDANCE_HELP.into()
}
