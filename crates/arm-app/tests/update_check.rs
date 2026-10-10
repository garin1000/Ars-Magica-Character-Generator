//! The optional update check (M6.6). Everything that decides anything is pure and
//! tested here: the tag and response parsing, the version comparison, the
//! ask-again rule and the consent gate in front of the fetch seam.
//!
//! The response is untrusted input (CLAUDE.md): it must be capped, parsed
//! strictly, and never panic, whatever bytes arrive.

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use arm_app::update_check::{
    FetchError, FetchLimits, check_with, fetch_latest_release, latest_release_api_url,
    newer_release, parse_latest_release, parse_release_tag, release_page_url, running_version,
    update_prompt_due,
};
use proptest::prelude::*;
use semver::Version;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn v(major: u64, minor: u64, patch: u64) -> Version {
    Version::new(major, minor, patch)
}

/// A latest-release body the way GitHub sends one: the three fields read, plus
/// fields that must be ignored.
fn release_body(tag: &str, draft: bool, prerelease: bool) -> Vec<u8> {
    format!(
        "{{\"url\": \"https://example.invalid\", \"html_url\": \"https://example.invalid\", \
         \"tag_name\": \"{tag}\", \"name\": \"Release\", \"draft\": {draft}, \
         \"prerelease\": {prerelease}, \"body\": \"<script>alert(1)</script>\", \
         \"assets\": [{{\"id\": 1}}]}}"
    )
    .into_bytes()
}

// ---------------------------------------------------------------------------
// The running version

#[test]
fn the_running_version_is_this_crates_cargo_version() {
    assert_eq!(
        running_version(),
        Version::parse(env!("CARGO_PKG_VERSION")).unwrap()
    );
}

/// `running_version()` reads the Cargo version, and the "is there a newer
/// release?" answer is only right if that is the version the user sees. The
/// version is written in four places nothing else ties together.
#[test]
fn the_version_agrees_in_every_manifest() {
    let cargo = env!("CARGO_PKG_VERSION");
    let json = |relative: &str| -> serde_json::Value {
        serde_json::from_str(&fs::read_to_string(repo_root().join(relative)).unwrap()).unwrap()
    };

    let tauri_conf = json("crates/arm-app/tauri.conf.json");
    assert_eq!(tauri_conf["version"], cargo, "tauri.conf.json");

    let package = json("ui/package.json");
    assert_eq!(package["version"], cargo, "ui/package.json");

    let lock = json("ui/package-lock.json");
    assert_eq!(lock["version"], cargo, "ui/package-lock.json (top level)");
    assert_eq!(
        lock["packages"][""]["version"], cargo,
        "ui/package-lock.json (root package entry)"
    );
}

// ---------------------------------------------------------------------------
// Release tags

#[test]
fn a_release_tag_parses_with_or_without_one_leading_v() {
    assert_eq!(parse_release_tag("v1.2.3"), Some(v(1, 2, 3)));
    assert_eq!(parse_release_tag("v0.4.0"), Some(v(0, 4, 0)));
    assert_eq!(parse_release_tag("1.2.3"), Some(v(1, 2, 3)));
}

#[test]
fn a_tag_that_is_not_a_plain_release_version_is_rejected() {
    // Control: the rejections below prove nothing unless a good tag parses.
    assert_eq!(parse_release_tag("v1.2.3"), Some(v(1, 2, 3)));

    let too_long = format!("v{}", "1".repeat(10 * 1024));
    let junk_10k = "x".repeat(10 * 1024);
    for tag in [
        "v1.2",
        "1.2.3-beta",
        "v1.2.3+build",
        "vv1.2.3",
        "v01.2.3",
        "v99999999999999999999.0.0",
        "",
        "v",
        too_long.as_str(),
        junk_10k.as_str(),
    ] {
        let shown: String = tag.chars().take(40).collect();
        assert_eq!(parse_release_tag(tag), None, "accepted {shown:?}");
    }
}

// ---------------------------------------------------------------------------
// The latest-release response

#[test]
fn a_published_release_yields_its_version_and_ignores_every_other_field() {
    assert_eq!(
        parse_latest_release(&release_body("v1.2.3", false, false)),
        Some(v(1, 2, 3))
    );
}

#[test]
fn a_draft_or_prerelease_is_not_an_update() {
    assert_eq!(
        parse_latest_release(&release_body("v9.0.0", false, false)),
        Some(v(9, 0, 0)),
        "control"
    );
    assert_eq!(
        parse_latest_release(&release_body("v9.0.0", true, false)),
        None
    );
    assert_eq!(
        parse_latest_release(&release_body("v9.0.0", false, true)),
        None
    );
}

#[test]
fn a_malformed_response_is_no_answer() {
    assert_eq!(
        parse_latest_release(br#"{"tag_name":"v2.0.0","draft":false,"prerelease":false}"#),
        Some(v(2, 0, 0)),
        "control"
    );

    let deep_array = format!("{}{}", "[".repeat(100_000), "]".repeat(100_000));
    let deep_tag = format!(
        "{{\"tag_name\": {}{}, \"draft\": false, \"prerelease\": false}}",
        "[".repeat(100_000),
        "]".repeat(100_000)
    );
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("non-UTF-8", b"{\"tag_name\":\"v2.0.0\xff\xfe\"}".to_vec()),
        (
            "truncated",
            br#"{"tag_name":"v2.0.0","draft":false,"prere"#.to_vec(),
        ),
        (
            "missing tag_name",
            br#"{"draft":false,"prerelease":false}"#.to_vec(),
        ),
        (
            "numeric tag_name",
            br#"{"tag_name":200,"draft":false,"prerelease":false}"#.to_vec(),
        ),
        ("empty", Vec::new()),
        ("not an object", br#""v2.0.0""#.to_vec()),
        ("deeply nested", deep_array.into_bytes()),
        ("deeply nested tag_name", deep_tag.into_bytes()),
        (
            "bad tag",
            br#"{"tag_name":"latest","draft":false,"prerelease":false}"#.to_vec(),
        ),
    ];
    for (label, body) in cases {
        assert_eq!(parse_latest_release(&body), None, "{label}");
    }
}

// ---------------------------------------------------------------------------
// The comparison

#[test]
fn only_a_strictly_newer_release_is_reported() {
    let running = v(0, 4, 0);
    assert_eq!(newer_release(&running, &v(0, 4, 1)), Some(v(0, 4, 1)));
    assert_eq!(newer_release(&running, &v(0, 5, 0)), Some(v(0, 5, 0)));
    assert_eq!(newer_release(&running, &v(1, 0, 0)), Some(v(1, 0, 0)));
    assert_eq!(newer_release(&running, &v(0, 4, 0)), None, "equal");
    assert_eq!(newer_release(&running, &v(0, 3, 9)), None, "older");
}

// ---------------------------------------------------------------------------
// The ask-again rule

#[test]
fn the_prompt_is_due_on_first_start_and_on_the_first_start_of_a_newer_version() {
    let running = v(0, 5, 0);
    assert!(update_prompt_due(None, &running), "never asked");
    assert!(
        update_prompt_due(Some("0.4.0"), &running),
        "asked by an older version"
    );
    assert!(update_prompt_due(Some("garbage"), &running), "unparsable");
    assert!(update_prompt_due(Some(""), &running), "empty");
    assert!(
        !update_prompt_due(Some("0.5.0"), &running),
        "already asked by this version"
    );
    assert!(
        !update_prompt_due(Some("0.6.0"), &running),
        "a downgrade does not ask"
    );
}

// ---------------------------------------------------------------------------
// The fixed URLs

#[test]
fn both_urls_are_fixed_and_derived_from_the_repository() {
    assert_eq!(
        release_page_url(),
        "https://github.com/garin1000/Ars-Magica-Character-Generator/releases/latest"
    );
    assert_eq!(
        latest_release_api_url(),
        "https://api.github.com/repos/garin1000/Ars-Magica-Character-Generator/releases/latest"
    );
}

// ---------------------------------------------------------------------------
// The consent gate

fn fetch_must_not_run() -> Result<Vec<u8>, FetchError> {
    panic!("the check fetched without consent");
}

#[test]
fn without_consent_nothing_is_fetched() {
    // Non-vacuous only together with the consented test below, which proves the
    // fetch IS called once the check is enabled.
    assert_eq!(check_with(None, fetch_must_not_run, &v(0, 4, 0)), None);
    assert_eq!(
        check_with(Some(false), fetch_must_not_run, &v(0, 4, 0)),
        None
    );
}

#[test]
fn with_consent_a_newer_release_is_reported_and_failures_stay_silent() {
    let calls = AtomicUsize::new(0);
    let newer = check_with(
        Some(true),
        || {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(release_body("v99.0.0", false, false))
        },
        &v(0, 4, 0),
    );
    assert_eq!(newer, Some(v(99, 0, 0)));
    assert_eq!(calls.load(Ordering::SeqCst), 1, "exactly one fetch");

    let same = check_with(
        Some(true),
        || Ok(release_body("v0.4.0", false, false)),
        &v(0, 4, 0),
    );
    assert_eq!(same, None, "the running version is not an update");

    for error in [
        FetchError::Status(403),
        FetchError::TooLarge,
        FetchError::Timeout,
        FetchError::Transport("refused".to_owned()),
    ] {
        assert_eq!(
            check_with(Some(true), || Err(error.clone()), &v(0, 4, 0)),
            None,
            "{error:?}"
        );
    }
    assert_eq!(
        check_with(Some(true), || Ok(b"not json".to_vec()), &v(0, 4, 0)),
        None
    );
}

// ---------------------------------------------------------------------------
// The real fetch, against a local server

const GOOD_BODY: &[u8] = br#"{"tag_name":"v99.0.0","draft":false,"prerelease":false}"#;

/// Test limits: plain `http` to 127.0.0.1, a short timeout, a small cap.
fn test_limits() -> FetchLimits {
    FetchLimits {
        timeout: Duration::from_secs(5),
        max_bytes: 1024,
        https_only: false,
    }
}

/// Reads one request's head (up to the blank line) and returns it as text.
fn read_request_head(stream: &mut TcpStream) -> String {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        match stream.read(&mut byte) {
            Ok(1) => head.push(byte[0]),
            _ => break,
        }
    }
    String::from_utf8_lossy(&head).into_owned()
}

/// A one-shot HTTP server on 127.0.0.1: for each of `responses`, accept one
/// connection, read the request head, send that response and close. The request
/// heads arrive on the returned channel.
fn serve(responses: Vec<Vec<u8>>) -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/releases/latest", listener.local_addr().unwrap());
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        for response in responses {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let head = read_request_head(&mut stream);
            let _ = sender.send(head);
            let _ = stream.write_all(&response);
        }
    });
    (url, receiver)
}

fn response(status: &str, headers: &str, body: &[u8]) -> Vec<u8> {
    let mut bytes = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\n{headers}\
         Connection: close\r\n\r\n"
    )
    .into_bytes();
    bytes.extend_from_slice(body);
    bytes
}

fn ok_response(body: &[u8]) -> Vec<u8> {
    response(
        "200 OK",
        &format!("Content-Length: {}\r\n", body.len()),
        body,
    )
}

#[test]
fn the_production_limits_are_ten_seconds_half_a_mebibyte_and_https_only() {
    assert_eq!(
        FetchLimits::production(),
        FetchLimits {
            timeout: Duration::from_secs(10),
            max_bytes: 512 * 1024,
            https_only: true,
        }
    );
}

#[test]
fn a_good_response_is_returned_and_the_request_names_itself_to_github() {
    let (url, requests) = serve(vec![ok_response(GOOD_BODY)]);
    assert_eq!(
        fetch_latest_release(&url, &test_limits()),
        Ok(GOOD_BODY.to_vec())
    );

    let head = requests.recv().unwrap().to_ascii_lowercase();
    assert!(head.starts_with("get "), "{head}");
    assert!(
        head.contains(&format!(
            "user-agent: arm-char-gen/{}\r\n",
            running_version()
        )),
        "GitHub refuses requests without a User-Agent: {head}"
    );
    assert!(
        head.contains("accept: application/vnd.github+json\r\n"),
        "{head}"
    );
    assert!(
        head.contains("x-github-api-version: 2026-03-10\r\n"),
        "{head}"
    );
}

#[test]
fn a_body_over_the_cap_is_refused() {
    let big = vec![b' '; 2048];
    let (url, _requests) = serve(vec![ok_response(&big)]);
    assert_eq!(
        fetch_latest_release(&url, &test_limits()),
        Err(FetchError::TooLarge)
    );

    // Without a Content-Length the cap must still hold while streaming.
    let (url, _requests) = serve(vec![response("200 OK", "", &big)]);
    assert_eq!(
        fetch_latest_release(&url, &test_limits()),
        Err(FetchError::TooLarge)
    );
}

#[test]
fn a_server_that_stalls_times_out() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/releases/latest", listener.local_addr().unwrap());
    thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        read_request_head(&mut stream);
        // Never answer: block until the client gives up and drops the connection.
        let mut rest = Vec::new();
        let _ = stream.read_to_end(&mut rest);
    });

    let limits = FetchLimits {
        timeout: Duration::from_millis(300),
        ..test_limits()
    };
    let started = Instant::now();
    let result = fetch_latest_release(&url, &limits);
    let elapsed = started.elapsed();
    assert_eq!(result, Err(FetchError::Timeout));
    assert!(
        elapsed < Duration::from_secs(3),
        "the timeout did not bound the call: {elapsed:?}"
    );
}

#[test]
fn an_error_status_is_an_error() {
    for (status, code) in [("403 Forbidden", 403), ("404 Not Found", 404)] {
        let (url, _requests) = serve(vec![response(
            status,
            &format!("Content-Length: {}\r\n", GOOD_BODY.len()),
            GOOD_BODY,
        )]);
        assert_eq!(
            fetch_latest_release(&url, &test_limits()),
            Err(FetchError::Status(code)),
            "{status}"
        );
    }
}

#[test]
fn a_redirect_is_not_followed() {
    // The second response is what a followed redirect would get: a valid body.
    let (url, requests) = serve(vec![
        response(
            "301 Moved Permanently",
            "Location: /elsewhere\r\nContent-Length: 0\r\n",
            b"",
        ),
        ok_response(GOOD_BODY),
    ]);
    let result = fetch_latest_release(&url, &test_limits());
    assert!(result.is_err(), "a redirect was followed: {result:?}");
    assert!(requests.recv().is_ok(), "the first request never arrived");
    assert!(
        requests.recv_timeout(Duration::from_millis(200)).is_err(),
        "a second request was made"
    );
}

#[test]
fn a_refused_connection_is_an_error() {
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    // The listener is dropped, so nothing listens on `port` any more.
    let result = fetch_latest_release(
        &format!("http://127.0.0.1:{port}/releases/latest"),
        &test_limits(),
    );
    assert!(
        matches!(result, Err(FetchError::Transport(_))),
        "got {result:?}"
    );
}

#[test]
fn plain_http_is_refused_when_https_only() {
    let (url, requests) = serve(vec![ok_response(GOOD_BODY)]);
    let limits = FetchLimits {
        https_only: true,
        ..test_limits()
    };
    let result = fetch_latest_release(&url, &limits);
    assert!(
        matches!(result, Err(FetchError::Transport(_))),
        "got {result:?}"
    );
    assert!(
        requests.recv_timeout(Duration::from_millis(200)).is_err(),
        "a plain-http request went out"
    );
}

// ---------------------------------------------------------------------------
// Decision 1: the webview gains no network reach and no capability

#[test]
fn the_webview_gains_no_capability_and_no_remote_connect_source() {
    let capability: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo_root().join("crates/arm-app/capabilities/default.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        capability["permissions"],
        serde_json::json!(["core:default"]),
        "the update check runs in Rust; the webview needs no new permission"
    );

    let conf: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo_root().join("crates/arm-app/tauri.conf.json")).unwrap(),
    )
    .unwrap();
    let csp = conf["app"]["security"]["csp"]
        .as_str()
        .expect("tauri.conf.json must declare a CSP string");
    let connect_src = csp
        .split(';')
        .map(str::trim)
        .find(|directive| directive.starts_with("connect-src"))
        .expect("the CSP must declare connect-src");
    assert!(
        !connect_src.contains("https:") && !connect_src.contains("github"),
        "the webview must not be able to reach the network: {connect_src}"
    );
}

// ---------------------------------------------------------------------------
// Never panic

proptest! {
    #[test]
    fn arbitrary_response_bytes_never_panic(body in prop::collection::vec(any::<u8>(), 0..4096)) {
        let _ = parse_latest_release(&body);
    }

    #[test]
    fn arbitrary_json_shaped_bytes_never_panic(
        tag in ".{0,64}",
        draft in any::<bool>(),
        prerelease in any::<bool>(),
    ) {
        let _ = parse_latest_release(&serde_json::to_vec(&serde_json::json!({
            "tag_name": tag, "draft": draft, "prerelease": prerelease,
        })).unwrap());
    }

    #[test]
    fn arbitrary_tags_never_panic(tag in any::<String>()) {
        let _ = parse_release_tag(&tag);
        let _ = update_prompt_due(Some(&tag), &v(0, 4, 0));
    }
}
