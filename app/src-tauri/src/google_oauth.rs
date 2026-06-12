//! Desktop OAuth 2.0 for Google Drive: loopback-redirect + PKCE sign-in, refresh
//! token kept in the OS keychain (Windows Credential Manager via `keyring`),
//! and access-token refresh.
//!
//! Why this shape: a desktop app has no safe place for a client secret and no
//! fixed redirect URL, so Google's "installed app" flow uses a localhost
//! loopback redirect plus PKCE. We spin up a one-shot HTTP listener on a random
//! 127.0.0.1 port, send the user to Google's consent page in their browser, and
//! catch the redirect back with the authorization code. The code is exchanged
//! for tokens; only the long-lived refresh token is persisted, and only in the
//! OS keychain — never in the settings JSON or the SQLite DB.
//!
//! The client id/secret are filled in from a Google Cloud "Desktop app" OAuth
//! client. Desktop client secrets are not confidential (PKCE is what actually
//! protects the exchange), but Google still requires the secret in the token
//! call for desktop clients, so we ship both.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Duration;

use base64::Engine as _;
use sha2::{Digest, Sha256};

use crate::error::CommandError;

// Filled in from the Google Cloud Console "Desktop app" OAuth client. Until
// these are set, `is_configured()` is false and the commands return a clear
// "not configured" error instead of hitting Google with a placeholder.
const CLIENT_ID: &str = "REPLACE_WITH_GOOGLE_CLIENT_ID.apps.googleusercontent.com";
const CLIENT_SECRET: &str = "REPLACE_WITH_GOOGLE_CLIENT_SECRET";

const AUTH_URI: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URI: &str = "https://oauth2.googleapis.com/token";
/// Least privilege: app-created files only. Also keeps us out of Google's
/// restricted-scope security assessment.
const SCOPE: &str = "https://www.googleapis.com/auth/drive.file";
/// Drive's `about` resource returns the signed-in user (incl. email) and is
/// reachable with `drive.file` alone, so we learn the email without asking for
/// an extra `email`/`openid` scope.
const ABOUT_URI: &str = "https://www.googleapis.com/drive/v3/about?fields=user";

/// Username component of the keychain entry. The service component is the app's
/// bundle identifier (passed in) so the Dev flavor and stable keep separate
/// tokens.
const KEYCHAIN_USER: &str = "google-refresh-token";

/// How long we wait for the user to finish the consent flow in their browser.
const AUTH_TIMEOUT: Duration = Duration::from_secs(300);
const TOKEN_TIMEOUT: Duration = Duration::from_secs(30);

/// True once the shipped client id/secret have been filled in. The UI uses this
/// to explain that Drive sync needs a configured build, rather than failing
/// mid-flow against Google.
pub fn is_configured() -> bool {
    !CLIENT_ID.starts_with("REPLACE_") && !CLIENT_SECRET.starts_with("REPLACE_")
}

/// Runs the full interactive sign-in: opens the consent page (via `open_url`),
/// catches the loopback redirect, exchanges the code, stores the refresh token
/// in the keychain under `service`, and returns the signed-in account email.
pub fn authorize(
    service: &str,
    open_url: impl FnOnce(&str) -> Result<(), CommandError>,
) -> Result<String, CommandError> {
    if !is_configured() {
        return Err(not_configured());
    }

    let verifier = random_token();
    let challenge = code_challenge(&verifier);
    let state = random_token();

    // Bind first so we can put the actual port in the redirect URI.
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|error| failure(format!("Could not open a local callback port. {error}")))?;
    let port = listener
        .local_addr()
        .map_err(|error| failure(error.to_string()))?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{port}");

    let auth_url = format!(
        "{AUTH_URI}?response_type=code&client_id={}&redirect_uri={}&scope={}\
         &code_challenge={}&code_challenge_method=S256&state={}\
         &access_type=offline&prompt=consent",
        form_encode(CLIENT_ID),
        form_encode(&redirect_uri),
        form_encode(SCOPE),
        form_encode(&challenge),
        form_encode(&state),
    );

    open_url(&auth_url)?;

    let code = wait_for_code(&listener, &state)?;
    let tokens = exchange_code(&code, &verifier, &redirect_uri)?;

    let refresh = tokens.refresh_token.ok_or_else(|| {
        failure(
            "Google did not return a refresh token. Remove Scribe from your Google account's \
             third-party access and sign in again.",
        )
    })?;
    store_refresh_token(service, &refresh)?;

    let email = fetch_email(&tokens.access_token).unwrap_or_default();
    Ok(email)
}

/// Removes the stored refresh token. Signing out is best-effort: a missing
/// entry is success.
pub fn sign_out(service: &str) -> Result<(), CommandError> {
    let entry = keychain_entry(service)?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(failure(format!("Could not clear the saved Google token. {error}"))),
    }
}

/// Exchanges the stored refresh token for a fresh access token. This is the
/// entry point every Drive operation uses; access tokens are short-lived so we
/// fetch one per sync session.
pub fn access_token(service: &str) -> Result<String, CommandError> {
    if !is_configured() {
        return Err(not_configured());
    }
    let refresh = load_refresh_token(service)?;
    let body = format!(
        "grant_type=refresh_token&refresh_token={}&client_id={}&client_secret={}",
        form_encode(&refresh),
        form_encode(CLIENT_ID),
        form_encode(CLIENT_SECRET),
    );
    let tokens = post_token(&body)?;
    Ok(tokens.access_token)
}

/// True when a refresh token is present in the keychain for `service`.
pub fn has_stored_token(service: &str) -> bool {
    keychain_entry(service)
        .and_then(|entry| match entry.get_password() {
            Ok(_) => Ok(true),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(error) => Err(failure(error.to_string())),
        })
        .unwrap_or(false)
}

// --- internals ---------------------------------------------------------------

struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
}

fn exchange_code(
    code: &str,
    verifier: &str,
    redirect_uri: &str,
) -> Result<TokenResponse, CommandError> {
    let body = format!(
        "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}\
         &client_secret={}&code_verifier={}",
        form_encode(code),
        form_encode(redirect_uri),
        form_encode(CLIENT_ID),
        form_encode(CLIENT_SECRET),
        form_encode(verifier),
    );
    post_token(&body)
}

fn post_token(body: &str) -> Result<TokenResponse, CommandError> {
    let client = http_client()?;
    let response = client
        .post(TOKEN_URI)
        .timeout(TOKEN_TIMEOUT)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body.to_string())
        .send()
        .map_err(|error| failure(format!("Could not reach Google's token endpoint. {error}")))?;

    let status = response.status();
    let text = response
        .text()
        .map_err(|error| failure(format!("Could not read Google's token response. {error}")))?;

    if !status.is_success() {
        return Err(failure(format!(
            "Google rejected the sign-in (HTTP {status}). {}",
            truncate(&text, 300)
        )));
    }

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|error| failure(format!("Could not parse Google's token response. {error}")))?;

    let access_token = json
        .get("access_token")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| failure("Google's token response had no access token."))?
        .to_string();

    let refresh_token = json
        .get("refresh_token")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    Ok(TokenResponse { access_token, refresh_token })
}

fn fetch_email(access_token: &str) -> Result<String, CommandError> {
    let client = http_client()?;
    let text = client
        .get(ABOUT_URI)
        .timeout(TOKEN_TIMEOUT)
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .map_err(|error| failure(error.to_string()))?
        .text()
        .map_err(|error| failure(error.to_string()))?;

    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|error| failure(error.to_string()))?;
    Ok(json
        .get("user")
        .and_then(|user| user.get("emailAddress"))
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string())
}

/// Accepts the single loopback redirect, validates the CSRF `state`, replies
/// with a friendly page, and returns the authorization code.
fn wait_for_code(listener: &TcpListener, expected_state: &str) -> Result<String, CommandError> {
    // Poll for the redirect so we can enforce an overall timeout rather than
    // blocking forever if the user abandons the consent page.
    listener
        .set_nonblocking(true)
        .map_err(|error| failure(error.to_string()))?;
    let deadline = std::time::Instant::now() + AUTH_TIMEOUT;

    loop {
        if std::time::Instant::now() >= deadline {
            return Err(failure(
                "Timed out waiting for Google sign-in. Please try again.",
            ));
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .ok();
                let request_line = read_request_line(&mut stream);
                let target = request_line
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("")
                    .to_string();

                let params = parse_query(&target);
                let result = match (
                    params.iter().find(|(k, _)| k == "error").map(|(_, v)| v),
                    params.iter().find(|(k, _)| k == "state").map(|(_, v)| v),
                    params.iter().find(|(k, _)| k == "code").map(|(_, v)| v),
                ) {
                    (Some(error), _, _) => {
                        respond(&mut stream, false);
                        Err(failure(format!("Google sign-in was denied ({error}).")))
                    }
                    (_, state, Some(code)) if state.map(String::as_str) == Some(expected_state) => {
                        respond(&mut stream, true);
                        Ok(code.clone())
                    }
                    (_, _, Some(_)) => {
                        respond(&mut stream, false);
                        Err(failure("Sign-in state mismatch; please try again."))
                    }
                    _ => {
                        // Browsers also request /favicon.ico etc. Ignore and
                        // keep waiting for the real redirect.
                        respond(&mut stream, false);
                        continue;
                    }
                };
                return result;
            }
            Err(ref error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(error) => return Err(failure(error.to_string())),
        }
    }
}

fn read_request_line(stream: &mut std::net::TcpStream) -> String {
    let mut buffer = [0_u8; 4096];
    let read = stream.read(&mut buffer).unwrap_or(0);
    let text = String::from_utf8_lossy(&buffer[..read]);
    text.lines().next().unwrap_or("").to_string()
}

fn respond(stream: &mut std::net::TcpStream, ok: bool) {
    let message = if ok {
        "You're signed in. You can close this tab and return to Scribe."
    } else {
        "Sign-in could not be completed. You can close this tab and try again in Scribe."
    };
    let html = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Scribe</title></head>\
         <body style=\"font-family:system-ui;background:#0d1320;color:#e2e8f0;\
         display:flex;align-items:center;justify-content:center;height:100vh;margin:0\">\
         <p style=\"font-size:18px\">{message}</p></body></html>"
    );
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );
    let _ = stream.write_all(response.as_bytes());
}

// --- keychain ----------------------------------------------------------------

fn keychain_entry(service: &str) -> Result<keyring::Entry, CommandError> {
    keyring::Entry::new(service, KEYCHAIN_USER)
        .map_err(|error| failure(format!("The OS keychain is unavailable. {error}")))
}

fn store_refresh_token(service: &str, token: &str) -> Result<(), CommandError> {
    keychain_entry(service)?
        .set_password(token)
        .map_err(|error| failure(format!("Could not save the Google token to the keychain. {error}")))
}

fn load_refresh_token(service: &str) -> Result<String, CommandError> {
    match keychain_entry(service)?.get_password() {
        Ok(token) => Ok(token),
        Err(keyring::Error::NoEntry) => Err(failure(
            "Not signed in to Google. Open Settings → Integrations and sign in.",
        )),
        Err(error) => Err(failure(format!("Could not read the saved Google token. {error}"))),
    }
}

// --- helpers -----------------------------------------------------------------

fn http_client() -> Result<reqwest::blocking::Client, CommandError> {
    reqwest::blocking::Client::builder()
        .user_agent(concat!("Scribe/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| failure(error.to_string()))
}

/// 32 bytes of CSPRNG (two v4 UUIDs) as URL-safe base64 — a valid PKCE verifier
/// (43 chars, all unreserved) and a fine CSRF state value. Uses `uuid`'s
/// getrandom-backed generator, so no extra RNG dependency.
fn random_token() -> String {
    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(uuid::Uuid::new_v4().as_bytes());
    bytes.extend_from_slice(uuid::Uuid::new_v4().as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn code_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
}

/// Percent-encodes a value for a query string or x-www-form-urlencoded body,
/// leaving only the RFC 3986 unreserved set. Hand-rolled to avoid pulling in a
/// URL crate just for this.
fn form_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn parse_query(target: &str) -> Vec<(String, String)> {
    let query = target.split_once('?').map(|(_, q)| q).unwrap_or("");
    query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            (percent_decode(key), percent_decode(value))
        })
        .collect()
}

fn percent_decode(value: &str) -> String {
    let bytes = value.replace('+', " ");
    let bytes = bytes.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(decoded) = u8::from_str_radix(&value[i + 1..i + 3], 16) {
                out.push(decoded);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn not_configured() -> CommandError {
    CommandError::new(
        "google_not_configured",
        "This build has no Google OAuth client configured, so Drive sync is unavailable.",
    )
}

fn failure(message: impl Into<String>) -> CommandError {
    CommandError::new("google_auth_failed", message)
}

fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_chars).collect();
        format!("{truncated}…")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_verifier_is_valid_length_and_charset() {
        let verifier = random_token();
        assert_eq!(verifier.len(), 43, "32 bytes base64url-nopad is 43 chars");
        assert!(verifier
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'));
    }

    #[test]
    fn code_challenge_matches_known_rfc7636_vector() {
        // RFC 7636 Appendix B.
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        assert_eq!(
            code_challenge(verifier),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn form_encode_escapes_reserved_chars() {
        assert_eq!(form_encode("a/b=c&d"), "a%2Fb%3Dc%26d");
        assert_eq!(form_encode("http://127.0.0.1:1234"), "http%3A%2F%2F127.0.0.1%3A1234");
        // Unreserved set is left intact.
        assert_eq!(form_encode("Aa0-_.~"), "Aa0-_.~");
    }

    #[test]
    fn parse_query_decodes_pairs() {
        let params = parse_query("/?code=4%2F0Ab&state=xyz&scope=a+b");
        assert_eq!(params.iter().find(|(k, _)| k == "code").unwrap().1, "4/0Ab");
        assert_eq!(params.iter().find(|(k, _)| k == "state").unwrap().1, "xyz");
        assert_eq!(params.iter().find(|(k, _)| k == "scope").unwrap().1, "a b");
    }

    #[test]
    fn unconfigured_build_reports_clearly() {
        // The shipped placeholders must read as "not configured" until filled.
        if !is_configured() {
            let error = authorize("svc", |_| Ok(())).unwrap_err();
            assert_eq!(error.code, "google_not_configured");
        }
    }
}
