use std::sync::Arc;
use std::time::Instant;

use anyhow::{anyhow, Context};
use axum::{
    extract::{Request, State},
    http::{self, StatusCode},
    middleware::Next,
    response::Response,
};
use jwt_simple::prelude::{ECDSAP256PublicKeyLike, ES256PublicKey};

use crate::{
    current_fn,
    proxy_error::{ProxyError, ResultExt},
    types::{AppState, CurrentUser, JwtClaims},
    utils::Utils,
    ats::{ATSClient,AuditEvent, AuditEventDetails, AuditOutcome},
};

use uuid::Uuid;


pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, ProxyError> {
    // 1. Inisialisasi awal variabel audit event
    let mut requester_id = "unknown".to_string();
    let mut capability_id = "bearer_token".to_string();
    let mut validation_result = false;
    let mut rejection_reason: Option<String> = None;
    let mut outcome = AuditOutcome::Failure;

    // Helper closure untuk mengirim event agar kode tidak berulang
    let send_audit = |req_id: &str, cap_id: &str, res: bool, rej: Option<String>, out: AuditOutcome| {
        let event = Event {
            source_component: "proxy-reencryption".to_string(),
            actor: req_id.to_string(),
            target_object: cap_id.to_string(),
            outcome: out,
            action_type: "CAPABILITY_VALIDATION".to_string(),
            details: AuditEventDetails::CapabilityValidation {
                capability_id: cap_id.to_string(),
                requester_id: req_id.to_string(),
                validation_result: res,
                rejection_reason: rej,
            },
        };
        ATSClient::send_event_from_state(&state, event,"auth_middleware");
    };

    // 2. Ekstrak Header
    let authorization_header = request
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let bearer_token = match Utils::decode_authorization_header(authorization_header) {
        Ok(token) => token,
        Err(err) => {
            rejection_reason = Some("Missing or malformed Authorization header".to_string());
            send_audit(&requester_id, &capability_id, validation_result, rejection_reason, outcome);
            return Err(err);
        }
    };

    capability_id = bearer_token.clone();
    Utils::debug_print(current_fn!(), &bearer_token);

    // 3. Verifikasi JWT
    let es256_public_key =
        ES256PublicKey::from_pem(&state.jwt_ecdsa_pub_key).context(current_fn!())?;

    let claims = match es256_public_key.verify_token::<JwtClaims>(&bearer_token, None) {
        Ok(claims) => claims,
        Err(_) => {
            rejection_reason = Some("Access token already expired or invalid".to_string());
            send_audit(&requester_id, &capability_id, validation_result, rejection_reason, outcome);
            return Err(ProxyError::Anyhow {
                source: anyhow!("Access token already expired or invalid"),
                code: StatusCode::UNAUTHORIZED,
            });
        }
    };

    // 4. Jika Berhasil (Update state menjadi Success)
    requester_id = claims.subject.unwrap_or_else(|| "unknown".to_string());
    validation_result = true;
    outcome = AuditOutcome::Success;

    let current_user = CurrentUser {
        iota_address: requester_id.clone(),
        purpose: claims.custom.purpose,
        role: claims.custom.role,
    };
    request.extensions_mut().insert(current_user);

    // Kirim event audit untuk kondisi SUKSES
    send_audit(&requester_id, &capability_id, validation_result, None, outcome);

    let response = next.run(request).await;
    Ok(response)
}

pub async fn audit_logger_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    // println!("[Audit Middleware] Memproses request ke '{}'", request.uri().path());
    // 1. Ekstrak informasi dari request sebelum dikonsumsi oleh `next.run`
    let endpoint_called = request.uri().path().to_string();
    
    // Ambil X-Request-ID jika ada di header, jika tidak buat UUID v4 baru
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|val| val.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Ambil header X-Caller-Component / User-Agent jika ada
    let caller_component = request
        .headers()
        .get("x-caller-component")
        .and_then(|val| val.to_str().ok())
        .unwrap_or("unknown-client")
        .to_string();

    // Deteksi skema enkripsi kanal (misal TLS/HTTPS)
    let channel_encryption = request
        .uri()
        .scheme_str()
        .map(|s| s.to_uppercase())
        .unwrap_or_else(|| "TLS".to_string());

    let start = Instant::now();

    // 2. Lanjutkan request ke handler berikutnya
    let response = next.run(request).await;

    // 3. Hitung status hasil request
    let outcome = if response.status().is_success() {
        AuditOutcome::Success
    } else {
        AuditOutcome::Failure
    };

    // 4. Susun AuditEvent dengan varian PREServiceRequest (EV7)
    let audit_event = AuditEvent {
        source_component: "proxy-reencryption".to_string(),
        actor: "system/middleware".to_string(), // Atau ekstrak dari token/header auth jika ada
        target_object: endpoint_called.clone(),
        outcome,
        action_type: "PRE_SERVICE_REQUEST".to_string(),
        details: AuditEventDetails::PREServiceRequest {
            endpoint_called,
            request_id,
            caller_component,
            channel_encryption,
        },
    };

    // 5. Kirim event audit secara asinkron
    ATSClient::send_event_from_state(&state, audit_event, "pre_service_request");

    response
}
