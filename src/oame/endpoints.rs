use url::Url;

pub const AGENT_API_ORIGIN: &str = "https://agent.oame.net";
pub const AGENT_API_BASE_PATH: &str = "/v1/";

pub const ENROLL_PATH: &str = "/v1/enroll";
pub const HEARTBEAT_PATH: &str = "/v1/heartbeat";
pub const CONFIG_PATH: &str = "/v1/config";
pub const IDENTITY_PATH: &str = "/v1/identity";

pub const REMOTE_TRANSPORT_HOST: &str = "remote.oame.net";
pub const LEGACY_REMOTE_TRANSPORT_HOST: &str = "sup.oame.net";

pub fn agent_api_base_url() -> Result<Url, String> {
    Url::parse(&format!("{AGENT_API_ORIGIN}{AGENT_API_BASE_PATH}"))
        .map_err(|e| format!("invalid built-in OAME Agent API URL: {e}"))
}

pub fn agent_api_url(path: &str) -> Result<Url, String> {
    if !matches!(path, ENROLL_PATH | HEARTBEAT_PATH | CONFIG_PATH | IDENTITY_PATH) {
        return Err("unsupported OAME Agent API v1 endpoint".to_owned());
    }
    Url::parse(&format!("{AGENT_API_ORIGIN}{path}"))
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

    #[test]
    fn wire_protocol_v1_exposes_only_approved_machine_paths() {
        for path in [ENROLL_PATH, HEARTBEAT_PATH, CONFIG_PATH, IDENTITY_PATH] {
            let url = agent_api_url(path).expect("approved Agent API endpoint");
            assert_eq!(url.host_str(), Some("agent.oame.net"));
            assert_eq!(url.path(), path);
        }
        assert!(agent_api_url("/v1/other").is_err());
    }
}
