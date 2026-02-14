use owo_colors::OwoColorize;

use crate::models::{DailyActivity, DailyReadiness, DailySleep, DailyStress, Sleep};

pub fn colored_score(score: i64) -> String {
    if score >= 85 {
        format!("{}", score.green())
    } else if score >= 70 {
        format!("{}", score.yellow())
    } else {
        format!("{}", score.red())
    }
}

pub fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let mins = (seconds % 3600) / 60;
    if hours > 0 {
        format!("{hours}h {mins:02}m")
    } else {
        format!("{mins}m")
    }
}

fn format_time(iso: &str) -> &str {
    // "2024-02-13T23:15:00+08:00" → "23:15"
    if let Some(t) = iso.find('T') {
        let rest = &iso[t + 1..];
        if rest.len() >= 5 {
            return &rest[..5];
        }
    }
    iso
}

fn format_percent(part: i64, total: i64) -> String {
    if total == 0 {
        return "0%".to_string();
    }
    format!("{}%", (part as f64 / total as f64 * 100.0).round() as i64)
}

fn format_contributor_key(key: &str) -> String {
    key.split('_')
        .map(|w| match w {
            "hrv" | "hr" | "spo2" => w.to_uppercase(),
            _ => {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => {
                        let mut s = f.to_uppercase().to_string();
                        s.push_str(c.as_str());
                        s
                    }
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn display_contributors(contributors: &serde_json::Value) {
    if let Some(obj) = contributors.as_object() {
        for (key, value) in obj {
            if let Some(score) = value.as_i64() {
                println!(
                    "  {:<24}{}",
                    format_contributor_key(key),
                    colored_score(score)
                );
            }
        }
    }
}

// --- Command display functions ---

pub fn display_scores(
    daily_sleep: Option<&DailySleep>,
    daily_readiness: Option<&DailyReadiness>,
    daily_activity: Option<&DailyActivity>,
) {
    let s = daily_sleep
        .and_then(|d| d.score)
        .map_or("--".dimmed().to_string(), colored_score);
    let r = daily_readiness
        .and_then(|d| d.score)
        .map_or("--".dimmed().to_string(), colored_score);
    let a = daily_activity
        .and_then(|d| d.score)
        .map_or("--".dimmed().to_string(), colored_score);
    println!("  Sleep {s}  Readiness {r}  Activity {a}");

    // Show readiness contributors (the most actionable breakdown)
    if let Some(ref c) = daily_readiness.and_then(|d| d.contributors.clone()) {
        println!();
        println!("  {}", "Readiness contributors:".dimmed());
        display_contributors(c);
    }

    // Show temperature deviation if notable
    if let Some(temp) = daily_readiness.and_then(|d| d.temperature_deviation) {
        if temp.abs() >= 0.5 {
            println!("  Temp Deviation:  {temp:+.1}°C");
        }
    }
}

pub fn display_sleep(daily: Option<&DailySleep>, records: &[Sleep]) {
    let sleep = records
        .iter()
        .find(|s| s.sleep_type.as_deref() == Some("long_sleep"))
        .or(records.first());

    let score = daily.and_then(|d| d.score);

    match sleep {
        Some(s) => {
            if let Some(v) = score {
                println!("  Sleep Score: {}", colored_score(v));
            }

            let total = s.total_sleep_duration.unwrap_or(0);
            println!("  Total Sleep: {}", format_duration(total));

            if let Some(eff) = s.efficiency {
                println!("  Efficiency:  {eff}%");
            }

            if let Some(deep) = s.deep_sleep_duration {
                println!(
                    "  Deep:        {} ({})",
                    format_duration(deep),
                    format_percent(deep, total)
                );
            }
            if let Some(rem) = s.rem_sleep_duration {
                println!(
                    "  REM:         {} ({})",
                    format_duration(rem),
                    format_percent(rem, total)
                );
            }
            if let Some(light) = s.light_sleep_duration {
                println!(
                    "  Light:       {} ({})",
                    format_duration(light),
                    format_percent(light, total)
                );
            }

            if let Some(hrv) = s.average_hrv {
                println!("  Avg HRV:     {hrv} ms");
            }
            if let Some(hr) = s.average_heart_rate {
                println!("  Avg HR:      {} bpm", hr.round() as i64);
            }
            if let Some(low) = s.lowest_heart_rate {
                println!("  Lowest HR:   {low} bpm");
            }

            if let (Some(start), Some(end)) = (&s.bedtime_start, &s.bedtime_end) {
                println!(
                    "  Bedtime:     {} → {}",
                    format_time(start),
                    format_time(end)
                );
            }
        }
        None => {
            // No period data yet — show score + contributors from daily_sleep
            if let Some(d) = daily {
                if let Some(v) = d.score {
                    println!("  Sleep Score: {}", colored_score(v));
                }
                if let Some(ref c) = d.contributors {
                    display_contributors(c);
                }
                println!("  {}", "(detailed breakdown not yet synced)".dimmed());
            } else {
                println!("  No sleep data");
            }
        }
    }
}

pub fn display_readiness(record: Option<&DailyReadiness>) {
    let Some(r) = record else {
        println!("  No readiness data");
        return;
    };

    if let Some(v) = r.score {
        println!("  Readiness Score: {}", colored_score(v));
    }

    if let Some(temp) = r.temperature_deviation {
        println!("  Temp Deviation:  {temp:+.1}°C");
    }

    if let Some(ref c) = r.contributors {
        display_contributors(c);
    }
}

pub fn display_activity(record: Option<&DailyActivity>) {
    let Some(a) = record else {
        println!("  No activity data");
        return;
    };

    if let Some(v) = a.score {
        println!("  Activity Score: {}", colored_score(v));
    }

    if let Some(steps) = a.steps {
        println!("  Steps:          {}", format_number(steps));
    }

    if let Some(total) = a.total_calories {
        let active = a.active_calories.unwrap_or(0);
        println!(
            "  Calories:       {} (active: {})",
            format_number(total),
            format_number(active)
        );
    }

    if let Some(dist) = a.equivalent_walking_distance {
        println!("  Walking Dist:   {:.1} km", dist as f64 / 1000.0);
    }

    if let Some(high) = a.high_activity_time {
        println!("  High Activity:  {}", format_duration(high));
    }
    if let Some(med) = a.medium_activity_time {
        println!("  Med Activity:   {}", format_duration(med));
    }
    if let Some(low) = a.low_activity_time {
        println!("  Low Activity:   {}", format_duration(low));
    }
}

pub fn display_hrv(daily: Option<&DailySleep>, records: &[Sleep]) {
    let sleep = records
        .iter()
        .find(|s| s.sleep_type.as_deref() == Some("long_sleep"))
        .or(records.first());

    match sleep {
        Some(s) => {
            println!("  {}", "HRV (from sleep)".dimmed());

            if let Some(hrv) = s.average_hrv {
                println!("  Avg HRV:     {hrv} ms");
            } else {
                println!("  Avg HRV:     --");
            }

            if let Some(hr) = s.average_heart_rate {
                println!("  Avg HR:      {} bpm", hr.round() as i64);
            }
            if let Some(low) = s.lowest_heart_rate {
                println!("  Lowest HR:   {low} bpm");
            }
            if let Some(breath) = s.average_breath {
                println!("  Avg Breath:  {breath:.1} rpm");
            }
        }
        None => {
            if let Some(d) = daily {
                if let Some(v) = d.score {
                    println!(
                        "  Sleep Score: {} {}",
                        colored_score(v),
                        "(HRV requires detailed sync)".dimmed()
                    );
                }
            } else {
                println!("  No sleep data for HRV");
            }
        }
    }
}

pub fn display_stress(record: Option<&DailyStress>) {
    let Some(s) = record else {
        println!("  No stress data");
        return;
    };

    if let Some(ref summary) = s.day_summary {
        let colored = match summary.as_str() {
            "restored" => summary.green().to_string(),
            "normal" => summary.yellow().to_string(),
            "stressful" => summary.red().to_string(),
            _ => summary.clone(),
        };
        println!("  Stress Summary: {colored}");
    }

    if let Some(stress) = s.stress_high {
        println!("  Stress High:    {}", format_duration(stress));
    }
    if let Some(recovery) = s.recovery_high {
        println!("  Recovery High:  {}", format_duration(recovery));
    }
}

pub fn display_trend(
    days: &[String],
    sleep: &[crate::models::DailySleep],
    readiness: &[crate::models::DailyReadiness],
    activity: &[crate::models::DailyActivity],
) {
    use std::collections::HashMap;

    let sleep_map: HashMap<&str, Option<i64>> =
        sleep.iter().map(|s| (s.day.as_str(), s.score)).collect();
    let readiness_map: HashMap<&str, Option<i64>> = readiness
        .iter()
        .map(|r| (r.day.as_str(), r.score))
        .collect();
    let activity_map: HashMap<&str, Option<i64>> =
        activity.iter().map(|a| (a.day.as_str(), a.score)).collect();

    println!(
        "  {}",
        format!(
            "{:<12}{:>7}{:>11}{:>10}",
            "Date", "Sleep", "Readiness", "Activity"
        )
        .dimmed()
    );

    let mut sleep_sum: i64 = 0;
    let mut sleep_count: i64 = 0;
    let mut readiness_sum: i64 = 0;
    let mut readiness_count: i64 = 0;
    let mut activity_sum: i64 = 0;
    let mut activity_count: i64 = 0;

    for day in days {
        let s = sleep_map.get(day.as_str()).copied().flatten();
        let r = readiness_map.get(day.as_str()).copied().flatten();
        let a = activity_map.get(day.as_str()).copied().flatten();

        if let Some(v) = s {
            sleep_sum += v;
            sleep_count += 1;
        }
        if let Some(v) = r {
            readiness_sum += v;
            readiness_count += 1;
        }
        if let Some(v) = a {
            activity_sum += v;
            activity_count += 1;
        }

        // Format date as "Mon Feb 10"
        let label = chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d")
            .map(|d| d.format("%a %b %d").to_string())
            .unwrap_or_else(|_| day.clone());

        let sc = s.map_or("  --".dimmed().to_string(), |v| {
            format!("{:>4}", colored_score(v))
        });
        let rc = r.map_or("  --".dimmed().to_string(), |v| {
            format!("{:>4}", colored_score(v))
        });
        let ac = a.map_or("  --".dimmed().to_string(), |v| {
            format!("{:>4}", colored_score(v))
        });

        println!("  {label:<12}   {sc}      {rc}     {ac}");
    }

    // Averages
    let avg = |sum: i64, count: i64| -> String {
        if count == 0 {
            "  --".dimmed().to_string()
        } else {
            format!("{:>4}", colored_score(sum / count))
        }
    };

    println!(
        "  {:<12}   {}      {}     {}",
        "Average".dimmed(),
        avg(sleep_sum, sleep_count),
        avg(readiness_sum, readiness_count),
        avg(activity_sum, activity_count),
    );
}

fn format_number(n: i64) -> String {
    if n >= 1000 {
        format!("{},{:03}", n / 1000, n % 1000)
    } else {
        n.to_string()
    }
}
