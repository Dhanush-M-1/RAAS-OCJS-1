mod docker;
mod models;
mod moderator;
mod policy;
mod predict;
mod queue;

use axum::{
    Json, Router,
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use models::{HealthCheck, JudgeResult, Submission};
use policy::TierPolicy;
use queue::{start, submit};
use std::sync::OnceLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;

/// Largest accepted submission body. Submissions are source text, so 2 MiB is
/// far beyond any genuine program while still bounding a request that would
/// otherwise be read into memory with no ceiling at all.
const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;

/// Header carrying the shared secret.
const TOKEN_HEADER: &str = "x-raas-token";

/// Shortest token we are willing to enforce. Anything shorter is refused as
/// configuration rather than honoured as a secret.
const MIN_TOKEN_LEN: usize = 16;

fn is_acceptable_token(candidate: &str) -> bool {
    candidate.len() >= MIN_TOKEN_LEN
}

/// The shared secret, read once from `RAAS_AUTH_TOKEN`.
///
/// A token shorter than `MIN_TOKEN_LEN` is treated as absent rather than
/// enforced: it is too weak to be worth anything and far more likely to be a
/// misconfiguration than an intentional choice, so failing closed on it would
/// only be confusing.
fn auth_token() -> Option<&'static str> {
    static TOKEN: OnceLock<Option<String>> = OnceLock::new();
    TOKEN
        .get_or_init(|| {
            std::env::var("RAAS_AUTH_TOKEN")
                .ok()
                .map(|t| t.trim().to_string())
                .filter(|t| is_acceptable_token(t))
        })
        .as_deref()
}

/// Compare without leaking length-independent timing information.
///
/// The length check short-circuits, which is fine: the token length is fixed by
/// configuration and not a secret worth protecting.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn token_is_valid(headers: &HeaderMap, expected: &str) -> bool {
    headers
        .get(TOKEN_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|supplied| constant_time_eq(supplied.as_bytes(), expected.as_bytes()))
        .unwrap_or(false)
}

/// Reject `/submit` unless the caller presents the shared secret.
///
/// With no `RAAS_AUTH_TOKEN` configured this is a pass-through, so local
/// development and the benchmark harness keep working unchanged. That is
/// exactly why the server prints a loud warning in that case: an unauthenticated
/// judge executes arbitrary submitted code, so running without a token is only
/// ever acceptable on a machine nobody else can reach.
async fn require_auth(req: Request, next: Next) -> Response {
    match auth_token() {
        None => next.run(req).await,
        Some(expected) => {
            if token_is_valid(req.headers(), expected) {
                next.run(req).await
            } else {
                (
                    StatusCode::UNAUTHORIZED,
                    "missing or invalid x-raas-token\n",
                )
                    .into_response()
            }
        }
    }
}

/// Bind address. Defaults to the historical `0.0.0.0:3000` so nothing that
/// already depends on it changes, but overridable so a deployment can keep the
/// judge on loopback and expose it only through a tunnel or reverse proxy.
fn bind_addr() -> String {
    std::env::var("RAAS_BIND").unwrap_or_else(|_| "0.0.0.0:3000".to_string())
}

/// CORS policy. `RAAS_ALLOWED_ORIGINS` takes a comma-separated allowlist; with
/// the variable unset we keep the permissive historical behaviour and say so.
fn cors_layer() -> CorsLayer {
    let configured = std::env::var("RAAS_ALLOWED_ORIGINS")
        .ok()
        .filter(|v| !v.trim().is_empty());

    match configured {
        Some(list) => {
            let origins: Vec<HeaderValue> = list
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .filter_map(|s| match s.parse::<HeaderValue>() {
                    Ok(v) => Some(v),
                    Err(_) => {
                        eprintln!("[config] ignoring unparseable origin: {s:?}");
                        None
                    }
                })
                .collect();
            println!(
                "[config] CORS allowlist: {} origin(s)",
                origins.len()
            );
            CorsLayer::new()
                .allow_origin(origins)
                .allow_headers(Any)
                .allow_methods(Any)
        }
        None => {
            println!(
                "[config] RAAS_ALLOWED_ORIGINS unset - allowing any origin (historical default)"
            );
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers(Any)
                .allow_methods(Any)
        }
    }
}

async fn health_check() -> Json<HealthCheck> {
    println!("[HEALTH CHECK] Returned Status OK");
    return Json(HealthCheck {
        status: String::from("OK"),
    });
}

async fn judge(submission: Submission, policy: &(dyn TierPolicy + Send + Sync)) -> JudgeResult {
    let tier = policy.initial_tier(&submission);
    let tier_started = tier.name().to_string();
    let start = std::time::Instant::now();
    let outcome = match docker::run_submission(&submission, &tier, policy, start).await {
        Ok(o) => o,
        Err(e) => {
            println!("{}", e.to_string());
            return JudgeResult {
                submission_id: submission.id.clone(),
                approach: policy.name().to_string(),
                verdict: "SE".to_string(),
                cpu_time_ms: 0,
                peak_memory_bytes: 0,
                allocated_memory_bytes: 0,
                wall_time_ms: 0,
                tier_started,
                tier_promoted: false,
                promotion_time_ms: 0,
                cases: vec![],
            };
        }
    };
    let wall_ms = start.elapsed().as_millis() as u64;
    let cpu_ms = outcome.results.iter().map(|c| c.cpu_time_ms).sum();
    let mem = outcome
        .results
        .iter()
        .map(|c| c.peak_memory_bytes)
        .max()
        .unwrap_or(0);
    let allocated_mem = if outcome.tier_promoted || tier == policy::Tier::High {
        0
    } else {
        docker::low_mem_hard_limit()
    };
    let verdict = outcome
        .results
        .iter()
        .find(|c| c.verdict != "AC")
        .map(|c| c.verdict.as_str())
        .unwrap_or("AC");
    JudgeResult {
        submission_id: submission.id.clone(),
        approach: policy.name().to_string(),
        verdict: verdict.to_string(),
        cpu_time_ms: cpu_ms,
        peak_memory_bytes: mem,
        allocated_memory_bytes: allocated_mem,
        wall_time_ms: wall_ms,
        tier_started,
        tier_promoted: outcome.tier_promoted,
        promotion_time_ms: outcome.promotion_time_ms,
        cases: outcome.results,
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Ensure native Docker engine is used so host cgroups are directly accessible
    if std::path::Path::new("/var/run/docker.sock").exists() {
        unsafe {
            std::env::set_var("DOCKER_CONTEXT", "default");
        }
    }
    let info = tokio::process::Command::new("docker")
        .arg("info")
        .output()
        .await;
    match info {
        Ok(o) if o.status.success() => {}
        _ => return Err(std::io::Error::other("Docker is not running")),
    }

    let addr = bind_addr();
    match auth_token() {
        Some(_) => println!("[config] /submit requires the {TOKEN_HEADER} header"),
        None => println!(
            "[config] WARNING: RAAS_AUTH_TOKEN is unset - /submit is UNAUTHENTICATED and \
             will compile and execute whatever it is sent. Do not expose this port."
        ),
    }

    let app = Router::new()
        .route("/health", get(health_check))
        .route(
            "/submit",
            post(submit).layer(middleware::from_fn(require_auth)),
        )
        .layer(cors_layer())
        .layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
        .with_state(start());

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Judge is online and listening on {addr}");
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_time_eq_behaves_like_equality() {
        assert!(constant_time_eq(b"abcdefghij012345", b"abcdefghij012345"));
        assert!(!constant_time_eq(b"abcdefghij012345", b"abcdefghij012346"));
        assert!(!constant_time_eq(b"short", b"a-much-longer-value"));
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn token_matching_requires_exact_match() {
        let mut headers = HeaderMap::new();
        headers.insert(TOKEN_HEADER, HeaderValue::from_static("correct-horse-battery"));
        assert!(token_is_valid(&headers, "correct-horse-battery"));

        headers.insert(TOKEN_HEADER, HeaderValue::from_static("wrong-horse-battery"));
        assert!(!token_is_valid(&headers, "correct-horse-battery"));

        // A prefix must not be accepted.
        headers.insert(TOKEN_HEADER, HeaderValue::from_static("correct-horse-batter"));
        assert!(!token_is_valid(&headers, "correct-horse-battery"));
    }

    #[test]
    fn missing_header_is_rejected() {
        let headers = HeaderMap::new();
        assert!(!token_is_valid(&headers, "correct-horse-battery"));
    }

    #[test]
    fn header_lookup_is_case_insensitive() {
        // HTTP header names are case-insensitive; a proxy may normalise them.
        let mut headers = HeaderMap::new();
        headers.insert("X-RAAS-TOKEN", HeaderValue::from_static("correct-horse-battery"));
        assert!(token_is_valid(&headers, "correct-horse-battery"));
    }

    #[test]
    fn short_tokens_are_treated_as_unset() {
        // A weak token must not be honoured; it should fall through to the
        // loud "unauthenticated" warning instead of creating false assurance.
        assert!(!is_acceptable_token("short"));
        assert!(!is_acceptable_token(""));
        assert!(!is_acceptable_token("fifteen-chars!"));
        assert!(is_acceptable_token("sixteen-chars!!!"));
        assert!(is_acceptable_token("a-long-random-shared-secret"));
    }
}
