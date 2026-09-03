use std::sync::Arc;

use anyhow::{anyhow, Context};
use axum::{
    extract::{Request, State},
    http::{self, StatusCode, Request},
    middleware::Next,
    response::Response,
};
use jwt_simple::prelude::{ECDSAP256PublicKeyLike, ES256PublicKey};

use crate::{
    current_fn,
    proxy_error::{ProxyError, ResultExt},
    types::{AppState, CurrentUser, JwtClaims},
    utils::Utils,
};

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, ProxyError> {
    let authorization_header = request
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let bearer_token = Utils::decode_authorization_header(authorization_header)?;
    Utils::debug_print(current_fn!(), &bearer_token);

    let es256_public_key =
        ES256PublicKey::from_pem(&state.jwt_ecdsa_pub_key).context(current_fn!())?;

    let claims = es256_public_key
        .verify_token::<JwtClaims>(&bearer_token, None)
        .map_err(|_| anyhow!("Access token already expired or invalid"))
        .code(StatusCode::UNAUTHORIZED)?;

    let current_user = CurrentUser {
        iota_address: claims.subject.unwrap(),
        purpose: claims.custom.purpose,
        role: claims.custom.role,
    };
    request.extensions_mut().insert(current_user);

    let response = next.run(request).await;

    Ok(response)
}

pub async fn audit_logger_middleware<B>(
    State(state): State<Arc<AppState>>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    let method = request.method().to_string();
    let uri = request.uri().to_string();
    let start = Instant::now();

    // Lanjutkan request ke handler berikutnya
    let response = next.run(request).await;

    let latency = start.elapsed().as_millis() as u64;
    let status_code = response.status().as_u16();

    // Buat data AuditEvent Anda
    let audit_event = AuditEvent {
        method,
        endpoint: uri,
        status_code,
        latency_ms: latency,
        // Tambahkan field lain sesuai struct AuditEvent Anda
    };

    // Kirim menggunakan ATSClient di background (karena send_event sudah pakai tokio::spawn)
    state.ats_client.send_event(audit_event, "API_REQUEST_LOG");

    response
}
