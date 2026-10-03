use hbb_common::{
    base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _},
    sodiumoxide::randombytes::randombytes,
};
use reqwest::Method;
use sha2::{Digest, Sha256};
use std::{fmt::Write as _, time::{SystemTime, UNIX_EPOCH}};

use super::agent_identity::AgentIdentity;

pub const PROTOCOL_VERSION: &str = "1";
pub const HEADER_INSTALLATION_ID: &str = "X-OAME-Installation-ID";
pub const HEADER_TIMESTAMP: &str = "X-OAME-Timestamp";
pub const HEADER_NONCE: &str = "X-OAME-Nonce";
pub const HEADER_SIGNATURE: &str = "X-OAME-Signature";
pub const HEADER_PROTOCOL_VERSION: &str = "X-OAME-Protocol-Version";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedRequestHeaders {
    pub installation_id: String,
    pub timestamp: String,
    pub nonce: String,
    pub signature: String,
    pub protocol_version: String,
}

pub fn body_sha256(body: &[u8]) -> String {
    let digest = Sha256::digest(body);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

pub fn canonical_request(
    method: &Method,
    path: &str,
    body: &[u8],
    timestamp: u64,
    nonce: &str,
    installation_id: &str,
    protocol_version: &str,
) -> Result<String, String> {
    validate_path(path)?;
    if nonce.is_empty() {
        return Err("OAME request nonce must not be empty".to_owned());
    }
    if installation_id.is_empty() {
        return Err("OAME installation_id must not be empty".to_owned());
    }
    if protocol_version.is_empty() {
        return Err("OAME protocol version must not be empty".to_owned());
    }

    Ok(format!(
        "OAME1\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        method.as_str().to_ascii_uppercase(),
        path,
        body_sha256(body),
        timestamp,
        nonce,
        installation_id,
        protocol_version
    ))
}

pub fn sign_request(
    identity: &AgentIdentity,
    method: &Method,
    path: &str,
    body: &[u8],
) -> Result<SignedRequestHeaders, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("system clock error while signing OAME request: {e}"))?
        .as_secs();
    let nonce = generate_nonce();
    sign_request_with_values(identity, method, path, body, timestamp, &nonce, PROTOCOL_VERSION)
}

fn sign_request_with_values(
    identity: &AgentIdentity,
    method: &Method,
    path: &str,
    body: &[u8],
    timestamp: u64,
    nonce: &str,
    protocol_version: &str,
) -> Result<SignedRequestHeaders, String> {
    let canonical = canonical_request(
        method,
        path,
        body,
        timestamp,
        nonce,
        identity.installation_id(),
        protocol_version,
    )?;
    let signature = identity.sign(canonical.as_bytes())?;

    Ok(SignedRequestHeaders {
        installation_id: identity.installation_id().to_owned(),
        timestamp: timestamp.to_string(),
        nonce: nonce.to_owned(),
        signature: URL_SAFE_NO_PAD.encode(signature),
        protocol_version: protocol_version.to_owned(),
    })
}

fn generate_nonce() -> String {
    URL_SAFE_NO_PAD.encode(randombytes(24))
}

fn validate_path(path: &str) -> Result<(), String> {
    if !path.starts_with("/v1/") {
        return Err("OAME signed request path must be under /v1/".to_owned());
    }
    if path.contains('?') || path.contains('#') {
        return Err("OAME Agent API v1 signed endpoints do not use query or fragment components".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_body_sha256_is_lowercase_hex() {
        assert_eq!(
            body_sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn canonical_request_matches_adr_126_format() {
        let canonical = canonical_request(
            &Method::POST,
            "/v1/heartbeat",
            br#"{\"ok\":true}"#,
            1_700_000_000,
            "nonce123",
            "installation-123",
            "1",
        )
        .expect("canonical request");

        let expected_hash = body_sha256(br#"{\"ok\":true}"#);
        assert_eq!(
            canonical,
            format!(
                "OAME1\nPOST\n/v1/heartbeat\n{}\n1700000000\nnonce123\ninstallation-123\n1",
                expected_hash
            )
        );
        assert!(!canonical.ends_with('\n'));
    }

    #[test]
    fn signed_v1_path_rejects_query_parameters() {
        assert!(canonical_request(
            &Method::GET,
            "/v1/config?site=x",
            b"",
            1_700_000_000,
            "nonce123",
            "installation-123",
            "1",
        )
        .is_err());
    }
}
