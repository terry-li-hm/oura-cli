use anyhow::Result;
use chrono::{Days, Local};
use clap::{Parser, Subcommand};

mod client;
mod display;
#[allow(dead_code)]
mod models;

#[derive(Parser)]
#[command(
    name = "oura",
    version,
    about = "Oura Ring CLI — sleep, readiness, and activity from your terminal"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Sleep + readiness + activity scores (default)
    Scores {
        /// Date: YYYY-MM-DD, "today", or "yesterday"
        date: Option<String>,
    },
    /// Detailed sleep breakdown
    Sleep {
        /// Date: YYYY-MM-DD, "today", or "yesterday"
        date: Option<String>,
    },
    /// Readiness score and contributors
    Readiness {
        /// Date: YYYY-MM-DD, "today", or "yesterday"
        date: Option<String>,
    },
    /// Activity summary (steps, calories, movement)
    Activity {
        /// Date: YYYY-MM-DD, "today", or "yesterday"
        date: Option<String>,
    },
    /// Heart rate variability from sleep
    Hrv {
        /// Date: YYYY-MM-DD, "today", or "yesterday"
        date: Option<String>,
    },
    /// Daily stress summary
    Stress {
        /// Date: YYYY-MM-DD, "today", or "yesterday"
        date: Option<String>,
    },
    /// Score trend over the last N days (default: 7)
    Trend {
        /// Number of days to show
        #[arg(short, long, default_value = "7")]
        days: u32,
    },
    /// Raw JSON from any endpoint (for piping)
    Json {
        /// API endpoint (e.g. daily_sleep, sleep, daily_activity, daily_stress)
        endpoint: String,
        /// Date: YYYY-MM-DD, "today", or "yesterday"
        date: Option<String>,
    },
}

fn resolve_date(input: Option<&str>) -> String {
    let today = Local::now().date_naive();
    match input.map(|s| s.to_lowercase()).as_deref() {
        None | Some("today") => today.format("%Y-%m-%d").to_string(),
        Some("yesterday") => today
            .pred_opt()
            .expect("date underflow")
            .format("%Y-%m-%d")
            .to_string(),
        Some(date) => date.to_string(),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = client::OuraClient::new()?;
    let cmd = cli.command.unwrap_or(Command::Scores { date: None });

    match cmd {
        Command::Scores { date } => {
            let d = resolve_date(date.as_deref());
            let sleep = client.daily_sleep(&d)?;
            let readiness = client.daily_readiness(&d)?;
            let activity = client.daily_activity(&d)?;
            display::display_scores(
                sleep.first().and_then(|s| s.score),
                readiness.first().and_then(|r| r.score),
                activity.first().and_then(|a| a.score),
            );
        }
        Command::Sleep { date } => {
            let d = resolve_date(date.as_deref());
            let sleep = client.sleep(&d)?;
            let daily = client.daily_sleep(&d)?;
            let score = daily.first().and_then(|s| s.score);
            display::display_sleep(score, &sleep);
        }
        Command::Readiness { date } => {
            let d = resolve_date(date.as_deref());
            let data = client.daily_readiness(&d)?;
            display::display_readiness(data.first());
        }
        Command::Activity { date } => {
            let d = resolve_date(date.as_deref());
            let data = client.daily_activity(&d)?;
            display::display_activity(data.first());
        }
        Command::Hrv { date } => {
            let d = resolve_date(date.as_deref());
            let sleep = client.sleep(&d)?;
            display::display_hrv(&sleep);
        }
        Command::Stress { date } => {
            let d = resolve_date(date.as_deref());
            let data = client.daily_stress(&d)?;
            display::display_stress(data.first());
        }
        Command::Trend { days } => {
            let today = Local::now().date_naive();
            let start = today
                .checked_sub_days(Days::new((days - 1) as u64))
                .expect("date underflow");
            let start_str = start.format("%Y-%m-%d").to_string();
            let end_str = today.format("%Y-%m-%d").to_string();

            let sleep = client.daily_sleep_range(&start_str, &end_str)?;
            let readiness = client.daily_readiness_range(&start_str, &end_str)?;
            let activity = client.daily_activity_range(&start_str, &end_str)?;

            // Build list of all dates in range
            let mut date_list = Vec::new();
            let mut d = start;
            while d <= today {
                date_list.push(d.format("%Y-%m-%d").to_string());
                d = d.succ_opt().expect("date overflow");
            }

            display::display_trend(&date_list, &sleep, &readiness, &activity);
        }
        Command::Json { endpoint, date } => {
            let d = resolve_date(date.as_deref());
            let json = client.raw(&endpoint, &d)?;
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
    }

    Ok(())
}
