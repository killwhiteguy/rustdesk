use serde_derive::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MachineErrorEnvelope {
    pub error: MachineError,
    pub message: String,
    pub server_time: u64,
    pub request_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MachineError {
    pub code: String,
}

impl MachineError {
    pub fn is_known_v1_code(&self) -> bool {
        matches!(
            self.code.as_str(),
            "malformed_request"
                | "unsupported_protocol"
                | "invalid_enrollment_token"
                | "enrollment_expired"
                | "enrollment_revoked"
                | "enrollment_exhausted"
                | "installation_mismatch"
                | "unknown_identity"
                | "identity_revoked"
                | "reenrollment_required"
                | "suspected_clone"
                | "invalid_signature"
                | "replay_detected"
                | "clock_skew"
                | "forbidden"
                | "rate_limited"
                | "server_error"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stable_v1_error_envelope() {
        let parsed: MachineErrorEnvelope = serde_json::from_str(
            r#"{
                "error":{"code":"clock_skew"},
                "message":"request timestamp outside accepted window",
                "server_time":1700000000,
                "request_id":"req-123"
            }"#,
        )
        .expect("machine error envelope");

        assert_eq!(parsed.error.code, "clock_skew");
        assert!(parsed.error.is_known_v1_code());
        assert_eq!(parsed.server_time, 1_700_000_000);
    }

    #[test]
    fn unknown_future_error_code_remains_parseable() {
        let parsed: MachineErrorEnvelope = serde_json::from_str(
            r#"{
                "error":{"code":"future_code"},
                "message":"future error",
                "server_time":1700000000,
                "request_id":"req-456"
            }"#,
        )
        .expect("forward-compatible machine error envelope");

        assert!(!parsed.error.is_known_v1_code());
    }
}
