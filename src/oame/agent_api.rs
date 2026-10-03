use reqwest::{header::AUTHORIZATION, Client, Method, RequestBuilder};
use url::Url;

use super::{
    agent_identity::AgentIdentity,
    endpoints::{agent_api_base_url, agent_api_url, ENROLL_PATH},
    wire::{
        sign_request, HEADER_INSTALLATION_ID, HEADER_NONCE, HEADER_PROTOCOL_VERSION,
        HEADER_SIGNATURE, HEADER_TIMESTAMP,
    },
};

#[derive(Clone, Debug)]
pub struct AgentApiClient {
    base_url: Url,
    http: Client,
}

impl AgentApiClient {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            base_url: agent_api_base_url()?,
            http: Client::new(),
        })
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub fn signed_request(
        &self,
        identity: &AgentIdentity,
        method: Method,
        path: &str,
        body: Vec<u8>,
    ) -> Result<RequestBuilder, String> {
        let url = agent_api_url(path)?;
        let signed = sign_request(identity, &method, path, &body)?;

        Ok(self
            .http
            .request(method, url)
            .header(HEADER_INSTALLATION_ID, signed.installation_id)
            .header(HEADER_TIMESTAMP, signed.timestamp)
            .header(HEADER_NONCE, signed.nonce)
            .header(HEADER_SIGNATURE, signed.signature)
            .header(HEADER_PROTOCOL_VERSION, signed.protocol_version)
            .body(body))
    }

    pub fn enrollment_request(
        &self,
        identity: &AgentIdentity,
        enrollment_token: &str,
        body: Vec<u8>,
    ) -> Result<RequestBuilder, String> {
        if enrollment_token.is_empty() {
            return Err("OAME enrollment token must not be empty".to_owned());
        }

        Ok(self
            .signed_request(identity, Method::POST, ENROLL_PATH, body)?
            .header(AUTHORIZATION, format!("Bearer {enrollment_token}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oame::endpoints::{CONFIG_PATH, HEARTBEAT_PATH, IDENTITY_PATH};

    #[test]
    fn client_is_pinned_to_machine_api_origin() {
        let client = AgentApiClient::new().expect("Agent API client");
        assert_eq!(client.base_url().scheme(), "https");
        assert_eq!(client.base_url().host_str(), Some("agent.oame.net"));
        assert_eq!(client.base_url().path(), "/v1/");
    }

    #[test]
    fn only_approved_agent_v1_paths_resolve() {
        for path in [ENROLL_PATH, HEARTBEAT_PATH, CONFIG_PATH, IDENTITY_PATH] {
            let url = agent_api_url(path).expect("approved endpoint");
            assert_eq!(url.host_str(), Some("agent.oame.net"));
        }
        assert!(agent_api_url("/v1/other").is_err());
    }
}
