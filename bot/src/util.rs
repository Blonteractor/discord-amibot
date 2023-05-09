use poise::serenity_prelude::{self as serenity, CreateEmbed};

use crate::error::BotError;

// Re-implemetation of poise::builtinins::paginate to make a select menu instead
// Length of pages and options should be equal
pub async fn make_select_menu<'a, T: 'a>(
    ctx: crate::Context<'_>,
    pages: &'a [T],
    options: &[&str],
) -> Result<(), BotError>
where
    &'a T: Into<CreateEmbed>,
{
    if pages.len() != options.len() {
        return Err("Pages and options have an unequal len in make_select_menu".into());
    }

    let colour = ctx.data().colourscheme.primary;

    let embed_pages = pages
        .iter()
        .map(|c| c.into().color(colour).to_owned())
        .collect::<Vec<CreateEmbed>>();

    // Define some unique identifiers for the navigation buttons
    let ctx_id = ctx.id();
    let select_menu_id = format!("{}menu", ctx.id());
    ctx.send(|b| {
        b.embed(|b| {
            *b = embed_pages[0].clone();
            b
        })
        .components(|b| {
            b.create_action_row(|b| {
                b.create_select_menu(|m| {
                    m.custom_id(&select_menu_id).placeholder("Course").options(
                        |create_options_builder| {
                            for (i, option) in options.iter().enumerate() {
                                create_options_builder.create_option(|create_option| {
                                    create_option.label(option).value(i)
                                });
                            }
                            create_options_builder
                        },
                    )
                })
            })
        })
    })
    .await?;

    // Loop through incoming interactions
    while let Some(interaction) = serenity::CollectComponentInteraction::new(ctx)
        // We defined our interaction IDs to start with `ctx_id`. If they don't, some other command's button was pressed
        .filter(move |interaction| interaction.data.custom_id.starts_with(&ctx_id.to_string()))
        // Timeout when no choice has has been selected for 24 hours
        .timeout(std::time::Duration::from_secs(3600 * 24))
        .await
    {
        // Update the message with the new page contents
        interaction
            .create_interaction_response(ctx, |b| {
                b.kind(serenity::InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|b| {
                        // We set the value of the choice to index of the page, so we know its a usize
                        let option_number: usize =
                            interaction.data.values[0].parse().unwrap_or_default();
                        b.embed(|b| {
                            *b = embed_pages[option_number].clone();
                            b
                        })
                    })
            })
            .await?;
    }

    Ok(())
}
