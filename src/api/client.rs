use std::time::Duration;

use anyhow::Result;
use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use super::models::*;
use crate::error::BypassError;

const BASE_URL: &str = "https://api.app.shortcut.com/api/v3";

/// Retryable HTTP status codes.
const RETRYABLE: &[u16] = &[429, 500, 503, 504];
/// Maximum number of retry attempts (not counting the initial request).
const MAX_RETRIES: u32 = 5;
/// Base delay for exponential backoff.
const BASE_DELAY: Duration = Duration::from_secs(1);
/// Upper bound on any single backoff delay.
const MAX_DELAY: Duration = Duration::from_secs(30);

pub struct ShortcutClient {
    http: Client,
    token: String,
}

impl ShortcutClient {
    pub fn new(token: String) -> Result<Self> {
        let http = Client::builder()
            .user_agent(concat!("bypass-cli/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self { http, token })
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    /// Execute a cloneable request, retrying on transient errors with
    /// exponential backoff.  Honors the `Retry-After` header on 429s.
    async fn send_with_retry(&self, req: reqwest::Request) -> Result<reqwest::Response> {
        let mut attempt = 0u32;
        loop {
            let cloned = req
                .try_clone()
                .ok_or_else(|| anyhow::anyhow!("request body is not cloneable"))?;
            let resp = self.http.execute(cloned).await?;
            let status = resp.status().as_u16();

            if !RETRYABLE.contains(&status) || attempt >= MAX_RETRIES {
                return Ok(resp);
            }

            let delay = if status == 429 {
                // Honor `Retry-After: <seconds>` when present.
                resp.headers()
                    .get(reqwest::header::RETRY_AFTER)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(Duration::from_secs)
                    .unwrap_or_else(|| BASE_DELAY * (1 << attempt))
            } else {
                BASE_DELAY * (1 << attempt)
            }
            .min(MAX_DELAY);

            attempt += 1;
            tokio::time::sleep(delay).await;
        }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let req = self
            .http
            .get(format!("{BASE_URL}{path}"))
            .header("Shortcut-Token", &self.token)
            .build()?;
        let resp = self.send_with_retry(req).await?;
        self.handle_response(resp).await
    }

    async fn post<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T> {
        let req = self
            .http
            .post(format!("{BASE_URL}{path}"))
            .header("Shortcut-Token", &self.token)
            .json(body)
            .build()?;
        let resp = self.send_with_retry(req).await?;
        self.handle_response(resp).await
    }

    async fn handle_response<T: DeserializeOwned>(&self, resp: reqwest::Response) -> Result<T> {
        let status = resp.status();
        if status.is_success() {
            return Ok(resp.json::<T>().await?);
        }
        let body = resp.text().await.unwrap_or_default();
        let message = serde_json::from_str::<Value>(&body)
            .ok()
            .and_then(|v| {
                v.get("message")
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or(body);
        Err(BypassError::Api {
            status: status.as_u16(),
            message,
        }
        .into())
    }

    // ------------------------------------------------------------------
    // Read endpoints (used for name resolution)
    // ------------------------------------------------------------------

    pub async fn list_members(&self) -> Result<Vec<Member>> {
        self.get("/members").await
    }

    pub async fn list_groups(&self) -> Result<Vec<Group>> {
        self.get("/groups").await
    }

    pub async fn list_workflows(&self) -> Result<Vec<Workflow>> {
        self.get("/workflows").await
    }

    // ------------------------------------------------------------------
    // Create endpoints
    // ------------------------------------------------------------------

    pub async fn create_objective(&self, req: &CreateObjectiveRequest) -> Result<Objective> {
        self.post("/objectives", req).await
    }

    pub async fn create_epic(&self, req: &CreateEpicRequest) -> Result<Epic> {
        self.post("/epics", req).await
    }

    pub async fn create_story(&self, req: &CreateStoryRequest) -> Result<Story> {
        self.post("/stories", req).await
    }
}
