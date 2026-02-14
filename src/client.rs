use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;

use crate::models::{ApiResponse, DailyActivity, DailyReadiness, DailySleep, DailyStress, Sleep};

pub struct OuraClient {
    client: Client,
    token: String,
}

/// Oura API v2 has inconsistent end_date behavior: some endpoints treat it as
/// inclusive, others as exclusive. Bumping end_date by +1 day ensures we always
/// get the target date's data regardless.
fn next_day(date: &str) -> Result<String> {
    let d = NaiveDate::parse_from_str(date, "%Y-%m-%d").context("Invalid date format")?;
    Ok(d.succ_opt().expect("date overflow").format("%Y-%m-%d").to_string())
}

impl OuraClient {
    pub fn new() -> Result<Self> {
        let token = std::env::var("OURA_TOKEN").context(
            "OURA_TOKEN not set. Get your token at https://cloud.ouraring.com/personal-access-tokens",
        )?;
        Ok(Self {
            client: Client::new(),
            token,
        })
    }

    fn fetch<T: DeserializeOwned>(&self, endpoint: &str, date: &str) -> Result<Vec<T>> {
        let end = next_day(date)?;
        let url = format!("https://api.ouraring.com/v2/usercollection/{endpoint}");
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .query(&[("start_date", date), ("end_date", &end)])
            .send()
            .context("Failed to reach Oura API")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            bail!("Oura API returned {status}: {body}");
        }

        let body: ApiResponse<T> = resp.json().context("Failed to parse API response")?;
        Ok(body.data)
    }

    fn fetch_range<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        start: &str,
        end: &str,
    ) -> Result<Vec<T>> {
        let end_plus = next_day(end)?;
        let url = format!("https://api.ouraring.com/v2/usercollection/{endpoint}");
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .query(&[("start_date", start), ("end_date", &end_plus)])
            .send()
            .context("Failed to reach Oura API")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            bail!("Oura API returned {status}: {body}");
        }

        let body: ApiResponse<T> = resp.json().context("Failed to parse API response")?;
        Ok(body.data)
    }

    pub fn daily_sleep(&self, date: &str) -> Result<Vec<DailySleep>> {
        self.fetch("daily_sleep", date)
    }

    pub fn daily_readiness(&self, date: &str) -> Result<Vec<DailyReadiness>> {
        self.fetch("daily_readiness", date)
    }

    pub fn daily_activity(&self, date: &str) -> Result<Vec<DailyActivity>> {
        self.fetch("daily_activity", date)
    }

    pub fn sleep(&self, date: &str) -> Result<Vec<Sleep>> {
        self.fetch("sleep", date)
    }

    pub fn daily_stress(&self, date: &str) -> Result<Vec<DailyStress>> {
        self.fetch("daily_stress", date)
    }

    pub fn daily_sleep_range(&self, start: &str, end: &str) -> Result<Vec<DailySleep>> {
        self.fetch_range("daily_sleep", start, end)
    }

    pub fn daily_readiness_range(&self, start: &str, end: &str) -> Result<Vec<DailyReadiness>> {
        self.fetch_range("daily_readiness", start, end)
    }

    pub fn daily_activity_range(&self, start: &str, end: &str) -> Result<Vec<DailyActivity>> {
        self.fetch_range("daily_activity", start, end)
    }

    pub fn raw(&self, endpoint: &str, date: &str) -> Result<serde_json::Value> {
        let end = next_day(date)?;
        let url = format!("https://api.ouraring.com/v2/usercollection/{endpoint}");
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .query(&[("start_date", date), ("end_date", &end)])
            .send()
            .context("Failed to reach Oura API")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            bail!("Oura API returned {status}: {body}");
        }

        resp.json().context("Failed to parse response")
    }
}
