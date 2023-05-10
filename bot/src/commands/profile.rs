use crate::{CommandResult, Context, Result};
use amizone::api::client::UserClient;

static PROFILE_HELP: &'static str = "/profile - Retrieve and display your user profile information.\n\n\
Usage: /profile\n\n\
Example:\n\
/profile\n\n\
Note: This command fetches and displays your user profile information, including details like your name, enrollment number, program, batch, date of birth, blood group, validity, and ID number.\n\
The information is presented in an embedded format with different fields.";

/// Retrieve and display your user profile information.
#[poise::command(prefix_command, slash_command, help_text_fn = "profile_help")]
pub async fn profile(ctx: Context<'_>) -> CommandResult {
    ctx.defer().await?;
    let mut invocation_data = ctx.invocation_data::<Result<UserClient>>().await.unwrap();

    let client = invocation_data.as_mut()?;

    let profile = client.get_user_profile().await?;

    let batch = profile.batch;
    let name = profile.name;
    let blood_group = profile.blood_group;
    let dob = profile.date_of_birth.unwrap_or_default();
    let enrollement_number = profile.enrollment_number;
    let validity = profile.enrollment_validity.unwrap_or_default();
    let program = profile.program;
    let id_number = profile.id_card_number;
    let colour = ctx.data().colourscheme.tertiary;

    ctx.send(|b| {
        b.embed(|e| {
            e.color(colour)
                .title(name)
                .field("Enrollement Number", enrollement_number, true)
                .field("Program", program, true)
                .field("Batch", batch, true)
                .field(
                    "Date Of Birth",
                    dob.to_string().split("T").next().unwrap_or_default(),
                    true,
                )
                .field("Blood Group", blood_group, true)
                .field(
                    "Valid Till",
                    validity.to_string().split("T").next().unwrap_or_default(),
                    true,
                )
                .field("ID number", id_number, true)
        })
    })
    .await?;
    Ok(())
}

fn profile_help() -> String {
    PROFILE_HELP.into()
}
