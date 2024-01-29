use std::ops::Deref;

use crate::util;
use crate::{CommandResult, Context, Result};
use amizone::api::client::UserClient;
use amizone::api::types::Course;
use poise::serenity_prelude::CreateEmbed;

static COURSES_HELP: &str ="/courses - Retrieve and select your courses.\n\n\
Usage: /courses [semester]\n\n\
Arguments:\n\
- [semester]: Optional semester number. If provided, it fetches the courses for the specified semester. \
If not provided, it fetches the courses for the current semester.\n\n\
Example:\n\
/courses\n\
/courses 3\n\n\
Note: This command retrieves and displays your courses. It allows you to select a course from the provided list \
of options. If you specify a semester, it fetches the courses for that specific semester. Otherwise, \
it fetches the courses for the current semester.";

/// Retrieve and select your courses
#[poise::command(prefix_command, slash_command, help_text_fn = "courses_help")]
pub async fn courses(
    ctx: Context<'_>,
    #[description = "Semester number"] semester: Option<usize>,
) -> CommandResult {
    ctx.defer_ephemeral().await?;
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

    util::make_select_menu(ctx, courses.as_slice(), options.as_slice(), "Select Course").await?;

    Ok(())
}

fn courses_help() -> String {
    COURSES_HELP.into()
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

impl From<&AmizoneCourse> for CreateEmbed {
    fn from(value: &AmizoneCourse) -> Self {
        let (attended, held) = match &value.attendance {
            Some(attendance) => (attendance.attended, attendance.held),
            _ => (-1, -1),
        };

        let syllabus = &value.syllabus_doc;
        let (obtained_marks, max_marks) = match value.internal_marks {
            Some(ref marks) => (marks.have, marks.max),
            None => (-1f32, -1f32),
        };

        let course_type = &value.r#type;

        let (code, name) = match value.r#ref {
            Some(ref course_ref) => (course_ref.code.as_str(), course_ref.name.as_str()),
            None => ("", ""),
        };

        let percentage = (attended as f64 / held as f64) * 100.0;
        let percentage_str = format!("*{:.2}%*", percentage);

        CreateEmbed::default()
            .title(format!("{} **{}**", name, code))
            .url(syllabus)
            .field(
                "Attendance",
                format!("`{}` / **{}** = {}", attended, held, percentage_str),
                true,
            )
            .field(
                "Internal Marks",
                format!("`{}` / **{}**", obtained_marks, max_marks),
                false,
            )
            .field("Course Type", course_type, false)
            .to_owned()
    }
}
