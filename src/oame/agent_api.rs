use reqwest::Client;
use url::Url;

use super::endpoints::agent_api_base_url;

/// Machine-facing OAME API client.
///
/// This client is intentionally separate from RustDesk native transport and
/// from the human/Portal API. Endpoint paths and signed request serialization
/// are added only when the corresponding Agent API contract is implemented.
#[derive(Clone, Debug)]
pub struct AgentApiClient {
    base_url: Url,
    http: Client,
}

impl AgentApiClient {
    /// Builds a client pinned to the canonical OAME Agent API origin.
    /// Enrollment input is never allowed to replace this origin.
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            base_url: agent_api_base_url()?,
            http: Client::new(),
        })
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Resolve a machine API path below the fixed `/v1/` root.
    ///
    /// Absolute URLs, scheme-relative URLs and parent traversal are rejected,
    /// so enrollment/bootstrap data cannot redirect the managed client to an
    /// arbitrary backend.
    pub fn endpoint(&self, relative_path: &str) -> Result<Url, String> {
        let path = relative_path.trim();
        if path.is_empty() {
            return Err("OAME Agent API endpoint path must not be empty".to_owned());
        }
        if path.starts_with('/')
            || path.starts_with("//")
            || path.contains("://")
            || path.split('/').any(|part| part == "..")
        {
            return Err("OAME Agent API endpoint must be a relative /v1/ path".to_owned());
        }

        self.base_url
            .join(path)
            .map_err(|e| format!("invalid OAME Agent API endpoint path: {e}"))
    }

    pub(crate) fn http(&self) -> &Client {
        &self.http
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_is_pinned_to_machine_api_origin() {
        let client = AgentApiClient::new().expect("Agent API client");
        assert_eq!(client.base_url().scheme(), "https");
        assert_eq!(client.base_url().host_str(), Some("agent.oame.net"));
        assert_eq!(client.base_url().path(), "/v1/");

        let endpoint = client.endpoint("heartbeat").expect("relative endpoint");
        assert_eq!(endpoint.as_str(), "https://agent.oame.net/v1/heartbeat");
    }

    #[test]
    fn arbitrary_backend_override_is_rejected() {
        let client = AgentApiClient::new().expect("Agent API client");
        for invalid in [
            "https://evil.example/v1/enroll",
            "//evil.example/v1/enroll",
            "/v1/enroll",
            "../api/remote/enroll",
        ] {
            assert!(client.endpoint(invalid).is_err(), "accepted: {invalid}");
        }
    }
}
