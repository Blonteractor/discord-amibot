use std::ops::Deref;

use crate::util;
use crate::{CommandResult, Context, Result};
use amizone::api::client::UserClient;
use amizone::api::types::Course;
use poise::serenity_prelude::CreateEmbed;

static DATESHEET_HELP: &'static str ="/datesheet - Retrieves your datesheet for upcoming examination.

Usage: /datesheet
    
Example: /datesheet
    
Note: This command requires you to be logged in using the /login command. If you are not logged in, you will be prompted to do so first.";

/// Retrieves your datesheet for upcoming examination
#[poise::command(prefix_command, slash_command, help_text_fn = "datesheet_help")]
pub async fn courses(
    ctx: Context<'_>,
    #[description = "Semester number"] semester: Option<usize>,
) -> CommandResult {
    ctx.defer().await?;
    let mut invocation_data = ctx.invocation_data::<Result<UserClient>>().await.unwrap();

    let client = invocation_data.as_mut()?;
    let courses = match semester {
        Some(sem) => client.get_courses(sem).await?,
        None => client.get_current_courses().await?,
    }
    .into_iter()
    .map(|course| course.into())
    .collect::<Vec<AmizoneCourse>>();

    let options = courses
        .iter()
        .map(|course| match course.r#ref {
            Some(ref course_ref) => course_ref.name.as_str(),
            None => "",
        })
        .collect::<Vec<&str>>();

    util::make_select_menu(ctx, courses.as_slice(), options.as_slice()).await?;

    Ok(())
}

fn datesheet_help() -> String {
    DATESHEET_HELP.into()
}

#[derive(Clone)]
struct AmizoneCourse {
    inner: Course,
}

impl Deref for AmizoneCourse {
    type Target = Course;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Course> for AmizoneCourse {
    fn from(value: Course) -> Self {
        AmizoneCourse { inner: value }
    }
}

impl Into<CreateEmbed> for &AmizoneCourse {
    fn into(self) -> CreateEmbed {
        let (attended, held) = match &self.attendance {
            Some(attendance) => (attendance.attended, attendance.held),
            _ => (-1, -1),
        };

        let syllabus = &self.syllabus_doc;
        let (obtained_marks, max_marks) = match self.internal_marks {
            Some(ref marks) => (marks.have, marks.max),
            None => (-1f32, -1f32),
        };

        let course_type = &self.r#type;

        let (code, name) = match self.r#ref {
            Some(ref course_ref) => (course_ref.code.as_str(), course_ref.name.as_str()),
            None => ("", ""),
        };

        let percentage = (attended as f64 / held as f64) * 100.0;
        let percentage_str = format!("{:.2}%", percentage);

        CreateEmbed::default()
            .title(format!("{} *{}*", name, code))
            .url(syllabus)
            .field("Attended", attended, false)
            .field("Total Classes", held, false)
            .field("Marks Obtained", obtained_marks, false)
            .field("Max Marks", max_marks, false)
            .field("Course Type", course_type, false)
            .to_owned()
    }
}
