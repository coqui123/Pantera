use axum::{
    response::{IntoResponse, Json, Response},
    extract::{State, Extension},
    http::StatusCode,
};
use axum_extra::extract::{CookieJar, cookie::{Cookie, SameSite}};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use uuid::Uuid;
use time;

use crate::{
    errors::AppError,
    handlers::AppState,
    auth::{TezosAdminSession, AdminAuth},
};

// --- New Crypto & Encoding Crates ---
use bs58;
use blake2::{Blake2b, Digest as CryptoDigest};
use generic_array::GenericArray;
use signature::Verifier;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use base64::Engine as Base64Engine;

// Specific crypto crates
use ed25519_dalek::{VerifyingKey as Ed25519VerifyingKey, Signature as Ed25519Signature};
use k256::ecdsa::{VerifyingKey as Secp256k1VerifyingKey, Signature as Secp256k1Signature};
use p256::ecdsa::{VerifyingKey as P256VerifyingKey, Signature as P256Signature};

// --- Tezos Constants ---
mod tezos_consts {
    // Prefixes for Base58Check decoded data (raw bytes before encoding)
    pub const ED25519_PUBLIC_KEY_PREFIX: [u8; 4] = [13, 15, 37, 217]; // edpk
    pub const SECP256K1_PUBLIC_KEY_PREFIX: [u8; 4] = [3, 254, 226, 86];  // sppk
    pub const P256_PUBLIC_KEY_PREFIX: [u8; 4] = [3, 178, 139, 127];    // p2pk

    pub const ED25519_SIGNATURE_PREFIX: [u8; 5] = [9, 245, 205, 134, 18]; // edsig
    pub const SECP256K1_SIGNATURE_PREFIX: [u8; 5] = [13, 115, 101, 19, 63]; // spsig1
    pub const P256_SIGNATURE_PREFIX: [u8; 4] = [54, 240, 44, 52];

    // Raw key lengths (after prefix)
    pub const ED25519_PK_RAW_LEN: usize = 32;
    pub const SECP256K1_PK_COMPRESSED_RAW_LEN: usize = 33;
    pub const P256_PK_COMPRESSED_RAW_LEN: usize = 33;

    pub const ED25519_SIG_RAW_LEN: usize = 64;
    pub const SECP256K1_SIG_RAW_LEN: usize = 64; // (r,s) components, 32 bytes each
    pub const P256_SIG_RAW_LEN: usize = 64;

    // Address Prefixes (used for encoding the 20-byte PKH)
    pub const TZ1_ADDRESS_PREFIX: [u8; 3] = [6, 161, 159]; // tz1
    pub const TZ2_ADDRESS_PREFIX: [u8; 3] = [6, 161, 161]; // tz2
    pub const TZ3_ADDRESS_PREFIX: [u8; 3] = [6, 161, 164]; // tz3

    pub const MICHELINE_PACKED_PREFIX: u8 = 0x05;
    pub const MICHELINE_STRING_TAG: u8 = 0x01;
}

use tezos_consts::*;
#[allow(unused_imports)] // sha2::Digest is used for .finalize() trait method
use sha2::Digest as Sha2Digest;

type HmacSha256 = Hmac<Sha256>;

/// Sign session data with HMAC-SHA256 and return base64-encoded signed cookie value.
/// Format: base64(json_data).base64(hmac_signature)
fn sign_session_cookie(session_json: &str, hmac_key: &[u8; 32]) -> String {
    let encoded_data = base64::engine::general_purpose::STANDARD.encode(session_json);
    let mut mac = HmacSha256::new_from_slice(hmac_key)
        .expect("HMAC can take key of any size");
    mac.update(encoded_data.as_bytes());
    let signature = mac.finalize();
    let signature_bytes = signature.into_bytes();
    let encoded_sig = base64::engine::general_purpose::STANDARD.encode(signature_bytes);
    format!("{}.{}", encoded_data, encoded_sig)
}

/// Verify and decode a signed session cookie.
/// Returns None if cookie is invalid or tampered with.
pub fn verify_session_cookie(cookie_value: &str, hmac_key: &[u8; 32]) -> Option<TezosAdminSession> {
    // Split on the last dot (data.signature format)
    match cookie_value.rfind('.') {
        Some(dot_pos) => {
            let encoded_data = &cookie_value[..dot_pos];
            let encoded_sig = &cookie_value[dot_pos + 1..];
            
            // Decode signature
            let expected_sig_bytes = base64::engine::general_purpose::STANDARD.decode(encoded_sig).ok()?;
            
            // Verify HMAC
            let mut mac = HmacSha256::new_from_slice(hmac_key)
                .expect("HMAC can take key of any size");
            mac.update(encoded_data.as_bytes());
            mac.verify_slice(&expected_sig_bytes).ok()?;
            
            // Decode session data
            let session_bytes = base64::engine::general_purpose::STANDARD.decode(encoded_data).ok()?;
            let session_str = String::from_utf8(session_bytes).ok()?;
            serde_json::from_str::<TezosAdminSession>(&session_str).ok()
        }
        None => {
            // Legacy format: try to decode as plain base64 (backward compatibility)
            // This allows existing cookies to still work during transition
            if let Ok(session_bytes) = base64::engine::general_purpose::STANDARD.decode(cookie_value) {
                if let Ok(session_str) = String::from_utf8(session_bytes) {
                    if let Ok(session) = serde_json::from_str::<TezosAdminSession>(&session_str) {
                        return Some(session);
                    }
                }
            }
            None
        }
    }
}

// Helper for Base58Check decoding with prefix validation
fn b58_decode_with_prefix_check(encoded: &str, expected_prefix: &[u8]) -> Result<Vec<u8>, AppError> {
    let decoded_with_checksum = bs58::decode(encoded)
        .into_vec()
        .map_err(|e| AppError::ValidationError(format!("Base58 decode error: {}", e)))?;
    
    // Tezos Base58Check uses a 4-byte checksum.
    if decoded_with_checksum.len() < expected_prefix.len() + 4 {
        return Err(AppError::ValidationError("Invalid Base58Check data: too short".to_string()));
    }

    let prefix_len = expected_prefix.len();
    let payload_and_prefix_len = decoded_with_checksum.len() - 4;
    
    let received_prefix = &decoded_with_checksum[0..prefix_len];
    if received_prefix != expected_prefix {
        return Err(AppError::ValidationError(format!("Base58Check prefix mismatch. Expected: {:?}, Got: {:?}", expected_prefix, received_prefix)));
    }

    let payload = decoded_with_checksum[prefix_len..payload_and_prefix_len].to_vec();

    // Checksum verification (double SHA256)
    let mut hasher1 = sha2::Sha256::new();
    hasher1.update(&decoded_with_checksum[0..payload_and_prefix_len]);
    let hash1 = hasher1.finalize();

    let mut hasher2 = sha2::Sha256::new();
    hasher2.update(hash1);
    let hash2 = hasher2.finalize();

    let expected_checksum = &hash2[0..4];
    let received_checksum = &decoded_with_checksum[payload_and_prefix_len..];

    if expected_checksum != received_checksum {
        return Err(AppError::ValidationError("Base58Check checksum mismatch".to_string()));
    }
    
    Ok(payload)
}

// Helper for Base58Check encoding with prefix
fn b58_encode_with_prefix(payload: &[u8], prefix: &[u8]) -> String {
    let mut data_to_encode = Vec::with_capacity(prefix.len() + payload.len() + 4);
    data_to_encode.extend_from_slice(prefix);
    data_to_encode.extend_from_slice(payload);

    let mut hasher1 = sha2::Sha256::new();
    hasher1.update(&data_to_encode);
    let hash1 = hasher1.finalize();

    let mut hasher2 = sha2::Sha256::new();
    hasher2.update(hash1);
    let hash2 = hasher2.finalize();

    data_to_encode.extend_from_slice(&hash2[0..4]); // Add checksum
    bs58::encode(data_to_encode).into_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengePayload {
    challenge: String,
    packed_bytes_hex: String,
}

#[derive(Debug, Deserialize)]
pub struct TezosLoginPayload {
    pkh: String,
    public_key: String,
    signature: String,
    challenge: String,
}

#[derive(Serialize)]
pub struct ChallengeResponse {
    challenge: String,
    packed_bytes_hex: String,
}

#[derive(Debug, Clone)]
pub enum TezosCryptoPublicKey {
    Ed25519(GenericArray<u8, generic_array::typenum::U32>),
    Secp256k1(k256::EncodedPoint),
    P256(p256::EncodedPoint),
}

impl TezosCryptoPublicKey {
    pub fn from_base58check(key_str: &str) -> Result<Self, AppError> {
        if key_str.starts_with("edpk") {
            let raw_pk = b58_decode_with_prefix_check(key_str, &ED25519_PUBLIC_KEY_PREFIX)?;
            if raw_pk.len() != ED25519_PK_RAW_LEN {
                return Err(AppError::ValidationError("Invalid Ed25519 public key length".to_string()));
            }
            Ok(TezosCryptoPublicKey::Ed25519(*GenericArray::from_slice(&raw_pk)))
        } else if key_str.starts_with("sppk") {
            let raw_pk_with_prefix_payload = b58_decode_with_prefix_check(key_str, &SECP256K1_PUBLIC_KEY_PREFIX)?;
            if raw_pk_with_prefix_payload.len() != SECP256K1_PK_COMPRESSED_RAW_LEN {
                return Err(AppError::ValidationError("Invalid Secp256k1 public key length".to_string()));
            }
            let point = k256::EncodedPoint::from_bytes(&raw_pk_with_prefix_payload)
                .map_err(|e| AppError::ValidationError(format!("Secp256k1 PK decode error: {}", e)))?;
            Ok(TezosCryptoPublicKey::Secp256k1(point))
        } else if key_str.starts_with("p2pk") {
            let raw_pk_with_prefix_payload = b58_decode_with_prefix_check(key_str, &P256_PUBLIC_KEY_PREFIX)?;
            if raw_pk_with_prefix_payload.len() != P256_PK_COMPRESSED_RAW_LEN {
                return Err(AppError::ValidationError("Invalid P256 public key length".to_string()));
            }
            let point = p256::EncodedPoint::from_bytes(&raw_pk_with_prefix_payload)
                .map_err(|e| AppError::ValidationError(format!("P256 PK decode error: {}", e)))?;
            Ok(TezosCryptoPublicKey::P256(point))
        } else {
            Err(AppError::ValidationError("Unsupported public key prefix".to_string()))
        }
    }

    pub fn verify_signature(&self, signature_b58: &str, message_hash: &[u8]) -> Result<bool, AppError> {
        match self {
            TezosCryptoPublicKey::Ed25519(pk_bytes_arr) => {
                let sig_bytes = b58_decode_with_prefix_check(signature_b58, &ED25519_SIGNATURE_PREFIX)?;
                if sig_bytes.len() != ED25519_SIG_RAW_LEN {
                    return Err(AppError::ValidationError("Invalid Ed25519 signature length".to_string()));
                }
                let signature_array: [u8; ED25519_SIG_RAW_LEN] = sig_bytes.try_into()
                    .map_err(|_| AppError::Internal("Ed25519 signature conversion to array failed".to_string()))?;
                let signature = Ed25519Signature::from_bytes(&signature_array);
                
                let pk_array: [u8; ED25519_PK_RAW_LEN] = pk_bytes_arr.as_slice().try_into()
                    .map_err(|_| AppError::Internal("Ed25519 PK conversion to array failed".to_string()))?;
                let verifying_key = Ed25519VerifyingKey::from_bytes(&pk_array)
                    .map_err(|e| AppError::Internal(format!("Ed25519 VK build error: {}", e)))?;
                Ok(verifying_key.verify(message_hash, &signature).is_ok())
            },
            TezosCryptoPublicKey::Secp256k1(encoded_point) => {
                let sig_bytes = b58_decode_with_prefix_check(signature_b58, &SECP256K1_SIGNATURE_PREFIX)?;
                if sig_bytes.len() != SECP256K1_SIG_RAW_LEN {
                     return Err(AppError::ValidationError("Invalid Secp256k1 signature length".to_string()));
                }
                let signature = Secp256k1Signature::from_slice(&sig_bytes)
                    .map_err(|e| AppError::ValidationError(format!("Secp256k1 sig decode error: {}",e)))?;
                let verifying_key = Secp256k1VerifyingKey::from_encoded_point(encoded_point)
                    .map_err(|e| AppError::Internal(format!("Secp256k1 VK build error: {}",e)))?;
                Ok(verifying_key.verify(message_hash, &signature).is_ok())
            },
            TezosCryptoPublicKey::P256(encoded_point) => {
                let sig_bytes = b58_decode_with_prefix_check(signature_b58, &P256_SIGNATURE_PREFIX)?;
                if sig_bytes.len() != P256_SIG_RAW_LEN {
                    return Err(AppError::ValidationError("Invalid P256 signature length".to_string()));
                }
                let signature = P256Signature::from_slice(&sig_bytes)
                    .map_err(|e| AppError::ValidationError(format!("P256 sig decode error: {}",e)))?;
                let verifying_key = P256VerifyingKey::from_encoded_point(encoded_point)
                    .map_err(|e| AppError::Internal(format!("P256 VK build error: {}",e)))?;
                Ok(verifying_key.verify(message_hash, &signature).is_ok())
            },
        }
    }

    pub fn public_key_hash_b58check(&self) -> Result<String, AppError> {
        let raw_pk_bytes_for_hash: Vec<u8> = match self {
            TezosCryptoPublicKey::Ed25519(pk_bytes) => pk_bytes.to_vec(),
            TezosCryptoPublicKey::Secp256k1(point) => point.as_bytes().to_vec(),
            TezosCryptoPublicKey::P256(point) => point.as_bytes().to_vec(),
        };

        let mut hasher = Blake2b::<generic_array::typenum::U20>::new();
        hasher.update(&raw_pk_bytes_for_hash);
        let pkh_raw = hasher.finalize();

        let address_prefix = match self {
            TezosCryptoPublicKey::Ed25519(_) => &TZ1_ADDRESS_PREFIX,           
            TezosCryptoPublicKey::Secp256k1(_) => &TZ2_ADDRESS_PREFIX,
            TezosCryptoPublicKey::P256(_) => &TZ3_ADDRESS_PREFIX,
        };
        
        Ok(b58_encode_with_prefix(pkh_raw.as_slice(), address_prefix))
    }
}

/// Generates a new challenge for Tezos wallet signing.
pub async fn get_tezos_challenge(
    State(_app_state): State<AppState>,
) -> Result<Json<ChallengeResponse>, AppError> {
    let challenge = format!("Sign this message to log in as admin: {}", Uuid::new_v4());
    let packed_bytes = pack_micheline_string(&challenge)?;
    let packed_bytes_hex = hex::encode(&packed_bytes);
    tracing::info!("Generated Tezos login challenge: {}", challenge);
    tracing::debug!("Packed bytes (hex): {}", packed_bytes_hex);
    Ok(Json(ChallengeResponse { 
        challenge,
        packed_bytes_hex,
    }))
}

/// Verifies the signed Tezos challenge and logs the user in.
pub async fn tezos_login(
    State(app_state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<TezosLoginPayload>,
) -> Result<(CookieJar, Response), AppError>
{
    tracing::info!("Attempting Tezos login for PKH: {}", payload.pkh);

    let public_key = TezosCryptoPublicKey::from_base58check(&payload.public_key)
        .map_err(|e| AppError::ValidationError(format!("Invalid public key format or value: {}", e)))?;

    // Verify that the provided public key hash (pkh) matches the one derived from the public_key
    let derived_pkh = public_key.public_key_hash_b58check()?;
    if derived_pkh != payload.pkh {
        tracing::warn!("Mismatch between provided PKH ({}) and derived PKH ({}).", payload.pkh, derived_pkh);
        return Err(AppError::ValidationError("Public key hash does not match the provided public key.".to_string()));
    }

    // Pack the challenge string according to Tezos specification for signing
    // Format: 0x05 (prefix for packed data) || 0x01 (string tag) || len (4 bytes BE) || string_bytes
    let packed_challenge_bytes = pack_micheline_string(&payload.challenge)?;
    
    // Hash the packed challenge bytes using BLAKE2b (32-byte hash for message signing)
    let mut hasher = Blake2b::<generic_array::typenum::U32>::new(); // 32-byte output for message hash
    hasher.update(&packed_challenge_bytes);
    let message_hash_to_verify = hasher.finalize();

    // Verify the signature
    match public_key.verify_signature(&payload.signature, message_hash_to_verify.as_slice()) {
        Ok(true) => tracing::info!("Tezos signature VERIFIED for PKH: {}", payload.pkh),
        Ok(false) => {
            tracing::warn!("Tezos signature verification FAILED for PKH: {}", payload.pkh);
            return Err(AppError::Unauthorized);
        }
        Err(e) => {
            tracing::error!("Error during signature verification for PKH {}: {:#}", payload.pkh, e);
            // Distinguish between validation errors (bad signature format) and internal errors
            if matches!(e, AppError::ValidationError(_)) {
                return Err(e);
            } else {
                return Err(AppError::Internal(format!("Signature verification processing error: {:?}", e)));
            }
        }
    }

    if !app_state.config.auth.admin_tezos_addresses.contains(&payload.pkh) { // Check against pkh (address)
        tracing::warn!("PKH {} is not an admin address.", payload.pkh);
        return Err(AppError::Unauthorized);
    }

    let session_data = TezosAdminSession { address: payload.pkh.clone() }; // Store PKH in session
    let session_json = serde_json::to_string(&session_data)
        .map_err(|e| AppError::Internal(format!("Serialize session error: {}", e)))?;
    
    // Sign the session cookie with HMAC-SHA256
    let signed_cookie_value = sign_session_cookie(&session_json, &app_state.config.auth.cookie_hmac_key);

    let cookie_std_duration = std::time::Duration::from_secs(3600 * 24 * 7); // 7 days
    let cookie_time_duration: time::Duration = cookie_std_duration.try_into()
        .map_err(|_| AppError::Internal("Failed to convert duration for cookie.".to_string()))?;

    let mut cookie = Cookie::new("tezos_admin_session", signed_cookie_value);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_secure(true); // Ensure this is true for production
    cookie.set_same_site(SameSite::Lax);
    cookie.set_max_age(cookie_time_duration);

    tracing::info!("Setting admin session cookie for PKH: {}", payload.pkh);
    let updated_jar = jar.add(cookie);
    let response_body = (StatusCode::OK, Json("Login successful")).into_response();
    tracing::info!("Tezos login completed successfully for PKH: {}", payload.pkh);
    Ok((updated_jar, response_body))
}

/// Logs the admin out by clearing the session cookie.
pub async fn logout(
    State(_app_state): State<AppState>,
    jar: CookieJar 
) -> Result<(CookieJar, Response), AppError> { 
    tracing::info!("Logging out Tezos admin.");
    let mut cookie = Cookie::new("tezos_admin_session", "");
    cookie.set_path("/");
    let updated_jar = jar.remove(cookie);
    let response_body = axum::response::Redirect::to("/login").into_response();
    Ok((updated_jar, response_body))
}

/// Public endpoint to check authentication status (for frontend)
pub async fn auth_status(
    State(app_state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut status = serde_json::json!({
        "authenticated": false,
        "dev_mode": app_state.config.auth.dev_mode,
    });

    // Check for valid session cookie
    if let Some(cookie) = jar.get("tezos_admin_session") {
        if let Some(session) = verify_session_cookie(cookie.value(), &app_state.config.auth.cookie_hmac_key) {
            // Verify the address is still in admin list
            if app_state.config.auth.admin_tezos_addresses.contains(&session.address) {
                status["authenticated"] = serde_json::Value::Bool(true);
                status["is_admin_address"] = serde_json::Value::Bool(true);
            }
        }
    }

    // If dev mode is enabled, user is considered authenticated
    if app_state.config.auth.dev_mode {
        status["authenticated"] = serde_json::Value::Bool(true);
    }

    Ok(Json(status))
}

/// Debug endpoint to check authentication status (dev mode only)
#[cfg(debug_assertions)]
pub async fn debug_auth_status(
    State(app_state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut debug_info = serde_json::json!({
        "dev_mode": app_state.config.auth.dev_mode,
        "has_cookie": false,
        "cookie_valid": false,
        "session_address": null,
        "is_admin_address": false,
        "admin_addresses": app_state.config.auth.admin_tezos_addresses.iter().collect::<Vec<_>>()
    });

    if let Some(cookie) = jar.get("tezos_admin_session") {
        debug_info["has_cookie"] = serde_json::Value::Bool(true);
        
        if let Some(session) = verify_session_cookie(cookie.value(), &app_state.config.auth.cookie_hmac_key) {
            debug_info["cookie_valid"] = serde_json::Value::Bool(true);
            debug_info["session_address"] = serde_json::Value::String(session.address.clone());
            debug_info["is_admin_address"] = serde_json::Value::Bool(
                app_state.config.auth.admin_tezos_addresses.contains(&session.address)
            );
        }
    }

    tracing::info!("Debug auth status: {}", debug_info);
    Ok(Json(debug_info))
}

// Function to pack a Micheline string (0x05 || 0x01 || len (4 bytes BE) || string_data)
fn pack_micheline_string(data: &str) -> Result<Vec<u8>, AppError> {
    let s_bytes = data.as_bytes();
    let s_len = s_bytes.len() as u32;

    let mut packed = Vec::new();
    packed.push(MICHELINE_PACKED_PREFIX); // 0x05 - packed data prefix
    packed.push(MICHELINE_STRING_TAG);    // 0x01 - string tag
    packed.extend_from_slice(&s_len.to_be_bytes()); // 4-byte big-endian length
    packed.extend_from_slice(s_bytes);    // string data
    Ok(packed)
}