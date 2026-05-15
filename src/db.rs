use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, NaiveDateTime};
use duckdb::{Connection, params};
use std::fs;
use std::path::Path;

use crate::models::{DailyReadiness, DailySleep, Sleep};

const DB_PATH: &str = "/Users/terry/oura-data/data/oura.duckdb";

#[derive(Debug)]
pub struct NightlySleepRecord {
    pub day: NaiveDate,
    pub sleep_score: Option<i32>,
    pub readiness_score: Option<i32>,
    pub bedtime_start: Option<NaiveDateTime>,
    pub bedtime_end: Option<NaiveDateTime>,
    pub total_sleep_duration: Option<i32>,
    pub time_in_bed: Option<i32>,
    pub awake_time: Option<i32>,
    pub light_sleep_duration: Option<i32>,
    pub deep_sleep_duration: Option<i32>,
    pub rem_sleep_duration: Option<i32>,
    pub efficiency: Option<i32>,
    pub restless_periods: Option<i32>,
    pub average_hrv: Option<f64>,
    pub average_heart_rate: Option<f64>,
    pub sleep_phase_5_min: Option<String>,
    pub temperature_deviation: Option<f64>,
}

#[derive(Debug)]
pub struct SleepDayData {
    pub readiness_score: Option<i32>,
    pub total_sleep_duration: Option<i32>,
    pub efficiency: Option<i32>,
    pub deep_sleep_duration: Option<i32>,
    pub rem_sleep_duration: Option<i32>,
    pub average_hrv: Option<f64>,
    pub average_heart_rate: Option<f64>,
    pub bedtime_start: Option<NaiveDateTime>,
}

pub fn open_db() -> Result<Connection> {
    if let Some(parent) = Path::new(DB_PATH).parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let conn = Connection::open(DB_PATH).context("Failed to open DuckDB database")?;
    init_schema(&conn)?;
    Ok(conn)
}

pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS nightly_sleep (
            day DATE PRIMARY KEY,
            sleep_score INTEGER,
            readiness_score INTEGER,
            bedtime_start TIMESTAMP,
            bedtime_end TIMESTAMP,
            total_sleep_duration INTEGER,
            time_in_bed INTEGER,
            awake_time INTEGER,
            light_sleep_duration INTEGER,
            deep_sleep_duration INTEGER,
            rem_sleep_duration INTEGER,
            efficiency INTEGER,
            restless_periods INTEGER,
            average_hrv DOUBLE,
            average_heart_rate DOUBLE,
            sleep_phase_5_min TEXT,
            temperature_deviation DOUBLE
        );
        ",
    )
    .context("Failed to initialize DuckDB schema")
}

pub fn upsert_nightly_sleep(conn: &Connection, record: &NightlySleepRecord) -> Result<()> {
    conn.execute(
        "
        INSERT INTO nightly_sleep (
            day,
            sleep_score,
            readiness_score,
            bedtime_start,
            bedtime_end,
            total_sleep_duration,
            time_in_bed,
            awake_time,
            light_sleep_duration,
            deep_sleep_duration,
            rem_sleep_duration,
            efficiency,
            restless_periods,
            average_hrv,
            average_heart_rate,
            sleep_phase_5_min,
            temperature_deviation
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(day) DO UPDATE SET
            sleep_score = excluded.sleep_score,
            readiness_score = excluded.readiness_score,
            bedtime_start = excluded.bedtime_start,
            bedtime_end = excluded.bedtime_end,
            total_sleep_duration = excluded.total_sleep_duration,
            time_in_bed = excluded.time_in_bed,
            awake_time = excluded.awake_time,
            light_sleep_duration = excluded.light_sleep_duration,
            deep_sleep_duration = excluded.deep_sleep_duration,
            rem_sleep_duration = excluded.rem_sleep_duration,
            efficiency = excluded.efficiency,
            restless_periods = excluded.restless_periods,
            average_hrv = excluded.average_hrv,
            average_heart_rate = excluded.average_heart_rate,
            sleep_phase_5_min = excluded.sleep_phase_5_min,
            temperature_deviation = excluded.temperature_deviation
        ",
        params![
            record.day,
            record.sleep_score,
            record.readiness_score,
            record.bedtime_start,
            record.bedtime_end,
            record.total_sleep_duration,
            record.time_in_bed,
            record.awake_time,
            record.light_sleep_duration,
            record.deep_sleep_duration,
            record.rem_sleep_duration,
            record.efficiency,
            record.restless_periods,
            record.average_hrv,
            record.average_heart_rate,
            record.sleep_phase_5_min,
            record.temperature_deviation,
        ],
    )
    .context("Failed to upsert nightly sleep row")?;

    Ok(())
}

pub fn build_nightly_sleep_record(
    date: NaiveDate,
    daily_sleep: Option<&DailySleep>,
    daily_readiness: Option<&DailyReadiness>,
    sleep: Option<&Sleep>,
) -> Result<NightlySleepRecord> {
    Ok(NightlySleepRecord {
        day: date,
        sleep_score: daily_sleep
            .and_then(|record| record.score)
            .map(|value| value as i32),
        readiness_score: daily_readiness
            .and_then(|record| record.score)
            .map(|value| value as i32),
        bedtime_start: sleep
            .and_then(|record| record.bedtime_start.as_deref())
            .map(parse_rfc3339_local)
            .transpose()?,
        bedtime_end: sleep
            .and_then(|record| record.bedtime_end.as_deref())
            .map(parse_rfc3339_local)
            .transpose()?,
        total_sleep_duration: sleep
            .and_then(|record| record.total_sleep_duration)
            .map(|value| value as i32),
        time_in_bed: sleep
            .and_then(|record| record.time_in_bed)
            .map(|value| value as i32),
        awake_time: sleep
            .and_then(|record| record.awake_time)
            .map(|value| value as i32),
        light_sleep_duration: sleep
            .and_then(|record| record.light_sleep_duration)
            .map(|value| value as i32),
        deep_sleep_duration: sleep
            .and_then(|record| record.deep_sleep_duration)
            .map(|value| value as i32),
        rem_sleep_duration: sleep
            .and_then(|record| record.rem_sleep_duration)
            .map(|value| value as i32),
        efficiency: sleep
            .and_then(|record| record.efficiency)
            .map(|value| value as i32),
        restless_periods: sleep
            .and_then(|record| record.restless_periods)
            .map(|value| value as i32),
        average_hrv: sleep
            .and_then(|record| record.average_hrv)
            .map(|value| value as f64),
        average_heart_rate: sleep.and_then(|record| record.average_heart_rate),
        sleep_phase_5_min: sleep.and_then(|record| record.sleep_phase_5_min.clone()),
        temperature_deviation: daily_readiness.and_then(|record| record.temperature_deviation),
    })
}

pub fn fetch_sleep_window(
    conn: &Connection,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<SleepDayData>> {
    let mut stmt = conn.prepare(
        "
        SELECT
            day,
            readiness_score,
            total_sleep_duration,
            efficiency,
            deep_sleep_duration,
            rem_sleep_duration,
            average_hrv,
            average_heart_rate,
            bedtime_start
        FROM nightly_sleep
        WHERE day BETWEEN ? AND ?
        ORDER BY day ASC
        ",
    )?;

    let rows = stmt.query_map(params![start_date, end_date], |row| {
        Ok(SleepDayData {
            readiness_score: row.get(1)?,
            total_sleep_duration: row.get(2)?,
            efficiency: row.get(3)?,
            deep_sleep_duration: row.get(4)?,
            rem_sleep_duration: row.get(5)?,
            average_hrv: row.get(6)?,
            average_heart_rate: row.get(7)?,
            bedtime_start: row.get(8)?,
        })
    })?;

    let mut data = Vec::new();
    for row in rows {
        data.push(row?);
    }

    Ok(data)
}

fn parse_rfc3339_local(value: &str) -> Result<NaiveDateTime> {
    Ok(DateTime::parse_from_rfc3339(value)
        .with_context(|| format!("Invalid RFC3339 timestamp: {value}"))?
        .naive_local())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DailyReadiness, DailySleep, Sleep};
    use chrono::{NaiveDate, NaiveDateTime};

    #[test]
    fn builds_nightly_sleep_record_from_api_models() {
        let daily_sleep = DailySleep {
            day: "2026-03-10".to_string(),
            score: Some(83),
            contributors: None,
        };
        let daily_readiness = DailyReadiness {
            day: "2026-03-10".to_string(),
            score: Some(81),
            temperature_deviation: Some(0.2),
            temperature_trend_deviation: None,
            contributors: None,
        };
        let sleep = Sleep {
            day: "2026-03-10".to_string(),
            sleep_type: Some("long_sleep".to_string()),
            period: None,
            bedtime_start: Some("2026-03-09T23:45:00+08:00".to_string()),
            bedtime_end: Some("2026-03-10T07:15:00+08:00".to_string()),
            sleep_phase_5_min: Some("112233".to_string()),
            sleep_phase_30_sec: None,
            app_sleep_phase_5_min: None,
            movement_30_sec: None,
            heart_rate: None,
            hrv: None,
            total_sleep_duration: Some(25_200),
            time_in_bed: Some(27_000),
            efficiency: Some(93),
            latency: None,
            deep_sleep_duration: Some(5_400),
            light_sleep_duration: Some(13_200),
            rem_sleep_duration: Some(6_600),
            awake_time: Some(1_800),
            restless_periods: Some(4),
            average_breath: None,
            average_heart_rate: Some(51.2),
            average_hrv: Some(42),
            lowest_heart_rate: None,
            readiness_score_delta: None,
            sleep_score_delta: None,
            low_battery_alert: None,
        };

        let record = build_nightly_sleep_record(
            NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
            Some(&daily_sleep),
            Some(&daily_readiness),
            Some(&sleep),
        )
        .unwrap();

        assert_eq!(record.sleep_score, Some(83));
        assert_eq!(record.readiness_score, Some(81));
        assert_eq!(
            record.bedtime_start,
            Some(
                NaiveDateTime::parse_from_str("2026-03-09 23:45:00", "%Y-%m-%d %H:%M:%S").unwrap()
            )
        );
        assert_eq!(record.total_sleep_duration, Some(25_200));
        assert_eq!(record.average_hrv, Some(42.0));
        assert_eq!(record.average_heart_rate, Some(51.2));
        assert_eq!(record.temperature_deviation, Some(0.2));
        assert_eq!(record.sleep_phase_5_min.as_deref(), Some("112233"));
    }
}
