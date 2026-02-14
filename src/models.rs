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
    pub steps: Option<i64>,
    pub equivalent_walking_distance: Option<i64>,
    pub high_activity_time: Option<i64>,
    pub medium_activity_time: Option<i64>,
    pub low_activity_time: Option<i64>,
    pub sedentary_time: Option<i64>,
    pub total_calories: Option<i64>,
    pub target_calories: Option<i64>,
    pub contributors: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct Sleep {
    pub day: String,
    #[serde(rename = "type")]
    pub sleep_type: Option<String>,
    pub bedtime_start: Option<String>,
    pub bedtime_end: Option<String>,
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
}

#[derive(Debug, Deserialize)]
pub struct DailyStress {
    pub day: String,
    pub day_summary: Option<String>,
    pub stress_high: Option<i64>,
    pub recovery_high: Option<i64>,
}
