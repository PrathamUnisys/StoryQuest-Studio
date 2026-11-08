use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Duration, Local, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::credentials::{delete_api_key, retrieve_api_key, store_api_key};

pub const YOUTUBE_SCOPES: &str = "https://www.googleapis.com/auth/youtube.readonly https://www.googleapis.com/auth/yt-analytics.readonly https://www.googleapis.com/auth/yt-analytics-monetary.readonly";

const CLIENT_ID_KEY: &str = "youtube_oauth_client_id";
const CLIENT_SECRET_KEY: &str = "youtube_oauth_client_secret";
const TOKEN_KEY: &str = "youtube_oauth_tokens";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub scope: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    token_type: Option<String>,
    scope: Option<String>,
    expires_in: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChannelStats {
    pub channel_id: String,
    pub title: String,
    pub subscriber_count: Option<i64>,
    pub hidden_subscriber_count: bool,
    pub view_count: i64,
    pub video_count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnalyticsStats {
    pub views: i64,
    pub estimated_minutes_watched: i64,
    pub subscribers_gained: i64,
    pub subscribers_lost: i64,
    pub period_start: String,
    pub period_end: String,
}

pub fn pkce_challenge(code_verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hasher.finalize())
}

pub fn generate_code_verifier() -> String {
    let bytes = uuid::Uuid::new_v4().into_bytes();
    let mut buf = [0u8; 32];
    buf[..16].copy_from_slice(&bytes);
    buf[16..].copy_from_slice(&uuid::Uuid::new_v4().into_bytes());
    URL_SAFE_NO_PAD.encode(buf)
}

pub fn generate_state() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

pub fn build_auth_url(client_id: &str, redirect_uri: &str, code_challenge: &str, state: &str) -> String {
    build_auth_url_with_hint(client_id, redirect_uri, code_challenge, state, None)
}

pub fn build_auth_url_with_hint(
    client_id: &str,
    redirect_uri: &str,
    code_challenge: &str,
    state: &str,
    login_hint: Option<&str>,
) -> String {
    let mut url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&code_challenge={}&code_challenge_method=S256&access_type=offline&prompt={}&state={}",
        urlencoding::encode(client_id),
        urlencoding::encode(redirect_uri),
        urlencoding::encode(YOUTUBE_SCOPES),
        urlencoding::encode(code_challenge),
        urlencoding::encode("select_account consent"),
        urlencoding::encode(state),
    );
    if let Some(hint) = login_hint.map(str::trim).filter(|h| !h.is_empty()) {
        if hint.contains('@') {
            url.push_str(&format!("&login_hint={}", urlencoding::encode(hint)));
        }
    }
    url
}

pub fn parse_channels_list(body: &serde_json::Value) -> Result<ChannelStats, String> {
    let item = body
        .get("items")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .ok_or_else(|| "YouTube Data API returned no channel items".to_string())?;
    let channel_id = item
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let title = item
        .get("snippet")
        .and_then(|s| s.get("title"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let stats = item
        .get("statistics")
        .ok_or_else(|| "YouTube channel missing statistics part".to_string())?;
    let parse_u64 = |key: &str| -> i64 {
        stats
            .get(key)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0)
    };
    let subscriber_count = stats
        .get("subscriberCount")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i64>().ok());
    Ok(ChannelStats {
        channel_id,
        title,
        subscriber_count,
        hidden_subscriber_count: stats
            .get("hiddenSubscriberCount")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        view_count: parse_u64("viewCount"),
        video_count: parse_u64("videoCount"),
    })
}

pub fn parse_analytics_report(body: &serde_json::Value, period_start: &str, period_end: &str) -> Result<AnalyticsStats, String> {
    let header = body
        .get("columnHeaders")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let names: Vec<String> = header
        .iter()
        .filter_map(|h| h.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .collect();
    let rows = body
        .get("rows")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut get = |name: &str| -> i64 {
        let idx = match names.iter().position(|n| n == name) {
            Some(i) => i,
            None => return 0,
        };
        rows.first()
            .and_then(|r| r.as_array())
            .and_then(|a| a.get(idx))
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
    };
    Ok(AnalyticsStats {
        views: get("views"),
        estimated_minutes_watched: get("estimatedMinutesWatched"),
        subscribers_gained: get("subscribersGained"),
        subscribers_lost: get("subscribersLost"),
        period_start: period_start.to_string(),
        period_end: period_end.to_string(),
    })
}

pub fn tokens_expired(expires_at: &str, now: DateTime<Utc>) -> bool {
    match expires_at.parse::<DateTime<Utc>>() {
        Ok(exp) => now >= exp - Duration::seconds(60),
        Err(_) => true,
    }
}

pub fn set_client_credentials(client_id: &str, client_secret: &str) -> Result<(), String> {
    if client_id.trim().is_empty() || client_secret.trim().is_empty() {
        return Err("Client ID and Client Secret are both required".to_string());
    }
    if !client_id.ends_with(".apps.googleusercontent.com") {
        return Err("Client ID must end with .apps.googleusercontent.com (Desktop app type in Google Cloud Console)".to_string());
    }
    store_api_key(CLIENT_ID_KEY, client_id.trim())?;
    store_api_key(CLIENT_SECRET_KEY, client_secret.trim())
}

pub fn env_login_hint() -> Option<String> {
    env_var(&["GOOGLE_YOUTUBE_LOGIN_HINT", "STORYQUEST_YOUTUBE_LOGIN_HINT"])
}

fn env_var(names: &[&str]) -> Option<String> {
    names
        .iter()
        .filter_map(|n| std::env::var(n).ok())
        .map(|v| v.trim().to_string())
        .find(|v| !v.is_empty())
}

pub fn get_client_credentials() -> Result<(String, String), String> {
    let id = retrieve_api_key(CLIENT_ID_KEY)?
        .filter(|v| !v.trim().is_empty())
        .or_else(|| env_var(&["GOOGLE_YOUTUBE_CLIENT_ID", "STORYQUEST_YOUTUBE_CLIENT_ID"]))
        .ok_or_else(|| "No Google OAuth Client ID configured. Set GOOGLE_YOUTUBE_CLIENT_ID in .env or in YouTube settings first.".to_string())?;
    let secret = retrieve_api_key(CLIENT_SECRET_KEY)?
        .filter(|v| !v.trim().is_empty())
        .or_else(|| env_var(&["GOOGLE_YOUTUBE_CLIENT_SECRET", "STORYQUEST_YOUTUBE_CLIENT_SECRET"]))
        .ok_or_else(|| "No Google OAuth Client Secret configured. Set GOOGLE_YOUTUBE_CLIENT_SECRET in .env or in YouTube settings first.".to_string())?;
    Ok((id, secret))
}

pub fn has_client_credentials() -> bool {
    get_client_credentials().is_ok()
}

pub fn store_tokens(tokens: &StoredTokens) -> Result<(), String> {
    store_api_key(TOKEN_KEY, &serde_json::to_string(tokens).map_err(|e| e.to_string())?)
}

pub fn load_tokens() -> Result<Option<StoredTokens>, String> {
    match retrieve_api_key(TOKEN_KEY)? {
        None => Ok(None),
        Some(raw) => serde_json::from_str(&raw).map(Some).map_err(|e| e.to_string()),
    }
}

pub fn clear_tokens() {
    let _ = delete_api_key(TOKEN_KEY);
}

async fn post_token(params: &[(&str, &str)]) -> Result<TokenResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://oauth2.googleapis.com/token")
        .form(params)
        .send()
        .await
        .map_err(|e| format!("Token request failed: {e}"))?;
    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Google token endpoint error: {text}"));
    }
    resp.json::<TokenResponse>()
        .await
        .map_err(|e| format!("Invalid token response: {e}"))
}

fn to_stored(raw: &TokenResponse) -> Result<StoredTokens, String> {
    let expires_in = raw.expires_in.unwrap_or(3600);
    let expires_at = Utc::now() + Duration::seconds(expires_in);
    Ok(StoredTokens {
        access_token: raw.access_token.clone(),
        refresh_token: raw.refresh_token.clone(),
        token_type: raw.token_type.clone().unwrap_or_else(|| "Bearer".to_string()),
        scope: raw.scope.clone().unwrap_or_default(),
        expires_at: expires_at.to_rfc3339(),
    })
}

pub async fn exchange_code(code: &str, code_verifier: &str, redirect_uri: &str) -> Result<StoredTokens, String> {
    let (client_id, client_secret) = get_client_credentials()?;
    let raw = post_token(&[
        ("code", code),
        ("client_id", &client_id),
        ("client_secret", &client_secret),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
        ("code_verifier", code_verifier),
    ])
    .await?;
    let stored = to_stored(&raw)?;
    store_tokens(&stored)?;
    Ok(stored)
}

pub async fn refresh_access_token(tokens: &StoredTokens) -> Result<StoredTokens, String> {
    let refresh = tokens
        .refresh_token
        .as_deref()
        .ok_or_else(|| "No refresh token stored; reconnect YouTube".to_string())?;
    let (client_id, client_secret) = get_client_credentials()?;
    let raw = post_token(&[
        ("client_id", &client_id),
        ("client_secret", &client_secret),
        ("refresh_token", refresh),
        ("grant_type", "refresh_token"),
    ])
    .await?;
    let mut stored = to_stored(&raw)?;
    if stored.refresh_token.is_none() {
        stored.refresh_token = tokens.refresh_token.clone();
    }
    store_tokens(&stored)?;
    Ok(stored)
}

pub async fn ensure_access_token() -> Result<String, String> {
    let tokens = load_tokens()?.ok_or_else(|| "YouTube is not connected".to_string())?;
    if tokens_expired(&tokens.expires_at, Utc::now()) {
        let fresh = refresh_access_token(&tokens).await?;
        Ok(fresh.access_token)
    } else {
        Ok(tokens.access_token)
    }
}

pub async fn fetch_channel_stats(access_token: &str) -> Result<ChannelStats, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://www.googleapis.com/youtube/v3/channels")
        .query(&[("part", "snippet,statistics"), ("mine", "true")])
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("YouTube Data API request failed: {e}"))?;
    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("YouTube Data API error: {text}"));
    }
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Invalid channel response: {e}"))?;
    parse_channels_list(&body)
}

pub async fn fetch_analytics(access_token: &str) -> Result<AnalyticsStats, String> {
    let end = Local::now().date_naive();
    let start = end - Duration::days(27);
    let start_str = start.format("%Y-%m-%d").to_string();
    let end_str = end.format("%Y-%m-%d").to_string();
    let client = reqwest::Client::new();
    let resp = client
        .get("https://youtubeanalytics.googleapis.com/v2/reports")
        .query(&[
            ("ids", "channel==MINE"),
            ("startDate", &start_str),
            ("endDate", &end_str),
            ("metrics", "views,estimatedMinutesWatched,subscribersGained,subscribersLost"),
        ])
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("YouTube Analytics API request failed: {e}"))?;
    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("YouTube Analytics API error: {text}"));
    }
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Invalid analytics response: {e}"))?;
    parse_analytics_report(&body, &start_str, &end_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_s256_known_vector() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        assert_eq!(
            pkce_challenge(verifier),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn parses_channels_list() {
        let body = serde_json::json!({
            "items": [{
                "id": "UC123",
                "snippet": {"title": "My Channel"},
                "statistics": {"viewCount": "12345", "subscriberCount": "678", "hiddenSubscriberCount": false, "videoCount": "42"}
            }]
        });
        let s = parse_channels_list(&body).unwrap();
        assert_eq!(s.channel_id, "UC123");
        assert_eq!(s.title, "My Channel");
        assert_eq!(s.view_count, 12345);
        assert_eq!(s.subscriber_count, Some(678));
        assert_eq!(s.video_count, 42);
    }

    #[test]
    fn parses_analytics_report() {
        let body = serde_json::json!({
            "columnHeaders": [{"name": "views"}, {"name": "estimatedMinutesWatched"}, {"name": "subscribersGained"}, {"name": "subscribersLost"}],
            "rows": [[1000, 5000, 25, 3]]
        });
        let a = parse_analytics_report(&body, "2026-01-01", "2026-01-28").unwrap();
        assert_eq!(a.views, 1000);
        assert_eq!(a.estimated_minutes_watched, 5000);
        assert_eq!(a.subscribers_gained, 25);
        assert_eq!(a.subscribers_lost, 3);
    }

    #[test]
    fn expired_token_detection() {
        let past = (Utc::now() - Duration::hours(2)).to_rfc3339();
        let future = (Utc::now() + Duration::hours(2)).to_rfc3339();
        assert!(tokens_expired(&past, Utc::now()));
        assert!(!tokens_expired(&future, Utc::now()));
    }

    #[test]
    fn auth_url_contains_pkce_and_scopes() {
        let url = build_auth_url("id.apps.googleusercontent.com", "http://127.0.0.1:8765", "challenge", "state123");
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("yt-analytics-monetary.readonly"));
        assert!(url.contains("access_type=offline"));
        assert!(url.contains("select_account"));
        assert!(url.contains("response_type=code"));
    }

    #[test]
    fn auth_url_includes_login_hint_when_given() {
        let url = build_auth_url_with_hint(
            "id.apps.googleusercontent.com",
            "http://127.0.0.1:8765",
            "challenge",
            "state123",
            Some("creator@example.com"),
        );
        assert!(url.contains("login_hint=creator%40example.com"));
        let no_hint = build_auth_url_with_hint("id", "http://127.0.0.1:8765", "c", "s", Some("not-an-email"));
        assert!(!no_hint.contains("login_hint"));
    }
}
