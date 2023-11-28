use std::ops::{Deref, DerefMut};

use crate::{CommandResult, Context, Result};
use amizone::api::{
    client::UserClient,
    types::{AttendanceState, Date, ScheduledClass},
};
use chrono::{prelude::Utc, Datelike, FixedOffset};
use poise::serenity_prelude::CreateEmbed;

static SCHEDULE_HELP: &str = "/schedule - View the schedule of classes.\n\n\
Usage: /schedule [day] [month] [year]\n\n\
Aliases: tt, classes
Arguments:\n\
- [day]: Optional day of the month. If provided, it shows the schedule for the specified day. \
If not provided, it shows the schedule for today.\n\
- [month]: Optional month. If provided, it shows the schedule for the specified month. \
If not provided, it shows the schedule for the current month.\n\
- [year]: Optional year. If provided, it shows the schedule for the specified year. \
If not provided, it shows the schedule for the current year.\n\n\
Example:\n\
/schedule\n\
/schedule 15 5 2023\n\n\
Note: This command displays the schedule of classes. If you specify a day, month, and year, \
it shows the schedule for that specific date. Otherwise, it shows the schedule for today's date.\n\n";

/// View the schedule of classes
#[poise::command(
    prefix_command,
    slash_command,
    help_text_fn = "schedule_help",
    aliases("tt", "classes")
)]
pub async fn schedule(
    ctx: Context<'_>,
    #[description = "Day of month, defaults to today"] day: Option<String>,
    #[description = "Month, defaults to current month"] month: Option<String>,
    #[description = "Year, defualts to current year"] year: Option<String>,
) -> CommandResult {
    ctx.defer().await?;
    let mut invocation_data = ctx.invocation_data::<Result<UserClient>>().await.unwrap();

    let client = invocation_data.as_mut()?;

    let color = ctx.data().colourscheme.primary;
    let now = Utc::now()
        .with_timezone(&FixedOffset::east_opt(5 * 60 * 60 + 30 * 60).unwrap())
        .date_naive();

    let date = Date {
        day: match day {
            Some(day) => match day.parse() {
                Ok(day) => {
                    if day > 31 {
                        ctx.say("Invalid day.").await?;
                        return Ok(());
                    } else {
                        day
                    }
                }
                Err(_) => {
                    ctx.say("Invalid day.").await?;
                    return Ok(());
                }
            },
            None => now.day() as i32,
        },
        month: match month {
            Some(month) => match month.parse() {
                Ok(month) => {
                    if month > 12 {
                        ctx.say("Invalid month.").await?;
                        return Ok(());
                    } else {
                        month
                    }
                }
                Err(_) => {
                    ctx.say("Invalid month.").await?;
                    return Ok(());
                }
            },
            None => now.month() as i32,
        },
        year: match year {
            Some(year) => match year.parse() {
                Ok(year) => {
                    if !(2000..2100).contains(&year) {
                        ctx.say("Invalid year.").await?;
                        return Ok(());
                    } else {
                        year
                    }
                }
                Err(_) => {
                    ctx.say("Invalid year.").await?;
                    return Ok(());
                }
            },
            None => now.year(),
        },
    };

    let schedule = ScheduledClasses {
        schedule: client.get_class_schedule(date.clone()).await?,
        date: date.clone(),
    };

    if schedule.is_holiday() {
        ctx.say("There are no classes scheduled for that day.")
            .await?;
        return Ok(());
    }

    ctx.send(|b| {
        b.embed(|e| {
            *e = <&ScheduledClasses as Into<CreateEmbed>>::into(&schedule);
            e.color(color);
            e
        })
    })
    .await?;

    Ok(())
}

fn schedule_help() -> String {
    SCHEDULE_HELP.into()
}

struct ScheduledClasses {
    schedule: Vec<ScheduledClass>,
    date: Date,
}

impl ScheduledClasses {
    pub fn is_holiday(&self) -> bool {
        self.schedule.is_empty()
    }
}

impl Deref for ScheduledClasses {
    type Target = Vec<ScheduledClass>;

    fn deref(&self) -> &Self::Target {
        &self.schedule
    }
}

impl DerefMut for ScheduledClasses {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.schedule
    }
}

impl From<&ScheduledClasses> for CreateEmbed {
    fn from(value: &ScheduledClasses) -> Self {
        let mut embed = CreateEmbed::default();

        embed.title(format!(
            "{}-{}-{}",
            value.date.day, value.date.month, value.date.year
        ));
        embed.description("_ _");

        for class in value.iter() {
            let course = &class.course.as_ref().unwrap();
            let name = format!("{} {}", course.code, course.name);

            let faculty = &class.faculty;
            let room = &class.room;

            let start = match &class.start_time {
                Some(time) => time
                    .to_string()
                    .split('T')
                    .nth(1)
                    .unwrap()
                    .split('Z')
                    .next()
                    .unwrap()
                    .to_string(),
                None => String::from("?"),
            };

            let end = match &class.end_time {
                Some(time) => time
                    .to_string()
                    .split('T')
                    .nth(1)
                    .unwrap()
                    .split('Z')
                    .next()
                    .unwrap()
                    .to_string(),
                None => String::from("?"),
            };

            let attendance = match &class.attendance.into() {
                AttendanceState::Pending => "🔵",
                AttendanceState::Present => "🟢",
                AttendanceState::Absent => "🔴",
                AttendanceState::Invalid | AttendanceState::Na => "❔",
            };

            embed.field(
                format!("🕔 {} - {}", start, end),
                format!("{} {} - {} \n 📍 {} \n", attendance, name, faculty, room),
                false,
            );
        }
        embed
    }
}
