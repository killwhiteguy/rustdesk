use hbb_common::base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde_derive::{Deserialize, Serialize};

use super::{agent_identity::AgentIdentity, wire::PROTOCOL_VERSION};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnrollmentOs {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnrollmentRequest {
    pub installation_id: String,
    pub public_key: String,
    pub remote_id: String,
    pub agent_version: String,
    pub protocol_version: u32,
    pub hostname: String,
    pub os: EnrollmentOs,
    pub architecture: String,
    pub capabilities: Vec<String>,
}

impl EnrollmentRequest {
    pub fn new(
        identity: &AgentIdentity,
        remote_id: String,
        agent_version: String,
        hostname: String,
        os: EnrollmentOs,
        architecture: String,
        capabilities: Vec<String>,
    ) -> Result<Self, String> {
        for (name, value) in [
            ("remote_id", remote_id.as_str()),
            ("agent_version", agent_version.as_str()),
            ("hostname", hostname.as_str()),
            ("os.name", os.name.as_str()),
            ("architecture", architecture.as_str()),
        ] {
            if value.is_empty() {
                return Err(format!("OAME enrollment {name} must not be empty"));
            }
        }
        if os.version.as_deref() == Some("") {
            return Err("OAME enrollment os.version must be omitted when unavailable".to_owned());
        }
        if capabilities.iter().any(|capability| capability.is_empty()) {
            return Err("OAME enrollment capabilities must contain only non-empty strings".to_owned());
        }

        Ok(Self {
            installation_id: identity.installation_id().to_owned(),
            public_key: URL_SAFE_NO_PAD.encode(identity.public_key()),
            remote_id,
            agent_version,
            protocol_version: PROTOCOL_VERSION
                .parse()
                .map_err(|e| format!("invalid OAME protocol version constant: {e}"))?,
            hostname,
            os,
            architecture,
            capabilities,
        })
    }

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self)
            .map_err(|e| format!("failed to serialize OAME enrollment request: {e}"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnrollmentResponse {
    pub device_id: String,
    pub remote_support_identity_id: String,
    pub enrollment_state: String,
    pub protocol_version: u32,
    pub server_time: u64,
}

impl EnrollmentResponse {
    pub fn validate_v1(&self) -> Result<(), String> {
        if self.device_id.is_empty() || self.remote_support_identity_id.is_empty() {
            return Err("OAME enrollment response contains empty opaque references".to_owned());
        }
        if !matches!(self.enrollment_state.as_str(), "pending" | "managed") {
            return Err("OAME enrollment response contains invalid enrollment_state".to_owned());
        }
        if self.protocol_version != 1 {
            return Err("OAME enrollment response protocol_version is not v1".to_owned());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_matches_v1_success_shape() {
        let response: EnrollmentResponse = serde_json::from_str(
            r#"{"device_id":"dev_1","remote_support_identity_id":"rsi_1","enrollment_state":"managed","protocol_version":1,"server_time":1760000000}"#,
        )
        .expect("enrollment response");
        response.validate_v1().expect("valid v1 response");
    }

    #[test]
    fn response_rejects_unknown_fields() {
        let parsed = serde_json::from_str::<EnrollmentResponse>(
            r#"{"device_id":"dev_1","remote_support_identity_id":"rsi_1","enrollment_state":"managed","protocol_version":1,"server_time":1760000000,"extra":true}"#,
        );
        assert!(parsed.is_err());
    }
}
