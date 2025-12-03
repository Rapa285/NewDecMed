use std::str::FromStr;
use std::sync::Arc;

use anyhow::{anyhow, Context};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::{Extension, Json};
use iota_types::base_types::IotaAddress;
use iota_types::crypto::{
    EncodeDecodeBase64, IotaKeyPair, IotaSignature, Signature, SignatureScheme,
};
use jwt_simple::claims::Claims;
use jwt_simple::prelude::{Duration, ECDSAP256KeyPairLike};
use redis::{Commands, SetExpiry, SetOptions};
use serde_json::json;
use shared_crypto::intent::{Intent, IntentMessage};
use umbral_pre::{reencrypt, Capsule, KeyFrag, PublicKey};

use crate::constants::{
    ADMINISTRATIVE_KEYS_READ_DUR, MEDICAL_KEYS_READ_DUR, MEDICAL_KEYS_UPDATE_DUR, NONCE_EXP_DUR,
};
use crate::current_fn;
use crate::proxy_error::{ProxyError, ResultExt};
use crate::types::{
    AccessKeys, AppState, AuthRole, ClientMedicalMetadata, CurrentUser,
    GenerateSignatureHandlerPayload, GetNonceHandlerPayload, HandlerCreateMedicalRecordPayload,
    HandlerGetAdministrativeDataQueryParams, HandlerGetMedicalRecordQueryParams,
    HandlerGetMedicalRecordUpdateQueryParams, HandlerUpdateMedicalRecordPayload, JwtClaims,
    MedicalMetadata, MoveHospitalPersonnelRole, PatientPrivateAdministrativeMetadata,
    ReencryptionPurposeType,
};
use crate::types::{GenerateJwtHandlerResponse, HandlerStoreKeysPayload};
use crate::utils::Utils;

pub struct Handlers {}

impl Handlers {
    pub async fn create_medical_record(
        State(state): State<Arc<AppState>>,
        Extension(current_user): Extension<CurrentUser>,
        Json(payload): Json<HandlerCreateMedicalRecordPayload>,
    ) -> Result<Response, ProxyError> {
        if current_user.role != AuthRole::MedicalPersonnel {
            return Err(ProxyError::Anyhow {
                source: anyhow!("Illegal action. Invalid role"),
                code: StatusCode::UNAUTHORIZED,
            });
        }

        if current_user.purpose != ReencryptionPurposeType::Update {
            return Err(ProxyError::Anyhow {
                source: anyhow!("Illegal action. Invalid purpose"),
                code: StatusCode::BAD_REQUEST,
            });
        }

        let (
            medical_metadata,
            hospital_personnel_iota_address,
            proxy_iota_address,
            proxy_iota_key_pair,
            patient_iota_address,
        ) = {
            let patient_iota_address = IotaAddress::from_str(&payload.patient_iota_address)
                .map_err(|_| anyhow!("Invalid patient IOTA address"))
                .code(StatusCode::BAD_REQUEST)?;
            let medical_metadata: ClientMedicalMetadata =
                Utils::serde_deserialize_from_base64(payload.medical_metadata)
                    .map_err(|_| anyhow!("Invalid medical metadata"))
                    .code(StatusCode::BAD_REQUEST)?;
            let hospital_personnel_iota_address = IotaAddress::from_str(&current_user.iota_address)
                .map_err(|_| anyhow!("Invalid hospital personnel IOTA address"))?;
            let proxy_iota_address =
                IotaAddress::from_str(&state.proxy_iota_address).context(current_fn!())?;
            let proxy_iota_key_pair = IotaKeyPair::decode(&state.proxy_iota_key_pair)
                .map_err(|e| anyhow!(e.to_string()))
                .context(current_fn!())?;

            (
                medical_metadata,
                hospital_personnel_iota_address,
                proxy_iota_address,
                proxy_iota_key_pair,
                patient_iota_address,
            )
        };

        let cid = Utils::add_and_pin_to_ipfs(medical_metadata.enc_data)
            .await
            .context(current_fn!())?;
        let created_at = Utils::sys_time_to_iso(std::time::SystemTime::now());

        let medical_metadata = MedicalMetadata {
            capsule: medical_metadata.capsule,
            cid,
            created_at,
            enc_key_and_nonce: medical_metadata.enc_key_and_nonce,
        };

        let _ = state
            .move_call
            .create_medical_record(
                &hospital_personnel_iota_address,
                Utils::serde_serialize_to_base64(&medical_metadata).context(current_fn!())?,
                &patient_iota_address,
                proxy_iota_address,
                proxy_iota_key_pair,
            )
            .await
            .context(current_fn!())?;

        Ok(Utils::build_success_response((), StatusCode::OK))
    }

    /**
     * This is just helper function
     */
    pub async fn generate_and_register_proxy_address(
        State(state): State<Arc<AppState>>,
    ) -> Result<Response, ProxyError> {
        let mnemonic = Utils::generate_mnemonic(12).context(current_fn!())?;

        let seed_words: Vec<&str> = mnemonic.words().collect();
        let seed_words = seed_words.join(" ");
        let seed = mnemonic.to_seed_normalized("proxy");

        let (proxy_iota_address, proxy_iota_keypair) =
            Utils::generate_iota_keys_ed(&seed).context(current_fn!())?;

        let _ = state
            .move_call
            .create_capability(
                &proxy_iota_address,
                IotaAddress::from_str(&state.global_admin_iota_address).context(current_fn!())?,
                IotaKeyPair::decode(&state.global_admin_iota_key_pair.clone())
                    .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?,
            )
            .await
            .context(current_fn!())?;

        let res_data = json!({
            "iota_address": proxy_iota_address.to_string(),
            "iota_key_pair": proxy_iota_keypair.encode().map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?,
            "seed_words": seed_words,
        });

        Ok(Utils::build_success_response(res_data, StatusCode::OK))
    }

    pub async fn generate_jwt_handler() -> Result<Response, ProxyError> {
        let (public_key, secret_key) = Utils::generate_jwt().context(current_fn!())?;

        let res_data = GenerateJwtHandlerResponse {
            public_key,
            secret_key,
        };

        Ok(Utils::build_success_response(res_data, StatusCode::OK))
    }

    /**
     * This is just helper function
     */
    pub async fn generate_signature(
        Json(payload): Json<GenerateSignatureHandlerPayload>,
    ) -> Result<Response, ProxyError> {
        let iota_keypair = IotaKeyPair::decode(&payload.iota_keypair)
            .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))
            .code(StatusCode::BAD_REQUEST)?;

        let intent_message = IntentMessage::new(Intent::personal_message(), payload.nonce);
        let signature = Signature::new_secure(&intent_message, &iota_keypair);
        let signature_string = signature.encode_base64();

        Ok(Utils::build_success_response(
            signature_string,
            StatusCode::OK,
        ))
    }

    pub async fn get_administrative_data(
        State(state): State<Arc<AppState>>,
        Extension(current_user): Extension<CurrentUser>,
        Query(query): Query<HandlerGetAdministrativeDataQueryParams>,
    ) -> Result<Response, ProxyError> {
        if current_user.role != AuthRole::AdministrativePersonnel
            && current_user.role != AuthRole::MedicalPersonnel
        {
            return Err(ProxyError::Anyhow {
                source: anyhow!("Illegal action. Invalid role"),
                code: StatusCode::UNAUTHORIZED,
            });
        }

        if current_user.purpose != ReencryptionPurposeType::Read
            && !(current_user.role == AuthRole::MedicalPersonnel
                && current_user.purpose == ReencryptionPurposeType::Update)
        {
            return Err(ProxyError::Anyhow {
                source: anyhow!("Illegal action. Invalid purpose"),
                code: StatusCode::BAD_REQUEST,
            });
        }

        let (hospital_personnel_iota_address, patient_iota_address, proxy_iota_address) = {
            let hospital_personnel_iota_address = IotaAddress::from_str(&current_user.iota_address)
                .map_err(|_| anyhow!("Invalid hospital personnel IOTA address"))
                .code(StatusCode::BAD_REQUEST)?;
            let patient_iota_address = IotaAddress::from_str(&query.patient_iota_address)
                .map_err(|_| anyhow!("Invalid patient IOTA address"))
                .code(StatusCode::UNAUTHORIZED)?;
            let proxy_iota_address =
                IotaAddress::from_str(&state.proxy_iota_address).context(current_fn!())?;

            (
                hospital_personnel_iota_address,
                patient_iota_address,
                proxy_iota_address,
            )
        };

        let (
            enc_patient_private_adm_data,
            access_keys,
            c_frag,
            enc_patient_private_adm_data_key_nonce,
            patient_private_adm_data_capsule,
        ) = {
            let mut conn = state.redis_pool.get().context(current_fn!())?;

            let access_keys: String = conn
                .get(format!(
                    "keys:{}@{}",
                    current_user.iota_address, query.patient_iota_address,
                ))
                .map_err(|_| anyhow!("Keys not found"))
                .code(StatusCode::BAD_REQUEST)?;

            let access_keys: AccessKeys =
                Utils::serde_deserialize_from_base64(access_keys).context(current_fn!())?;

            let patient_administrative_metadata = state
                .move_call
                .get_administrative_data(
                    &hospital_personnel_iota_address,
                    &patient_iota_address,
                    proxy_iota_address,
                )
                .await
                .context(current_fn!())?;

            let patient_private_adm_metadata: PatientPrivateAdministrativeMetadata =
                Utils::serde_deserialize_from_base64(
                    patient_administrative_metadata.private_metadata,
                )
                .context(current_fn!())?;

            let k_frag: KeyFrag = Utils::serde_deserialize_from_base64(access_keys.k_frag.clone())
                .context(current_fn!())?;
            let signer_pre_public_key: PublicKey =
                Utils::serde_deserialize_from_base64(access_keys.signer_pre_public_key.clone())
                    .context(current_fn!())?;
            let patient_pre_public_key: PublicKey =
                Utils::serde_deserialize_from_base64(access_keys.patient_pre_public_key.clone())
                    .context(current_fn!())?;
            let data_pre_public_key: PublicKey =
                Utils::serde_deserialize_from_base64(access_keys.data_pre_public_key.clone())
                    .context(current_fn!())?;
            let patient_private_adm_metadata_key_nonce_capsule: Capsule =
                Utils::serde_deserialize_from_base64(patient_private_adm_metadata.capsule.clone())
                    .context(current_fn!())?;

            let verified_kfrag = k_frag
                .verify(
                    &signer_pre_public_key,
                    Some(&patient_pre_public_key),
                    Some(&data_pre_public_key),
                )
                .map_err(|e| anyhow!(e.0.to_string()).context(current_fn!()))?;
            let verified_cfrag = reencrypt(
                &patient_private_adm_metadata_key_nonce_capsule,
                verified_kfrag,
            );
            let c_frag = verified_cfrag.unverify();

            (
                patient_private_adm_metadata.enc_data,
                access_keys,
                c_frag,
                patient_private_adm_metadata.enc_key_nonce,
                patient_private_adm_metadata.capsule,
            )
        };

        let res_data = json!({
            "c_frag": Utils::serde_serialize_to_base64(&c_frag).context(current_fn!())?,
            "data_pre_public_key": access_keys.data_pre_public_key,
            "data_pre_secret_key_seed_capsule": access_keys.data_pre_secret_key_seed_capsule,
            "enc_data_pre_secret_key_seed": access_keys.enc_data_pre_secret_key_seed,
            "enc_patient_private_adm_data": enc_patient_private_adm_data,
            "enc_patient_private_adm_data_key_nonce": enc_patient_private_adm_data_key_nonce,
            "patient_pre_public_key": access_keys.patient_pre_public_key,
            "patient_private_adm_data_capsule": patient_private_adm_data_capsule,
            "signer_pre_public_key": access_keys.signer_pre_public_key,
        });

        Ok(Utils::build_success_response(res_data, StatusCode::OK))
    }

    pub async fn get_medical_record(
        State(state): State<Arc<AppState>>,
        Extension(current_user): Extension<CurrentUser>,
        Query(query): Query<HandlerGetMedicalRecordQueryParams>,
    ) -> Result<Response, ProxyError> {
        if current_user.role != AuthRole::MedicalPersonnel {
            return Err(ProxyError::Anyhow {
                source: anyhow!("Illegal action. Invalid role"),
                code: StatusCode::UNAUTHORIZED,
            });
        }

        if current_user.purpose != ReencryptionPurposeType::Read {
            return Err(ProxyError::Anyhow {
                source: anyhow!("Illegal action. Invalid purpose"),
                code: StatusCode::BAD_REQUEST,
            });
        }

        let (hospital_personnel_iota_address, patient_iota_address, proxy_iota_address) = {
            let hospital_personnel_iota_address = IotaAddress::from_str(&current_user.iota_address)
                .map_err(|_| anyhow!("Invalid hospital personnel IOTA address"))
                .code(StatusCode::BAD_REQUEST)?;
            let patient_iota_address = IotaAddress::from_str(&query.patient_iota_address)
                .map_err(|_| anyhow!("Invalid patient IOTA address"))
                .code(StatusCode::UNAUTHORIZED)?;
            let proxy_iota_address =
                IotaAddress::from_str(&state.proxy_iota_address).context(current_fn!())?;

            (
                hospital_personnel_iota_address,
                patient_iota_address,
                proxy_iota_address,
            )
        };

        let (
            enc_administrative_data,
            enc_medical_data,
            access_keys,
            c_frag_administrative,
            c_frag_medical,
            current_index,
            prev_index,
            next_index,
            enc_medical_data_key_nonce,
            medical_data_capsule,
            medical_data_created_at,
            enc_administrative_data_key_nonce,
            administrative_data_capsule,
        ) = {
            let mut conn = state.redis_pool.get().context(current_fn!())?;

            let access_keys: String = conn
                .get(format!(
                    "keys:{}@{}",
                    current_user.iota_address, query.patient_iota_address,
                ))
                .map_err(|_| anyhow!("Keys not found"))
                .code(StatusCode::BAD_REQUEST)?;

            let access_keys: AccessKeys =
                Utils::serde_deserialize_from_base64(access_keys).context(current_fn!())?;

            let (medical_metadata, administrative_metadata, current_index, prev_index, next_index) =
                state
                    .move_call
                    .get_medical_record(
                        &hospital_personnel_iota_address,
                        query.index.unwrap_or(0),
                        &patient_iota_address,
                        proxy_iota_address,
                    )
                    .await
                    .context(current_fn!())?;

            let medical_metadata: MedicalMetadata =
                Utils::serde_deserialize_from_base64(medical_metadata.metadata)
                    .context(current_fn!())?;

            let patient_private_adm_metadata: PatientPrivateAdministrativeMetadata =
                Utils::serde_deserialize_from_base64(administrative_metadata.private_metadata)
                    .context(current_fn!())?;

            let enc_medical_data = Utils::get_data_ipfs(medical_metadata.cid)
                .await
                .context(current_fn!())?;

            let k_frag: KeyFrag = Utils::serde_deserialize_from_base64(access_keys.k_frag.clone())
                .context(current_fn!())?;
            let signer_pre_public_key: PublicKey =
                Utils::serde_deserialize_from_base64(access_keys.signer_pre_public_key.clone())
                    .context(current_fn!())?;
            let patient_pre_public_key: PublicKey =
                Utils::serde_deserialize_from_base64(access_keys.patient_pre_public_key.clone())
                    .context(current_fn!())?;
            let medical_record_pre_public_key: PublicKey =
                Utils::serde_deserialize_from_base64(access_keys.data_pre_public_key.clone())
                    .context(current_fn!())?;
            let medical_metadata_key_nonce_capsule: Capsule =
                Utils::serde_deserialize_from_base64(medical_metadata.capsule.clone())
                    .context(current_fn!())?;
            let patient_private_adm_metadata_key_nonce_capsule: Capsule =
                Utils::serde_deserialize_from_base64(patient_private_adm_metadata.capsule.clone())
                    .context(current_fn!())?;

            let verified_kfrag = k_frag
                .verify(
                    &signer_pre_public_key,
                    Some(&patient_pre_public_key),
                    Some(&medical_record_pre_public_key),
                )
                .map_err(|e| anyhow!(e.0.to_string()).context(current_fn!()))?;
            let verified_cfrag_medical =
                reencrypt(&medical_metadata_key_nonce_capsule, verified_kfrag.clone());
            let c_frag_medical = verified_cfrag_medical.unverify();

            let verified_cfrag_administrative = reencrypt(
                &patient_private_adm_metadata_key_nonce_capsule,
                verified_kfrag,
            );
            let c_frag_administrative = verified_cfrag_administrative.unverify();

            (
                patient_private_adm_metadata.enc_data,
                enc_medical_data,
                access_keys,
                c_frag_administrative,
                c_frag_medical,
                current_index,
                prev_index,
                next_index,
                medical_metadata.enc_key_and_nonce,
                medical_metadata.capsule,
                medical_metadata.created_at,
                patient_private_adm_metadata.enc_key_nonce,
                patient_private_adm_metadata.capsule,
            )
        };

        let res_data = json!({
            "administrative_data_capsule": administrative_data_capsule,
            "c_frag_administrative": Utils::serde_serialize_to_base64(&c_frag_administrative).context(current_fn!())?,
            "c_frag_medical": Utils::serde_serialize_to_base64(&c_frag_medical).context(current_fn!())?,
            "current_index": current_index,
            "data_pre_public_key": access_keys.data_pre_public_key,
            "data_pre_secret_key_seed_capsule": access_keys.data_pre_secret_key_seed_capsule,
            "enc_administrative_data": enc_administrative_data,
            "enc_administrative_data_key_nonce": enc_administrative_data_key_nonce,
            "enc_data_pre_secret_key_seed": access_keys.enc_data_pre_secret_key_seed,
            "enc_medical_data": enc_medical_data,
            "enc_medical_data_key_nonce": enc_medical_data_key_nonce,
            "medical_data_capsule": medical_data_capsule,
            "medical_data_created_at": medical_data_created_at,
            "next_index": next_index,
            "patient_pre_public_key": access_keys.patient_pre_public_key,
            "prev_index": prev_index,
            "signer_pre_public_key": access_keys.signer_pre_public_key,
        });

        Ok(Utils::build_success_response(res_data, StatusCode::OK))
    }

    pub async fn get_medical_record_update(
        State(state): State<Arc<AppState>>,
        Extension(current_user): Extension<CurrentUser>,
        Query(query): Query<HandlerGetMedicalRecordUpdateQueryParams>,
    ) -> Result<Response, ProxyError> {
        if current_user.role != AuthRole::MedicalPersonnel {
            return Err(ProxyError::Anyhow {
                source: anyhow!("Illegal action. Invalid role"),
                code: StatusCode::UNAUTHORIZED,
            });
        }

        if current_user.purpose != ReencryptionPurposeType::Update {
            return Err(ProxyError::Anyhow {
                source: anyhow!("Illegal action. Invalid purpose"),
                code: StatusCode::BAD_REQUEST,
            });
        }

        let (hospital_personnel_iota_address, patient_iota_address, proxy_iota_address) = {
            let hospital_personnel_iota_address = IotaAddress::from_str(&current_user.iota_address)
                .map_err(|_| anyhow!("Invalid hospital personnel IOTA address"))
                .code(StatusCode::BAD_REQUEST)?;
            let patient_iota_address = IotaAddress::from_str(&query.patient_iota_address)
                .map_err(|_| anyhow!("Invalid patient IOTA address"))
                .code(StatusCode::UNAUTHORIZED)?;
            let proxy_iota_address =
                IotaAddress::from_str(&state.proxy_iota_address).context(current_fn!())?;

            (
                hospital_personnel_iota_address,
                patient_iota_address,
                proxy_iota_address,
            )
        };

        let (
            enc_administrative_data,
            enc_medical_data,
            access_keys,
            c_frag_administrative,
            c_frag_medical,
            enc_medical_data_key_nonce,
            medical_data_capsule,
            medical_data_created_at,
            enc_administrative_data_key_nonce,
            administrative_data_capsule,
        ) = {
            let mut conn = state.redis_pool.get().context(current_fn!())?;

            let access_keys: String = conn
                .get(format!(
                    "keys:{}@{}",
                    current_user.iota_address, query.patient_iota_address,
                ))
                .map_err(|_| anyhow!("Keys not found"))
                .code(StatusCode::BAD_REQUEST)?;

            let access_keys: AccessKeys =
                Utils::serde_deserialize_from_base64(access_keys).context(current_fn!())?;

            let (medical_metadata, administrative_metadata) = state
                .move_call
                .get_medical_record_update(
                    &hospital_personnel_iota_address,
                    query.index.unwrap_or(0),
                    &patient_iota_address,
                    proxy_iota_address,
                )
                .await
                .context(current_fn!())?;

            let medical_metadata: MedicalMetadata =
                Utils::serde_deserialize_from_base64(medical_metadata.metadata)
                    .context(current_fn!())?;

            let patient_private_adm_metadata: PatientPrivateAdministrativeMetadata =
                Utils::serde_deserialize_from_base64(administrative_metadata.private_metadata)
                    .context(current_fn!())?;

            let enc_medical_data = Utils::get_data_ipfs(medical_metadata.cid)
                .await
                .context(current_fn!())?;

            let k_frag: KeyFrag = Utils::serde_deserialize_from_base64(access_keys.k_frag.clone())
                .context(current_fn!())?;
            let signer_pre_public_key: PublicKey =
                Utils::serde_deserialize_from_base64(access_keys.signer_pre_public_key.clone())
                    .context(current_fn!())?;
            let patient_pre_public_key: PublicKey =
                Utils::serde_deserialize_from_base64(access_keys.patient_pre_public_key.clone())
                    .context(current_fn!())?;
            let data_pre_public_key: PublicKey =
                Utils::serde_deserialize_from_base64(access_keys.data_pre_public_key.clone())
                    .context(current_fn!())?;
            let medical_metadata_key_nonce_capsule: Capsule =
                Utils::serde_deserialize_from_base64(medical_metadata.capsule.clone())
                    .context(current_fn!())?;
            let patient_private_adm_metadata_key_nonce_capsule: Capsule =
                Utils::serde_deserialize_from_base64(patient_private_adm_metadata.capsule.clone())
                    .context(current_fn!())?;

            let verified_kfrag = k_frag
                .verify(
                    &signer_pre_public_key,
                    Some(&patient_pre_public_key),
                    Some(&data_pre_public_key),
                )
                .map_err(|e| anyhow!(e.0.to_string()).context(current_fn!()))?;
            let verified_cfrag_medical =
                reencrypt(&medical_metadata_key_nonce_capsule, verified_kfrag.clone());
            let c_frag_medical = verified_cfrag_medical.unverify();

            let verified_cfrag_administrative = reencrypt(
                &patient_private_adm_metadata_key_nonce_capsule,
                verified_kfrag,
            );
            let c_frag_administrative = verified_cfrag_administrative.unverify();

            (
                patient_private_adm_metadata.enc_data,
                enc_medical_data,
                access_keys,
                c_frag_administrative,
                c_frag_medical,
                medical_metadata.enc_key_and_nonce,
                medical_metadata.capsule,
                medical_metadata.created_at,
                patient_private_adm_metadata.enc_key_nonce,
                patient_private_adm_metadata.capsule,
            )
        };

        let res_data = json!({
            "administrative_data_capsule": administrative_data_capsule,
            "c_frag_administrative": Utils::serde_serialize_to_base64(&c_frag_administrative).context(current_fn!())?,
            "c_frag_medical": Utils::serde_serialize_to_base64(&c_frag_medical).context(current_fn!())?,
            "data_pre_public_key": access_keys.data_pre_public_key,
            "data_pre_secret_key_seed_capsule": access_keys.data_pre_secret_key_seed_capsule,
            "enc_administrative_data": enc_administrative_data,
            "enc_administrative_data_key_nonce": enc_administrative_data_key_nonce,
            "enc_data_pre_secret_key_seed": access_keys.enc_data_pre_secret_key_seed,
            "enc_medical_data": enc_medical_data,
            "enc_medical_data_key_nonce": enc_medical_data_key_nonce,
            "medical_data_capsule": medical_data_capsule,
            "medical_data_created_at": medical_data_created_at,
            "patient_pre_public_key": access_keys.patient_pre_public_key,
            "signer_pre_public_key": access_keys.signer_pre_public_key,
        });

        Ok(Utils::build_success_response(res_data, StatusCode::OK))
    }

    pub async fn get_nonce_handler(
        State(state): State<Arc<AppState>>,
        Json(payload): Json<GetNonceHandlerPayload>,
    ) -> Result<Response, ProxyError> {
        let patient_iota_address = IotaAddress::from_str(&payload.iota_address)
            .map_err(|_| anyhow!("Invalid patient IOTA address"))
            .code(StatusCode::BAD_REQUEST)?;
        let proxy_iota_address =
            IotaAddress::from_str(&state.proxy_iota_address).context(current_fn!())?;

        let _ = state
            .move_call
            .is_patient_registered(&patient_iota_address, proxy_iota_address)
            .await
            .context(current_fn!())?;

        let nonce = Utils::generate_64_bytes_seed();
        let nonce = hex::encode(&nonce);

        let mut conn = state.redis_pool.get().context(current_fn!())?;

        let _: () = conn
            .set_options(
                format!("nonce:{}", patient_iota_address.to_string()),
                nonce.clone(),
                SetOptions::default().with_expiration(SetExpiry::EX(NONCE_EXP_DUR)),
            )
            .context(current_fn!())?;

        Ok(Utils::build_success_response(nonce, StatusCode::OK))
    }

    pub async fn store_keys(
        State(state): State<Arc<AppState>>,
        Json(payload): Json<HandlerStoreKeysPayload>,
    ) -> Result<Response, ProxyError> {
        let patient_iota_address = IotaAddress::from_str(&payload.patient_iota_address)
            .map_err(|_| anyhow!("Invalid patient IOTA address"))
            .code(StatusCode::BAD_REQUEST)?;
        let hospital_personnel_iota_address =
            IotaAddress::from_str(&payload.hospital_personnel_iota_address)
                .map_err(|_| anyhow!("Invalid hospital personnel IOTA address"))
                .code(StatusCode::BAD_REQUEST)?;
        let signature = Utils::construct_signature_from_str(&payload.signature)
            .map_err(|_| anyhow!("Invalid signature"))
            .code(StatusCode::BAD_REQUEST)?;
        let proxy_iota_address =
            IotaAddress::from_str(&state.proxy_iota_address).context(current_fn!())?;

        let mut conn = state.redis_pool.get().context(current_fn!())?;

        let nonce: String = conn
            .get(format!("nonce:{}", patient_iota_address.to_string()))
            .map_err(|_| anyhow!("Nonce not found"))
            .code(StatusCode::BAD_REQUEST)?;

        let intent_message = IntentMessage::new(Intent::personal_message(), nonce);

        let _ = signature
            .verify_secure(
                &intent_message,
                patient_iota_address,
                SignatureScheme::ED25519,
            )
            .map_err(|_| anyhow!("Failed to verify signature"))
            .code(StatusCode::UNAUTHORIZED)?;

        let _: () = conn
            .del(patient_iota_address.to_string())
            .map_err(|_| anyhow!("Nonce expired"))
            .code(StatusCode::UNAUTHORIZED)?;

        // Get the role of hospital personnel
        let role = state
            .move_call
            .get_hospital_personnel_role(&hospital_personnel_iota_address, proxy_iota_address)
            .await
            .context(current_fn!())?;

        let (hospital_personnel_role, read_keys_duration, update_keys_duration): (
            AuthRole,
            u64,
            Option<u64>,
        ) = match role {
            MoveHospitalPersonnelRole::AdministrativePersonnel => (
                AuthRole::AdministrativePersonnel,
                ADMINISTRATIVE_KEYS_READ_DUR,
                None,
            ),
            MoveHospitalPersonnelRole::MedicalPersonnel => (
                AuthRole::MedicalPersonnel,
                MEDICAL_KEYS_READ_DUR,
                Some(MEDICAL_KEYS_UPDATE_DUR),
            ),
            _ => {
                return Err(ProxyError::Anyhow {
                    source: anyhow!("Invalid personnel account"),
                    code: StatusCode::BAD_REQUEST,
                })
            }
        };

        // Create access token for hospital personnel
        let es256_keypair = Utils::construct_es256_key_pair_from_pem(&state.jwt_ecdsa_key_pair)
            .context(current_fn!())?;

        let read_claims = JwtClaims {
            role: hospital_personnel_role.clone(),
            purpose: ReencryptionPurposeType::Read,
        };
        let read_claims =
            Claims::with_custom_claims(read_claims, Duration::from_secs(read_keys_duration))
                .with_subject(hospital_personnel_iota_address);

        let hospital_personnel_access_token_update = if update_keys_duration.is_some() {
            let update_claims = JwtClaims {
                role: hospital_personnel_role,
                purpose: ReencryptionPurposeType::Update,
            };
            let update_claims = Claims::with_custom_claims(
                update_claims,
                Duration::from_secs(update_keys_duration.unwrap()),
            )
            .with_subject(hospital_personnel_iota_address);
            Some(es256_keypair.sign(update_claims).context(current_fn!())?)
        } else {
            None
        };

        let hospital_personnel_access_token_read =
            es256_keypair.sign(read_claims).context(current_fn!())?;

        let access_keys = AccessKeys {
            enc_data_pre_secret_key_seed: payload.enc_data_pre_secret_key_seed,
            k_frag: payload.k_frag,
            data_pre_public_key: payload.data_pre_public_key,
            data_pre_secret_key_seed_capsule: payload.data_pre_secret_key_seed_capsule,
            patient_pre_public_key: payload.patient_pre_public_key,
            signer_pre_public_key: payload.signer_pre_public_key,
        };

        let _: () = conn
            .set_options(
                format!(
                    "keys:{}@{}",
                    hospital_personnel_iota_address.to_string(),
                    patient_iota_address.to_string()
                ),
                Utils::serde_serialize_to_base64(&access_keys).context(current_fn!())?,
                SetOptions::default().with_expiration(SetExpiry::EX(
                    update_keys_duration.unwrap_or(read_keys_duration),
                )),
            )
            .context(current_fn!())?;

        let res_data = json!({
            "access_token_read": hospital_personnel_access_token_read,
            "access_token_update": hospital_personnel_access_token_update,
        });

        Ok(Utils::build_success_response(res_data, StatusCode::OK))
    }

    pub async fn update_medical_record(
        State(state): State<Arc<AppState>>,
        Extension(current_user): Extension<CurrentUser>,
        Json(payload): Json<HandlerUpdateMedicalRecordPayload>,
    ) -> Result<Response, ProxyError> {
        if current_user.role != AuthRole::MedicalPersonnel {
            return Err(ProxyError::Anyhow {
                source: anyhow!("Illegal action. Invalid role"),
                code: StatusCode::UNAUTHORIZED,
            });
        }

        if current_user.purpose != ReencryptionPurposeType::Update {
            return Err(ProxyError::Anyhow {
                source: anyhow!("Illegal action. Invalid purpose"),
                code: StatusCode::BAD_REQUEST,
            });
        }

        let (
            medical_metadata,
            hospital_personnel_iota_address,
            proxy_iota_address,
            proxy_iota_key_pair,
            patient_iota_address,
        ) = {
            let patient_iota_address = IotaAddress::from_str(&payload.patient_iota_address)
                .map_err(|_| anyhow!("Invalid patient IOTA address"))
                .code(StatusCode::BAD_REQUEST)?;
            let medical_metadata: ClientMedicalMetadata =
                Utils::serde_deserialize_from_base64(payload.medical_metadata)
                    .map_err(|_| anyhow!("Invalid medical metadata"))
                    .code(StatusCode::BAD_REQUEST)?;
            let hospital_personnel_iota_address = IotaAddress::from_str(&current_user.iota_address)
                .map_err(|_| anyhow!("Invalid hospital personnel IOTA address"))?;
            let proxy_iota_address =
                IotaAddress::from_str(&state.proxy_iota_address).context(current_fn!())?;
            let proxy_iota_key_pair = IotaKeyPair::decode(&state.proxy_iota_key_pair)
                .map_err(|e| anyhow!(e.to_string()))
                .context(current_fn!())?;

            (
                medical_metadata,
                hospital_personnel_iota_address,
                proxy_iota_address,
                proxy_iota_key_pair,
                patient_iota_address,
            )
        };

        let cid = Utils::add_and_pin_to_ipfs(medical_metadata.enc_data)
            .await
            .context(current_fn!())?;
        let created_at = Utils::sys_time_to_iso(std::time::SystemTime::now());

        let medical_metadata = MedicalMetadata {
            capsule: medical_metadata.capsule,
            cid,
            created_at,
            enc_key_and_nonce: medical_metadata.enc_key_and_nonce,
        };

        let _ = state
            .move_call
            .update_medical_record(
                &hospital_personnel_iota_address,
                Utils::serde_serialize_to_base64(&medical_metadata).context(current_fn!())?,
                &patient_iota_address,
                proxy_iota_address,
                proxy_iota_key_pair,
            )
            .await
            .context(current_fn!())?;

        Ok(Utils::build_success_response((), StatusCode::OK))
    }
}
