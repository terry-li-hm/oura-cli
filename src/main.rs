use anyhow::{Context, Result};
use chrono::{Duration, Local, NaiveDate, NaiveDateTime, Timelike};
use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;

mod client;
mod db;
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
    /// Actionable sleep analysis
    Analyze {
        /// Date: YYYY-MM-DD, "today", or "yesterday"
        date: Option<String>,
    },
    /// Sleep stage hypnogram (5-min intervals)
    Hypnogram {
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
    /// Sync one night's sleep into DuckDB
    Sync {
        /// Date: YYYY-MM-DD, "today", or "yesterday"
        date: Option<String>,
    },
    /// This week versus 4-week average from DuckDB
    Weekly,
    /// 30-day readiness and bedtime ASCII trends from DuckDB
    Trend,
    /// Compare the 7 days before and after an event
    Event { date: NaiveDate, label: String },
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

fn parse_date(input: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(input, "%Y-%m-%d")
        .with_context(|| format!("Invalid date format: {input}"))
}

fn select_sleep_record(records: &[models::Sleep]) -> Option<&models::Sleep> {
    records
        .iter()
        .find(|record| record.sleep_type.as_deref() == Some("long_sleep"))
        .or(records.first())
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
            display::display_scores(sleep.first(), readiness.first(), activity.first());
        }
        Command::Sleep { date } => {
            let d = resolve_date(date.as_deref());
            let sleep = client.sleep(&d)?;
            let daily = client.daily_sleep(&d)?;
            display::display_sleep(daily.first(), &sleep);
        }
        Command::Analyze { date } => {
            let d = resolve_date(date.as_deref());
            let sleep = client.sleep(&d)?;
            let daily_sleep = client.daily_sleep(&d)?;
            let daily_readiness = client.daily_readiness(&d)?;
            display::display_analyze(daily_sleep.first(), daily_readiness.first(), &sleep);
        }
        Command::Hypnogram { date } => {
            let d = resolve_date(date.as_deref());
            let sleep = client.sleep(&d)?;
            let daily = client.daily_sleep(&d)?;
            display::display_hypnogram(daily.first(), &sleep);
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
            let daily = client.daily_sleep(&d)?;
            display::display_hrv(daily.first(), &sleep);
        }
        Command::Stress { date } => {
            let d = resolve_date(date.as_deref());
            let data = client.daily_stress(&d)?;
            display::display_stress(data.first());
        }
        Command::Sync { date } => {
            let resolved = resolve_date(date.as_deref());
            let parsed_date = parse_date(&resolved)?;
            let sleep = client.sleep(&resolved)?;
            let daily_sleep = client.daily_sleep(&resolved)?;
            let daily_readiness = client.daily_readiness(&resolved)?;
            let record = db::build_nightly_sleep_record(
                parsed_date,
                daily_sleep.first(),
                daily_readiness.first(),
                select_sleep_record(&sleep),
            )?;
            let conn = db::open_db()?;
            db::upsert_nightly_sleep(&conn, &record)?;
            println!("Synced {resolved}");
        }
        Command::Weekly => {
            let conn = db::open_db()?;
            weekly_view(&conn)?;
        }
        Command::Trend => {
            let conn = db::open_db()?;
            trend_view(&conn)?;
        }
        Command::Event { date, label } => {
            let conn = db::open_db()?;
            event_view(&conn, date, label)?;
        }
        Command::Json { endpoint, date } => {
            let d = resolve_date(date.as_deref());
            let json = client.raw(&endpoint, &d)?;
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
    }

    Ok(())
}

fn weekly_view(conn: &duckdb::Connection) -> Result<()> {
    let today = Local::now().date_naive();
    let week_start = today - Duration::days(7);
    let month_start = today - Duration::days(28);
    let yesterday = today - Duration::days(1);

    let week_data = db::fetch_sleep_window(conn, week_start, yesterday)?;
    let month_data = db::fetch_sleep_window(conn, month_start, yesterday)?;

    println!(
        "{:<20} | {:<10} | {:<10} | {:<5}",
        "Metric", "This Week", "4W Avg", "Δ"
    );
    println!("{}", "-".repeat(55));

    print_metric(
        "Readiness",
        &week_data,
        &month_data,
        |day| day.readiness_score.map(|value| value as f64),
        true,
        ValueFormat::OneDecimal,
        DeltaDirection::LeftMinusRight,
    );
    print_metric(
        "Sleep Duration",
        &week_data,
        &month_data,
        |day| day.total_sleep_duration.map(|value| value as f64),
        true,
        ValueFormat::Hours,
        DeltaDirection::LeftMinusRight,
    );
    print_metric(
        "Efficiency",
        &week_data,
        &month_data,
        |day| day.efficiency.map(|value| value as f64),
        true,
        ValueFormat::Percent,
        DeltaDirection::LeftMinusRight,
    );
    print_metric(
        "Deep Sleep",
        &week_data,
        &month_data,
        |day| day.deep_sleep_duration.map(|value| value as f64),
        true,
        ValueFormat::Minutes,
        DeltaDirection::LeftMinusRight,
    );
    print_metric(
        "REM Sleep",
        &week_data,
        &month_data,
        |day| day.rem_sleep_duration.map(|value| value as f64),
        true,
        ValueFormat::Minutes,
        DeltaDirection::LeftMinusRight,
    );
    print_metric(
        "Avg HRV",
        &week_data,
        &month_data,
        |day| day.average_hrv,
        true,
        ValueFormat::Milliseconds,
        DeltaDirection::LeftMinusRight,
    );
    print_metric(
        "Avg HR",
        &week_data,
        &month_data,
        |day| day.average_heart_rate,
        false,
        ValueFormat::Bpm,
        DeltaDirection::LeftMinusRight,
    );

    let week_bedtime = average_bedtime(&week_data);
    let month_bedtime = average_bedtime(&month_data);
    let diff = match (week_bedtime, month_bedtime) {
        (Some(week), Some(month)) => Some(week - month),
        _ => None,
    };

    let diff_str = match diff {
        Some(value) => {
            let text = format!("{value:+}m");
            if value >= 0 {
                format!("{}", text.green())
            } else {
                format!("{}", text.red())
            }
        }
        None => "--".to_string(),
    };

    println!(
        "{:<20} | {:<10} | {:<10} | {}",
        "Bedtime",
        format_bedtime(week_bedtime),
        format_bedtime(month_bedtime),
        diff_str
    );

    Ok(())
}

fn trend_view(conn: &duckdb::Connection) -> Result<()> {
    let today = Local::now().date_naive();
    let start_date = today - Duration::days(30);
    let end_date = today - Duration::days(1);
    let data = db::fetch_sleep_window(conn, start_date, end_date)?;

    if data.is_empty() {
        println!("No data for the last 30 days.");
        return Ok(());
    }

    println!("\n--- 30-Day Readiness Trend ---");
    let readiness_points: Vec<Option<i32>> = data.iter().map(|day| day.readiness_score).collect();
    draw_ascii_chart(&readiness_points, 0, 100, 20);

    println!("\n--- 30-Day Bedtime Trend (HKT) ---");
    let bedtime_points: Vec<Option<i32>> = data
        .iter()
        .map(|day| bedtime_to_mins_after_noon(day.bedtime_start))
        .collect();
    draw_ascii_chart(&bedtime_points, 21 * 60, 25 * 60, 20);
    println!("  21:00{:>25}01:00", "");

    let readiness_vals: Vec<(f64, f64)> = data
        .iter()
        .enumerate()
        .filter_map(|(index, day)| {
            day.readiness_score
                .map(|score| (index as f64, score as f64))
        })
        .collect();
    let bedtime_vals: Vec<(f64, f64)> = data
        .iter()
        .enumerate()
        .filter_map(|(index, day)| {
            bedtime_to_mins_after_noon(day.bedtime_start)
                .map(|minutes| (index as f64, minutes as f64))
        })
        .collect();

    if let Some(slope) = linear_regression_slope(&readiness_vals) {
        let total_change = slope * 30.0;
        println!(
            "\nReadiness trend: {:.1} points {}",
            total_change.abs(),
            if total_change >= 0.0 { "up" } else { "down" }
        );
    }

    if let Some(slope) = linear_regression_slope(&bedtime_vals) {
        let total_change = slope * 30.0;
        println!(
            "Bedtime trend: {:.1} min {}",
            total_change.abs(),
            if total_change >= 0.0 {
                "later"
            } else {
                "earlier"
            }
        );
    }

    Ok(())
}

fn event_view(conn: &duckdb::Connection, date: NaiveDate, label: String) -> Result<()> {
    let before_start = date - Duration::days(7);
    let after_end = date + Duration::days(7);
    let before_data = db::fetch_sleep_window(conn, before_start, date - Duration::days(1))?;
    let after_data = db::fetch_sleep_window(conn, date, after_end)?;

    println!("\nEvent: {} ({})", label.bold(), date);
    println!(
        "{:<20} | {:<12} | {:<12} | {:<5}",
        "Metric", "7d Before", "7d After", "Δ"
    );
    println!("{}", "-".repeat(60));

    print_metric(
        "Readiness",
        &before_data,
        &after_data,
        |day| day.readiness_score.map(|value| value as f64),
        true,
        ValueFormat::OneDecimal,
        DeltaDirection::RightMinusLeft,
    );
    print_metric(
        "Sleep Duration",
        &before_data,
        &after_data,
        |day| day.total_sleep_duration.map(|value| value as f64),
        true,
        ValueFormat::Hours,
        DeltaDirection::RightMinusLeft,
    );
    print_metric(
        "Efficiency",
        &before_data,
        &after_data,
        |day| day.efficiency.map(|value| value as f64),
        true,
        ValueFormat::Percent,
        DeltaDirection::RightMinusLeft,
    );
    print_metric(
        "Deep Sleep",
        &before_data,
        &after_data,
        |day| day.deep_sleep_duration.map(|value| value as f64),
        true,
        ValueFormat::Minutes,
        DeltaDirection::RightMinusLeft,
    );
    print_metric(
        "Avg HRV",
        &before_data,
        &after_data,
        |day| day.average_hrv,
        true,
        ValueFormat::Milliseconds,
        DeltaDirection::RightMinusLeft,
    );

    let before_bedtime = average_bedtime(&before_data);
    let after_bedtime = average_bedtime(&after_data);
    let diff = match (before_bedtime, after_bedtime) {
        (Some(before), Some(after)) => Some(after - before),
        _ => None,
    };

    let diff_str = match diff {
        Some(value) => {
            let text = format!("{value:+}m");
            if value <= 0 {
                format!("{}", text.green())
            } else {
                format!("{}", text.red())
            }
        }
        None => "--".to_string(),
    };

    println!(
        "{:<20} | {:<12} | {:<12} | {}",
        "Bedtime",
        format_bedtime(before_bedtime),
        format_bedtime(after_bedtime),
        diff_str
    );

    println!(
        "\n(Note: Based on {} days before and {} days after)",
        before_data.len(),
        after_data.len()
    );

    Ok(())
}

fn draw_ascii_chart(points: &[Option<i32>], min_val: i32, max_val: i32, rows: i32) {
    for row in (0..rows).rev() {
        let threshold = min_val + (max_val - min_val) * row / rows;
        print!("{:>4} | ", threshold);
        for point in points {
            match point {
                Some(value) if *value >= threshold => print!("*"),
                Some(_) => print!(" "),
                None => print!("."),
            }
        }
        println!();
    }
    println!("     +{}", "-".repeat(points.len()));
}

fn linear_regression_slope(data: &[(f64, f64)]) -> Option<f64> {
    if data.len() < 2 {
        return None;
    }

    let count = data.len() as f64;
    let sum_x = data.iter().map(|(x, _)| x).sum::<f64>();
    let sum_y = data.iter().map(|(_, y)| y).sum::<f64>();
    let sum_xy = data.iter().map(|(x, y)| x * y).sum::<f64>();
    let sum_xx = data.iter().map(|(x, _)| x * x).sum::<f64>();
    let denominator = count * sum_xx - sum_x * sum_x;

    if denominator == 0.0 {
        return None;
    }

    Some((count * sum_xy - sum_x * sum_y) / denominator)
}

fn bedtime_to_mins_after_noon(datetime: Option<NaiveDateTime>) -> Option<i32> {
    datetime.map(|timestamp| {
        let hour = timestamp.hour();
        let mut total = (hour as i32 * 60) + timestamp.minute() as i32;
        if hour < 12 {
            total += 24 * 60;
        }
        total
    })
}

fn average_bedtime(data: &[db::SleepDayData]) -> Option<i32> {
    let values: Vec<i32> = data
        .iter()
        .filter_map(|day| bedtime_to_mins_after_noon(day.bedtime_start))
        .collect();

    if values.is_empty() {
        return None;
    }

    Some(values.iter().sum::<i32>() / values.len() as i32)
}

fn format_bedtime(minutes: Option<i32>) -> String {
    match minutes {
        Some(value) => {
            let hour = (value / 60) % 24;
            let minute = value % 60;
            format!("{hour:02}:{minute:02}")
        }
        None => "--:--".to_string(),
    }
}

#[derive(Clone, Copy)]
enum ValueFormat {
    OneDecimal,
    Hours,
    Percent,
    Minutes,
    Milliseconds,
    Bpm,
}

#[derive(Clone, Copy)]
enum DeltaDirection {
    LeftMinusRight,
    RightMinusLeft,
}

fn print_metric<F>(
    name: &str,
    baseline: &[db::SleepDayData],
    comparison: &[db::SleepDayData],
    extractor: F,
    higher_is_better: bool,
    format: ValueFormat,
    delta_direction: DeltaDirection,
) where
    F: Fn(&db::SleepDayData) -> Option<f64>,
{
    let baseline_values: Vec<f64> = baseline.iter().filter_map(&extractor).collect();
    let comparison_values: Vec<f64> = comparison.iter().filter_map(extractor).collect();

    let baseline_avg = average(&baseline_values);
    let comparison_avg = average(&comparison_values);

    let diff_str = match (baseline_avg, comparison_avg) {
        (Some(left), Some(right)) => {
            let diff = match delta_direction {
                DeltaDirection::LeftMinusRight => left - right,
                DeltaDirection::RightMinusLeft => right - left,
            };
            let good = (higher_is_better && diff >= 0.0) || (!higher_is_better && diff <= 0.0);
            let text = format_signed_value(diff, format);
            if good {
                format!("{}", text.green())
            } else {
                format!("{}", text.red())
            }
        }
        _ => "--".to_string(),
    };

    println!(
        "{:<20} | {:<10} | {:<10} | {}",
        name,
        baseline_avg
            .map(|value| format_value(value, format))
            .unwrap_or_else(|| "No data".to_string()),
        comparison_avg
            .map(|value| format_value(value, format))
            .unwrap_or_else(|| "No data".to_string()),
        diff_str
    );
}

fn average(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    Some(values.iter().sum::<f64>() / values.len() as f64)
}

fn format_value(value: f64, format: ValueFormat) -> String {
    match format {
        ValueFormat::OneDecimal => format!("{value:.1}"),
        ValueFormat::Hours => format!("{:.1}h", value / 3600.0),
        ValueFormat::Percent => format!("{value:.1}%"),
        ValueFormat::Minutes => format!("{:.0}m", value / 60.0),
        ValueFormat::Milliseconds => format!("{value:.1}ms"),
        ValueFormat::Bpm => format!("{value:.1} bpm"),
    }
}

fn format_signed_value(value: f64, format: ValueFormat) -> String {
    match format {
        ValueFormat::OneDecimal => format!("{value:+.1}"),
        ValueFormat::Hours => format!("{:+.1}h", value / 3600.0),
        ValueFormat::Percent => format!("{value:+.1}%"),
        ValueFormat::Minutes => format!("{:+.0}m", value / 60.0),
        ValueFormat::Milliseconds => format!("{value:+.1}ms"),
        ValueFormat::Bpm => format!("{value:+.1} bpm"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sleep_record(kind: Option<&str>) -> models::Sleep {
        models::Sleep {
            day: "2026-03-10".to_string(),
            sleep_type: kind.map(str::to_string),
            period: None,
            bedtime_start: None,
            bedtime_end: None,
            sleep_phase_5_min: None,
            sleep_phase_30_sec: None,
            app_sleep_phase_5_min: None,
            movement_30_sec: None,
            heart_rate: None,
            hrv: None,
            total_sleep_duration: None,
            time_in_bed: None,
            efficiency: None,
            latency: None,
            deep_sleep_duration: None,
            light_sleep_duration: None,
            rem_sleep_duration: None,
            awake_time: None,
            restless_periods: None,
            average_breath: None,
            average_heart_rate: None,
            average_hrv: None,
            lowest_heart_rate: None,
            readiness_score_delta: None,
            sleep_score_delta: None,
            low_battery_alert: None,
        }
    }

    #[test]
    fn prefers_long_sleep_record() {
        let records = vec![sleep_record(Some("nap")), sleep_record(Some("long_sleep"))];

        let selected = select_sleep_record(&records).unwrap();

        assert_eq!(selected.sleep_type.as_deref(), Some("long_sleep"));
    }

    #[test]
    fn falls_back_to_first_record_when_no_long_sleep_exists() {
        let records = vec![sleep_record(Some("nap")), sleep_record(Some("rest"))];

        let selected = select_sleep_record(&records).unwrap();

        assert_eq!(selected.sleep_type.as_deref(), Some("nap"));
    }
}
