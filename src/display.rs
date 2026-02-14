use owo_colors::OwoColorize;

use crate::models::{DailyActivity, DailyReadiness, DailyStress, Sleep};

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
        format!("{}h {:02}m", hours, mins)
    } else {
        format!("{}m", mins)
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
                println!("  {:<24}{}", format_contributor_key(key), colored_score(score));
            }
        }
    }
}

// --- Command display functions ---

pub fn display_scores(sleep: Option<i64>, readiness: Option<i64>, activity: Option<i64>) {
    let s = sleep.map_or("--".dimmed().to_string(), colored_score);
    let r = readiness.map_or("--".dimmed().to_string(), colored_score);
    let a = activity.map_or("--".dimmed().to_string(), colored_score);
    println!("  Sleep {}  Readiness {}  Activity {}", s, r, a);
}

pub fn display_sleep(score: Option<i64>, records: &[Sleep]) {
    let sleep = records
        .iter()
        .find(|s| s.sleep_type.as_deref() == Some("long_sleep"))
        .or(records.first());

    let Some(s) = sleep else {
        println!("  No sleep data");
        return;
    };

    if let Some(v) = score {
        println!("  Sleep Score: {}", colored_score(v));
    }

    let total = s.total_sleep_duration.unwrap_or(0);
    println!("  Total Sleep: {}", format_duration(total));

    if let Some(eff) = s.efficiency {
        println!("  Efficiency:  {}%", eff);
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
        println!("  Avg HRV:     {} ms", hrv);
    }
    if let Some(hr) = s.average_heart_rate {
        println!("  Avg HR:      {} bpm", hr.round() as i64);
    }
    if let Some(low) = s.lowest_heart_rate {
        println!("  Lowest HR:   {} bpm", low);
    }

    if let (Some(start), Some(end)) = (&s.bedtime_start, &s.bedtime_end) {
        println!(
            "  Bedtime:     {} → {}",
            format_time(start),
            format_time(end)
        );
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
        println!("  Temp Deviation:  {:+.1}°C", temp);
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

pub fn display_hrv(records: &[Sleep]) {
    let sleep = records
        .iter()
        .find(|s| s.sleep_type.as_deref() == Some("long_sleep"))
        .or(records.first());

    let Some(s) = sleep else {
        println!("  No sleep data for HRV");
        return;
    };

    println!("  {}", "HRV (from sleep)".dimmed());

    if let Some(hrv) = s.average_hrv {
        println!("  Avg HRV:     {} ms", hrv);
    } else {
        println!("  Avg HRV:     --");
    }

    if let Some(hr) = s.average_heart_rate {
        println!("  Avg HR:      {} bpm", hr.round() as i64);
    }
    if let Some(low) = s.lowest_heart_rate {
        println!("  Lowest HR:   {} bpm", low);
    }
    if let Some(breath) = s.average_breath {
        println!("  Avg Breath:  {:.1} rpm", breath);
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
        println!("  Stress Summary: {}", colored);
    }

    if let Some(stress) = s.stress_high {
        println!("  Stress High:    {}", format_duration(stress));
    }
    if let Some(recovery) = s.recovery_high {
        println!("  Recovery High:  {}", format_duration(recovery));
    }
}

fn format_number(n: i64) -> String {
    if n >= 1000 {
        let whole = n / 1000;
        let frac = (n % 1000) / 100;
        if frac > 0 {
            format!("{},{:03}", whole, n % 1000)
        } else {
            format!("{},{:03}", whole, n % 1000)
        }
    } else {
        n.to_string()
    }
}
