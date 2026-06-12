//! Template for the Google OAuth "Desktop app" client credentials.
//!
//! Copy this file to `google_secrets.rs` (which is gitignored) and fill in the
//! real Client ID + secret from the Google Cloud Console. `build.rs`
//! auto-creates `google_secrets.rs` from this template on first build if it is
//! missing, so a fresh clone always compiles — Drive sync just reports
//! "not configured" until the real values are filled in.
//!
//! Why gitignored: desktop OAuth client secrets are not truly confidential
//! (PKCE is what protects the flow, and the value ends up in any shipped
//! binary regardless), but keeping the literal string out of this public repo
//! avoids GitHub secret-scanning auto-rotating it.

pub const CLIENT_ID: &str = "REPLACE_WITH_GOOGLE_CLIENT_ID.apps.googleusercontent.com";
pub const CLIENT_SECRET: &str = "REPLACE_WITH_GOOGLE_CLIENT_SECRET";
