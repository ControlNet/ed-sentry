use chrono::{Duration, TimeZone, Utc};
use ed_sentry::app::{
    ActiveTunnel, TunnelAuth, TunnelAuthError, TunnelAuthIssue, TunnelAuthValidation,
    TunnelAuthValidationResult, TunnelSessionId, TunnelSigningSecret,
};

const CONFIG_PASSWORD: &str = "fixture-password";
const WRONG_PASSWORD: &str = "wrong-password";

fn fixture_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 6, 28, 12, 0, 0)
        .single()
        .expect("fixture timestamp is valid")
}

fn active_tunnel() -> ActiveTunnel {
    ActiveTunnel::new(
        "fixture.trycloudflare.com",
        TunnelSessionId::new("session-fixture-auth"),
    )
}

#[test]
#[ignore]
fn tunnel_auth_manual_qa_exercises_issue_validate_and_rejections() -> Result<(), TunnelAuthError> {
    // Given: the non-production manual QA helper uses fixture-only credentials and tunnel IDs.
    let auth = TunnelAuth::from_secret(TunnelSigningSecret::from_bytes([8; 32]));
    let active = active_tunnel();
    let now = fixture_time();

    // When: the happy path, wrong password path, and expired-token path are exercised.
    let token = auth
        .issue_token(TunnelAuthIssue {
            config_password: CONFIG_PASSWORD,
            password_attempt: CONFIG_PASSWORD,
            active_tunnel: &active,
            now,
        })?
        .expect("matching password issues a token");
    let authorized = matches!(
        auth.validate_authorization(TunnelAuthValidation {
            config_password: CONFIG_PASSWORD,
            authorization_header: Some(&format!("Bearer {}", token.as_str())),
            active_tunnel: &active,
            now,
        })?,
        TunnelAuthValidationResult::Authorized(_)
    );
    let wrong_password_token = auth.issue_token(TunnelAuthIssue {
        config_password: CONFIG_PASSWORD,
        password_attempt: WRONG_PASSWORD,
        active_tunnel: &active,
        now,
    })?;
    let expired_token = auth
        .issue_token(TunnelAuthIssue {
            config_password: CONFIG_PASSWORD,
            password_attempt: CONFIG_PASSWORD,
            active_tunnel: &active,
            now: now - Duration::hours(13),
        })?
        .expect("matching password issues a token");
    let expired_result = auth.validate_authorization(TunnelAuthValidation {
        config_password: CONFIG_PASSWORD,
        authorization_header: Some(&format!("Bearer {}", expired_token.as_str())),
        active_tunnel: &active,
        now,
    });

    // Then: stdout records outcomes without printing passwords, tokens, or signing secrets.
    println!(
        "manual_qa issued_token=true authorized={authorized} wrong_password_token={} expired_result={expired_result:?}",
        wrong_password_token.is_some()
    );
    assert!(authorized);
    assert!(wrong_password_token.is_none());
    assert_eq!(expired_result, Err(TunnelAuthError::TokenExpired));
    Ok(())
}
