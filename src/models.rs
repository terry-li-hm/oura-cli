use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub data: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct DailySleep {
    pub day: String,
    pub score: Option<i64>,
    pub contributors: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct DailyReadiness {
    pub day: String,
    pub score: Option<i64>,
    pub temperature_deviation: Option<f64>,
    pub temperature_trend_deviation: Option<f64>,
    pub contributors: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct DailyActivity {
    pub day: String,
    pub score: Option<i64>,
    pub active_calories: Option<i64>,
    pub average_met_minutes: Option<f64>,
    pub steps: Option<i64>,
    pub equivalent_walking_distance: Option<i64>,
    pub high_activity_time: Option<i64>,
    pub high_activity_met_minutes: Option<i64>,
    pub medium_activity_time: Option<i64>,
    pub medium_activity_met_minutes: Option<i64>,
    pub low_activity_time: Option<i64>,
    pub low_activity_met_minutes: Option<i64>,
    pub sedentary_time: Option<i64>,
    pub sedentary_met_minutes: Option<i64>,
    pub total_calories: Option<i64>,
    pub target_calories: Option<i64>,
    pub meters_to_target: Option<i64>,
    pub non_wear_time: Option<i64>,
    pub resting_time: Option<i64>,
    pub inactivity_alerts: Option<i64>,
    pub class_5_min: Option<String>,
    pub contributors: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct Sleep {
    pub day: String,
    #[serde(rename = "type")]
    pub sleep_type: Option<String>,
    pub period: Option<i64>,
    pub bedtime_start: Option<String>,
    pub bedtime_end: Option<String>,
    pub sleep_phase_5_min: Option<String>,
    pub sleep_phase_30_sec: Option<String>,
    pub app_sleep_phase_5_min: Option<String>,
    pub movement_30_sec: Option<String>,
    pub heart_rate: Option<serde_json::Value>,
    pub hrv: Option<serde_json::Value>,
    pub total_sleep_duration: Option<i64>,
    pub time_in_bed: Option<i64>,
    pub efficiency: Option<i64>,
    pub latency: Option<i64>,
    pub deep_sleep_duration: Option<i64>,
    pub light_sleep_duration: Option<i64>,
    pub rem_sleep_duration: Option<i64>,
    pub awake_time: Option<i64>,
    pub restless_periods: Option<i64>,
    pub average_breath: Option<f64>,
    pub average_heart_rate: Option<f64>,
    pub average_hrv: Option<i64>,
    pub lowest_heart_rate: Option<i64>,
    pub readiness_score_delta: Option<i64>,
    pub sleep_score_delta: Option<i64>,
    pub low_battery_alert: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct DailyStress {
    pub day: String,
    pub day_summary: Option<String>,
    pub stress_high: Option<i64>,
    pub recovery_high: Option<i64>,
}
