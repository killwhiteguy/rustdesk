use url::Url;

/// Canonical managed OAME Agent API origin from Architecture Blueprint v1.7.
pub const AGENT_API_ORIGIN: &str = "https://agent.oame.net";

/// Versioned machine API root. Human/Portal API is intentionally not used here.
pub const AGENT_API_BASE_PATH: &str = "/v1/";

/// Canonical RustDesk native transport host for all new OAME Remote Support builds.
pub const REMOTE_TRANSPORT_HOST: &str = "remote.oame.net";

/// Temporary compatibility endpoint for controlled migration only.
/// New builds and managed configuration must not select this as their default.
pub const LEGACY_REMOTE_TRANSPORT_HOST: &str = "sup.oame.net";

pub fn agent_api_base_url() -> Result<Url, String> {
    Url::parse(&format!("{AGENT_API_ORIGIN}{AGENT_API_BASE_PATH}"))
        .map_err(|e| format!("invalid built-in OAME Agent API URL: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_endpoints_match_oame_topology() {
        let url = agent_api_base_url().expect("valid built-in Agent API URL");
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("agent.oame.net"));
        assert_eq!(url.path(), "/v1/");
        assert_eq!(REMOTE_TRANSPORT_HOST, "remote.oame.net");
        assert_ne!(REMOTE_TRANSPORT_HOST, LEGACY_REMOTE_TRANSPORT_HOST);
    }
}
