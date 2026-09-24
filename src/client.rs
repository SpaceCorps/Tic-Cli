//! HTTP client for the TIC API, translating HTTP status codes to [`ErrorCode`].
//!
//! Uses blocking HTTP via `ureq` with rustls and gzip.

use std::time::Duration;

use serde_json::Value;
use ureq::Agent;
use ureq::http::Response;

use crate::error::{Error, ErrorCode, Result};
use crate::obj;

const DEFAULT_BASE: &str = "https://api.tic.io/";
const MAX_BODY: u64 = 64 * 1024 * 1024;

#[derive(Clone)]
pub struct Client {
    agent: Agent,
    base: String,
    api_key: String,
}

#[allow(dead_code)]
enum Method {
    Get,
    Post,
}

impl Client {
    pub fn new(api_key: &str) -> Client {
        let agent: Agent = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(60)))
            .timeout_connect(Some(Duration::from_secs(15)))
            .http_status_as_error(false)
            .user_agent(concat!("tic-cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .into();

        let mut base = std::env::var("TIC_API_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BASE.to_string());
        if !base.ends_with('/') {
            base.push('/');
        }

        Client { agent, base, api_key: api_key.trim().to_string() }
    }

    pub fn get(&self, path: &str) -> Result<Value> {
        self.send(Method::Get, path, None)
    }

    #[allow(dead_code)]
    pub fn post(&self, path: &str, body: Option<&Value>) -> Result<Value> {
        self.send(Method::Post, path, body)
    }

    pub fn search(
        &self,
        collection: &str,
        query: &str,
        query_by: &str,
        filter_by: Option<&str>,
        sort_by: Option<&str>,
        per_page: Option<usize>,
    ) -> Result<Value> {
        let coll = collection.trim_start_matches('/');
        let mut path = format!("search/{coll}?q={}&query_by={}", seg(query), seg(query_by));
        if let Some(f) = filter_by.filter(|s| !s.trim().is_empty()) {
            path.push_str(&format!("&filter_by={}", seg(f)));
        }
        if let Some(s) = sort_by.filter(|s| !s.trim().is_empty()) {
            path.push_str(&format!("&sort_by={}", seg(s)));
        }
        if let Some(p) = per_page {
            path.push_str(&format!("&per_page={p}"));
        }
        self.get(&path)
    }

    fn send(&self, method: Method, path: &str, body: Option<&Value>) -> Result<Value> {
        let p = path.trim_start_matches('/');
        let url = format!("{}{}", self.base, p);

        macro_rules! headers {
            ($req:expr) => {
                $req.header("Authorization", &format!("Bearer {}", self.api_key))
                    .header("x-api-key", &self.api_key)
                    .header("Accept", "application/json")
            };
        }

        let result = match (method, body) {
            (Method::Get, _) => headers!(self.agent.get(&url)).call(),
            (Method::Post, Some(b)) => {
                let json = serde_json::to_vec(b).expect("a Value always serializes");
                headers!(self.agent.post(&url)).header("Content-Type", "application/json").send(&json[..])
            }
            (Method::Post, None) => headers!(self.agent.post(&url)).send_empty(),
        };

        let response = result.map_err(transport_error)?;
        read(response)
    }
}

pub fn extract_single_hit(search_result: &Value) -> Result<Value> {
    if let Some(hits) = search_result.get("hits").and_then(Value::as_array)
        && let Some(first) = hits.first()
    {
        if let Some(doc) = first.get("document") {
            return Ok(doc.clone());
        }
        return Ok(first.clone());
    }
    Err(Error::new(ErrorCode::NotFound, "No results found"))
}

pub fn extract_hits(search_result: &Value) -> Value {
    let mut items = Vec::new();
    if let Some(hits) = search_result.get("hits").and_then(Value::as_array) {
        for hit in hits {
            if let Some(doc) = hit.get("document") {
                items.push(doc.clone());
            } else {
                items.push(hit.clone());
            }
        }
    }
    let found = search_result.get("found").and_then(Value::as_i64).unwrap_or(items.len() as i64);
    obj! {
        "found" => found,
        "items" => items,
    }
}

fn read(mut response: Response<ureq::Body>) -> Result<Value> {
    let status = response.status().as_u16();
    let bytes = response.body_mut().with_config().limit(MAX_BODY).read_to_vec().map_err(transport_error)?;

    if !(200..300).contains(&status) {
        let body = String::from_utf8_lossy(&bytes).trim().to_string();
        return Err(status_error(status, &body));
    }

    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Ok(obj! { "status" => "ok" });
    }

    serde_json::from_slice(&bytes).or_else(|_| Ok(Value::String(String::from_utf8_lossy(&bytes).into_owned())))
}

fn transport_error(e: ureq::Error) -> Error {
    match e {
        ureq::Error::Timeout(_) => {
            Error::new(ErrorCode::Network, "The request timed out.").fix("Retry once, then stop.")
        }
        other => Error::new(ErrorCode::Network, "Could not reach the TIC API.")
            .detail(other.to_string())
            .fix("Retry once, then stop."),
    }
}

pub fn status_error(status: u16, body: &str) -> Error {
    let mut detail = format!("HTTP {status}");
    let extracted_msg = extract_error_message(body);
    if let Some(ref msg) = extracted_msg {
        detail.push_str(": ");
        detail.push_str(msg);
    } else if !body.is_empty() {
        detail.push_str(": ");
        detail.push_str(body);
    }

    let e = match status {
        401 | 403 => Error::new(
            ErrorCode::AuthRequired,
            extracted_msg.unwrap_or_else(|| "Authentication required or invalid TIC API key.".into()),
        )
        .fix("Set TIC_API_KEY, pass --api-key, or run: tic login <name>"),
        404 => Error::new(
            ErrorCode::NotFound,
            extracted_msg.unwrap_or_else(|| "The requested TIC resource was not found.".into()),
        ),
        429 => {
            Error::new(ErrorCode::RateLimited, extracted_msg.unwrap_or_else(|| "Rate limited by TIC API (429).".into()))
                .fix("Back off before retrying.")
        }
        400 | 422 => Error::new(
            ErrorCode::InvalidInput,
            extracted_msg.unwrap_or_else(|| "Invalid request or search parameters.".into()),
        ),
        s if s >= 500 => {
            Error::new(ErrorCode::Network, extracted_msg.unwrap_or_else(|| "TIC API returned a server error.".into()))
                .fix("Retry; if it persists, check https://tic.io status.")
        }
        _ => {
            Error::new(ErrorCode::Error, extracted_msg.unwrap_or_else(|| format!("Request failed with HTTP {status}")))
        }
    };
    e.detail(detail)
}

fn extract_error_message(body: &str) -> Option<String> {
    let v: Value = serde_json::from_str(body).ok()?;
    if let Some(msg) = v.get("message").and_then(Value::as_str) {
        return Some(msg.to_string());
    }
    if let Some(err) = v.get("error").and_then(Value::as_str) {
        return Some(err.to_string());
    }
    if let Some(detail) = v.get("detail").and_then(Value::as_str) {
        return Some(detail.to_string());
    }
    if let Some(title) = v.get("title").and_then(Value::as_str) {
        let mut text = title.to_string();
        if let Some(errors) = v.get("errors").and_then(Value::as_object) {
            let details: Vec<String> = errors
                .iter()
                .filter_map(|(k, val)| {
                    val.as_array().map(|arr| {
                        let msgs: Vec<&str> = arr.iter().filter_map(Value::as_str).collect();
                        format!("{k}: {}", msgs.join("; "))
                    })
                })
                .collect();
            if !details.is_empty() {
                text.push_str(" - ");
                text.push_str(&details.join("; "));
            }
        }
        return Some(text);
    }
    None
}

/// Percent-encodes a path or query component.
pub fn seg(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'*' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seg_escapes_special() {
        assert_eq!(seg("556792-6687"), "556792-6687");
        assert_eq!(seg("A B/C"), "A%20B%2FC");
    }

    #[test]
    fn status_maps_to_codes() {
        assert_eq!(status_error(401, "").code, ErrorCode::AuthRequired);
        assert_eq!(status_error(404, "").code, ErrorCode::NotFound);
        assert_eq!(status_error(429, "").code, ErrorCode::RateLimited);
        assert_eq!(status_error(422, "").code, ErrorCode::InvalidInput);
        assert_eq!(status_error(500, "").code, ErrorCode::Network);
    }
}
