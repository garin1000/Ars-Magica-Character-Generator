//! The optional update check (M6.6): ask GitHub's latest-release endpoint whether
//! a newer version exists, and say so. Nothing is ever downloaded.
//!
//! Everything that decides anything is a pure function here, so it is unit-tested
//! without a network: the tag and response parsing, the version comparison, the
//! ask-again rule, and [`check_with`] — the consent gate in front of the fetch seam.
//!
//! **The response is untrusted input** (CLAUDE.md). Only three fields are read,
//! strictly; the one value that can ever reach the screen is the parsed semver
//! version, and no URL is ever taken from the response — the release-page link is
//! the fixed [`release_page_url`].

use std::time::Duration;

use semver::Version;
use serde::Deserialize;

/// The bounds one fetch runs under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchLimits {
    /// The whole request, connect to last body byte.
    pub timeout: Duration,
    /// The largest body accepted.
    pub max_bytes: u64,
    /// Refuse any non-`https` URL.
    pub https_only: bool,
}

impl FetchLimits {
    /// The limits the shipped check uses: 10 s for the whole call, 512 KiB of
    /// body (a real latest-release response is a few KiB), `https` only.
    pub fn production() -> Self {
        FetchLimits {
            timeout: Duration::from_secs(10),
            max_bytes: 512 * 1024,
            https_only: true,
        }
    }
}

/// The REST API version the request asks for, so a later default cannot change
/// the response shape under the parser. Checked on docs.github.com 2026-10-10.
const GITHUB_API_VERSION: &str = "2026-03-10";

/// GET `url` under `limits` and return its body.
///
/// Any non-2xx status is an error, and so is a redirect: the API endpoint never
/// redirects, so `max_redirects(0)` follows none. The body is read through
/// ureq's limit, so an oversized response stops at the cap instead of filling
/// memory. Never panics.
pub fn fetch_latest_release(url: &str, limits: &FetchLimits) -> Result<Vec<u8>, FetchError> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(limits.timeout))
        .https_only(limits.https_only)
        .max_redirects(0)
        .http_status_as_error(true)
        .user_agent(format!("arm-char-gen/{}", running_version()))
        .build()
        .into();
    let mut response = agent
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
        .call()
        .map_err(fetch_error)?;
    let status = response.status();
    if !status.is_success() {
        return Err(FetchError::Status(status.as_u16()));
    }
    response
        .body_mut()
        .with_config()
        .limit(limits.max_bytes)
        .read_to_vec()
        .map_err(fetch_error)
}

/// ureq's error, reduced to the few cases the check distinguishes.
fn fetch_error(error: ureq::Error) -> FetchError {
    match error {
        ureq::Error::StatusCode(code) => FetchError::Status(code),
        ureq::Error::Timeout(_) => FetchError::Timeout,
        ureq::Error::BodyExceedsLimit(_) => FetchError::TooLarge,
        other => FetchError::Transport(other.to_string()),
    }
}

/// Why fetching the latest release failed. Every variant is silent to the user:
/// a failed check simply shows no notice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchError {
    /// The server answered with a non-success status.
    Status(u16),
    /// The response body was larger than the cap.
    TooLarge,
    /// The request did not finish within the timeout.
    Timeout,
    /// Anything else: connection, TLS, redirect, IO.
    Transport(String),
}

/// The version of this build, as Cargo knows it.
///
/// Cargo refuses to build a package whose version is not valid semver, so the
/// `expect` cannot fire at runtime; `tests/update_check.rs` pins that this is the
/// version every other manifest declares too.
pub fn running_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("Cargo validates the package version")
}

/// A release tag (`v1.2.3`) as a plain release version.
///
/// Exactly one leading `v` is stripped. A pre-release or build-metadata suffix is
/// rejected (it is not a release), and so is anything `semver` refuses: missing
/// parts, leading zeros, numbers that overflow.
pub fn parse_release_tag(tag: &str) -> Option<Version> {
    let bare = tag.strip_prefix('v').unwrap_or(tag);
    let version = Version::parse(bare).ok()?;
    if !version.pre.is_empty() || !version.build.is_empty() {
        return None;
    }
    Some(version)
}

/// The three fields of GitHub's latest-release response the check reads. Every
/// other field — `html_url`, `body`, `name` included — is ignored, never stored.
#[derive(Deserialize)]
struct LatestRelease {
    tag_name: String,
    draft: bool,
    prerelease: bool,
}

/// The version of a latest-release response body, or `None` for anything that is
/// not a published release with a plain version tag.
///
/// `serde_json`'s recursion limit bounds a deeply nested document.
pub fn parse_latest_release(body: &[u8]) -> Option<Version> {
    let release: LatestRelease = serde_json::from_slice(body).ok()?;
    if release.draft || release.prerelease {
        return None;
    }
    parse_release_tag(&release.tag_name)
}

/// `latest`, when it is strictly newer than `running`.
pub fn newer_release(running: &Version, latest: &Version) -> Option<Version> {
    (latest > running).then(|| latest.clone())
}

/// Whether the consent prompt must be shown: on the first start (nothing
/// recorded, or nothing usable), and on the first start of a newer version than
/// the one that last asked. A downgrade does not ask again.
pub fn update_prompt_due(asked: Option<&str>, running: &Version) -> bool {
    match asked.and_then(|text| Version::parse(text).ok()) {
        None => true,
        Some(asked) => running > &asked,
    }
}

/// The project's repository, as the workspace manifest declares it.
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// The `owner/name` part of [`REPOSITORY`].
fn repository_slug() -> &'static str {
    REPOSITORY
        .strip_prefix("https://github.com/")
        .unwrap_or(REPOSITORY)
}

/// The project's release page: the fixed link the update notice opens.
pub fn release_page_url() -> String {
    format!("{REPOSITORY}/releases/latest")
}

/// GitHub's latest-release API endpoint for this project. It ignores drafts.
pub fn latest_release_api_url() -> String {
    format!(
        "https://api.github.com/repos/{}/releases/latest",
        repository_slug()
    )
}

/// The whole check, behind its consent gate: `fetch` runs only when the user has
/// agreed (`enabled == Some(true)`), and every failure — fetch, parse, an equal or
/// older release — is `None`.
///
/// This is the seam the tests drive: production passes the real HTTP fetch, a
/// test passes a closure.
pub fn check_with(
    enabled: Option<bool>,
    fetch: impl FnOnce() -> Result<Vec<u8>, FetchError>,
    running: &Version,
) -> Option<Version> {
    if enabled != Some(true) {
        return None;
    }
    let body = fetch().ok()?;
    let latest = parse_latest_release(&body)?;
    newer_release(running, &latest)
}
