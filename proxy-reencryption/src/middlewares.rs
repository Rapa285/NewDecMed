use std::sync::Arc;

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
