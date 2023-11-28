use crate::{ApplicationContext, CommandResult, Result};
use amizone::api::client::UserClient;
use poise::modal::execute_modal;
use poise::Modal;

static FACULTFEEDBACK_HELP: &str = "Fill out faculty feedback form.\n\n\
Usage: /facultyfeedback\n\n\
Aliases: /ff, /feedback\n\n\
Example:\n\
/facultyfeedback\n\n\
Note: This command allows you to fill out a faculty feedback form. It presents a modal with fields to input your feedback. \
The feedback includes a query, query rating, and comments. You can rate the query and query rating on a scale of 1 to 5. \
After submitting the form, it will indicate the success of filling out the faculty feedback.\n";

///  Fill out faculty feedback form for every faculty at once.
#[poise::command(
    slash_command,
    help_text_fn = "facultyfeedback_help",
    aliases("ff", "feedback")
)]
pub async fn facultyfeedback(ctx: ApplicationContext<'_>) -> CommandResult {
    let default_feedback = FeedbackForm {
        query: String::from("5"),
        query_rating: String::from("3"),
        comments: String::from("Nice."),
    };

    let feedback = match execute_modal(
        ctx,
        Some(default_feedback),
        Some(std::time::Duration::from_secs(60 * 5)),
    )
    .await?
    {
        Some(f) => f,
        None => {
            ctx.say("No input provided").await?;
            return Ok(());
        }
    };

    let query = match feedback.query.parse::<i32>() {
        Ok(query) => query,
        Err(_) => {
            ctx.say("Invalid query, expected a number from 1 to 5.")
                .await?;
            return Ok(());
        }
    };

    if !(1..=5).contains(&query) {
        ctx.say("Invalid query, expected a number from 1 to 5.")
            .await?;
        return Ok(());
    }

    let query_rating = match feedback.query_rating.parse::<i32>() {
        Ok(query_rating) => query_rating,
        Err(_) => {
            ctx.say("Invalid query rating, expected a number from 1 to 3.")
                .await?;
            return Ok(());
        }
    };

    if !(1..=3).contains(&query_rating) {
        ctx.say("Invalid query rating, expected a number from 1 to 3.")
            .await?;
        return Ok(());
    }

    let msg = ctx.say("*Filling faculty feedback...*").await?;

    ctx.defer().await?;
    let mut invocation_data = ctx.invocation_data::<Result<UserClient>>().await.unwrap();

    let client = invocation_data.as_mut()?;

    let filled = client
        .fill_faculty_feedback(query, query_rating, feedback.comments)
        .await?;

    let reply = if filled > 0 {
        format!("Successsfully filled feedback for `{}` faculties.", filled)
    } else {
        String::from("No faculty feedback exists for you.")
    };

    msg.edit(poise::Context::Application(ctx), |b| b.content(reply))
        .await?;

    Ok(())
}

fn facultyfeedback_help() -> String {
    FACULTFEEDBACK_HELP.into()
}

#[derive(Modal)]
#[name = "Feedback Form"]
struct FeedbackForm {
    #[name = "Rating"]
    #[min_length = 1]
    #[max_length = 1]
    query: String,

    #[name = "Query Rating"]
    #[min_length = 1]
    #[max_length = 1]
    query_rating: String,

    #[name = "Comments"]
    #[min_length = 5]
    comments: String,
}
