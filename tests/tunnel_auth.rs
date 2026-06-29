use chrono::{Duration, TimeZone, Utc};
use ed_sentry::app::{
    ActiveTunnel, TunnelAuth, TunnelAuthError, TunnelAuthIssue, TunnelAuthPurpose,
    TunnelAuthValidation, TunnelAuthValidationResult, TunnelSessionId, TunnelSigningSecret,
};

const CONFIG_PASSWORD: &str = "fixture-password";
const WRONG_PASSWORD: &str = "wrong-password";

fn fixture_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 6, 28, 12, 0, 0)
        .single()
        .unwrap()
}

fn active_tunnel() -> ActiveTunnel {
    ActiveTunnel::new(
        "fixture.trycloudflare.com",
        TunnelSessionId::new("session-fixture-auth"),
    )
}

fn fixture_auth(seed: u8) -> TunnelAuth {
    TunnelAuth::from_secret(TunnelSigningSecret::from_bytes([seed; 32]))
}

#[test]
fn tunnel_auth_issues_token_when_password_matches() -> Result<(), TunnelAuthError> {
    // Given: configured tunnel auth and an active public tunnel session.
    let auth = fixture_auth(1);
    let active = active_tunnel();
    let now = fixture_time();

    // When: the configured password is supplied.
    let token = auth.issue_token(TunnelAuthIssue {
        config_password: CONFIG_PASSWORD,
        password_attempt: CONFIG_PASSWORD,
        active_tunnel: &active,
        now,
    })?;

    // Then: a Bearer token validates to the expected tunnel claims.
    let token = token.expect("matching password issues a token");
    let result = auth.validate_authorization(TunnelAuthValidation {
        config_password: CONFIG_PASSWORD,
        authorization_header: Some(&format!("Bearer {}", token.as_str())),
        active_tunnel: &active,
        now,
    })?;
    let TunnelAuthValidationResult::Authorized(claims) = result else {
        panic!("expected authorized tunnel claims");
    };
    assert_eq!(claims.subject, "tunnel-user");
    assert_eq!(claims.purpose, TunnelAuthPurpose::WebApi);
    assert_eq!(claims.issued_at, now.timestamp());
    assert_eq!(claims.expires_at, (now + Duration::hours(12)).timestamp());
    assert_eq!(claims.active_tunnel_host, "fixture.trycloudflare.com");
    assert_eq!(claims.active_tunnel_session_id, "session-fixture-auth");
    Ok(())
}

#[test]
fn tunnel_auth_issues_no_token_when_password_is_wrong() -> Result<(), TunnelAuthError> {
    // Given: configured tunnel auth and an active public tunnel session.
    let auth = fixture_auth(2);
    let active = active_tunnel();

    // When: the password attempt does not match the config password.
    let token = auth.issue_token(TunnelAuthIssue {
        config_password: CONFIG_PASSWORD,
        password_attempt: WRONG_PASSWORD,
        active_tunnel: &active,
        now: fixture_time(),
    })?;

    // Then: no token is minted for a later Authorization header.
    assert!(token.is_none());
    Ok(())
}

#[test]
fn tunnel_auth_empty_config_password_bypasses_protection() -> Result<(), TunnelAuthError> {
    // Given: the documented accepted-risk mode has no configured tunnel password.
    let auth = fixture_auth(3);
    let active = active_tunnel();

    // When: a protected Web API later asks the primitive to authorize without a header.
    let result = auth.validate_authorization(TunnelAuthValidation {
        config_password: "",
        authorization_header: None,
        active_tunnel: &active,
        now: fixture_time(),
    })?;

    // Then: protection is bypassed instead of requiring a token.
    assert_eq!(result, TunnelAuthValidationResult::Bypassed);
    Ok(())
}

#[test]
fn tunnel_auth_rejects_missing_and_non_bearer_authorization() {
    // Given: configured tunnel auth and an active public tunnel session.
    let auth = fixture_auth(4);
    let active = active_tunnel();

    for authorization_header in [None, Some("Basic credential"), Some("Bearer ")] {
        // When: the Authorization header is missing or not exactly Bearer token shaped.
        let result = auth.validate_authorization(TunnelAuthValidation {
            config_password: CONFIG_PASSWORD,
            authorization_header,
            active_tunnel: &active,
            now: fixture_time(),
        });

        // Then: the protected API seam rejects the request.
        assert_eq!(result, Err(TunnelAuthError::AuthorizationRejected));
    }
}

#[test]
fn tunnel_auth_rejects_malformed_wrong_signature_and_expired_tokens() -> Result<(), TunnelAuthError>
{
    // Given: two per-run authenticators and one valid token from the first run.
    let first_run = fixture_auth(5);
    let restarted_app = fixture_auth(6);
    let active = active_tunnel();
    let now = fixture_time();
    let token = first_run
        .issue_token(TunnelAuthIssue {
            config_password: CONFIG_PASSWORD,
            password_attempt: CONFIG_PASSWORD,
            active_tunnel: &active,
            now,
        })?
        .expect("matching password issues a token");

    let expired_token = first_run
        .issue_token(TunnelAuthIssue {
            config_password: CONFIG_PASSWORD,
            password_attempt: CONFIG_PASSWORD,
            active_tunnel: &active,
            now: now - Duration::hours(13),
        })?
        .expect("matching password issues a token");

    // When/Then: malformed input, restart-invalidated signature, and expired token reject.
    assert_eq!(
        first_run.validate_authorization(TunnelAuthValidation {
            config_password: CONFIG_PASSWORD,
            authorization_header: Some("Bearer not-a-jwt"),
            active_tunnel: &active,
            now,
        }),
        Err(TunnelAuthError::TokenRejected)
    );
    assert_eq!(
        restarted_app.validate_authorization(TunnelAuthValidation {
            config_password: CONFIG_PASSWORD,
            authorization_header: Some(&format!("Bearer {}", token.as_str())),
            active_tunnel: &active,
            now,
        }),
        Err(TunnelAuthError::TokenRejected)
    );
    assert_eq!(
        first_run.validate_authorization(TunnelAuthValidation {
            config_password: CONFIG_PASSWORD,
            authorization_header: Some(&format!("Bearer {}", expired_token.as_str())),
            active_tunnel: &active,
            now,
        }),
        Err(TunnelAuthError::TokenExpired)
    );
    Ok(())
}

#[test]
fn tunnel_auth_rejects_stale_host_and_session_tokens() -> Result<(), TunnelAuthError> {
    // Given: one token bound to the active host and session id at issue time.
    let auth = fixture_auth(7);
    let active = active_tunnel();
    let now = fixture_time();
    let token = auth
        .issue_token(TunnelAuthIssue {
            config_password: CONFIG_PASSWORD,
            password_attempt: CONFIG_PASSWORD,
            active_tunnel: &active,
            now,
        })?
        .expect("matching password issues a token");
    let bearer = format!("Bearer {}", token.as_str());

    // When/Then: changing the active tunnel host invalidates the old token.
    let stale_host = ActiveTunnel::new(
        "rotated.trycloudflare.com",
        TunnelSessionId::new("session-fixture-auth"),
    );
    assert_eq!(
        auth.validate_authorization(TunnelAuthValidation {
            config_password: CONFIG_PASSWORD,
            authorization_header: Some(&bearer),
            active_tunnel: &stale_host,
            now,
        }),
        Err(TunnelAuthError::StaleHost)
    );

    // When/Then: changing the active tunnel session id also invalidates the old token.
    let stale_session = ActiveTunnel::new(
        "fixture.trycloudflare.com",
        TunnelSessionId::new("session-rotated"),
    );
    assert_eq!(
        auth.validate_authorization(TunnelAuthValidation {
            config_password: CONFIG_PASSWORD,
            authorization_header: Some(&bearer),
            active_tunnel: &stale_session,
            now,
        }),
        Err(TunnelAuthError::StaleSession)
    );
    Ok(())
}

#[test]
fn tunnel_auth_generated_per_run_secret_invalidates_tokens_after_restart(
) -> Result<(), TunnelAuthError> {
    // Given: two authenticators created through the production per-run secret path.
    let first_run = TunnelAuth::new_per_run()?;
    let restarted_app = TunnelAuth::new_per_run()?;
    let active = active_tunnel();
    let now = fixture_time();
    let token = first_run
        .issue_token(TunnelAuthIssue {
            config_password: CONFIG_PASSWORD,
            password_attempt: CONFIG_PASSWORD,
            active_tunnel: &active,
            now,
        })?
        .expect("matching password issues a token");

    // When: the token is checked by a new app run.
    let result = restarted_app.validate_authorization(TunnelAuthValidation {
        config_password: CONFIG_PASSWORD,
        authorization_header: Some(&format!("Bearer {}", token.as_str())),
        active_tunnel: &active,
        now,
    });

    // Then: the old run's token is not accepted by the new run's signing secret.
    assert_eq!(result, Err(TunnelAuthError::TokenRejected));
    Ok(())
}
