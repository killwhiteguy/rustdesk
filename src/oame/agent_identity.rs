use hbb_common::{
    config::Config,
    sodiumoxide::{
        base64::{decode, encode, Variant},
        crypto::sign,
    },
    uuid::Uuid,
};
use serde_derive::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use super::win_dpapi::{protect_machine, unprotect_machine};

const STORAGE_VERSION: u32 = 2;
const STORAGE_FILE: &str = "oame_agent_identity.json";
const SELF_CHECK_MESSAGE: &[u8] = b"oame-agent-identity-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredAgentIdentity {
    version: u32,
    installation_id: String,
    public_key: String,
    protected_private_key: String,
    created_at: u64,
}

#[derive(Debug, Clone)]
pub struct AgentIdentity {
    installation_id: String,
    public_key: Vec<u8>,
    private_key: Vec<u8>,
    created_at: u64,
}

impl AgentIdentity {
    pub fn load_or_create() -> Result<Self, String> {
        match Self::load()? {
            Some(identity) => Ok(identity),
            None => {
                let identity = Self::generate()?;
                identity.store()?;
                Ok(identity)
            }
        }
    }

    pub fn load() -> Result<Option<Self>, String> {
        let path = Self::storage_path();
        if !path.exists() {
            return Ok(None);
        }

        let raw = fs::read_to_string(&path)
            .map_err(|e| format!("failed to read OAME agent identity '{}': {e}", path.display()))?;
        let stored: StoredAgentIdentity = serde_json::from_str(&raw)
            .map_err(|e| format!("failed to parse OAME agent identity '{}': {e}", path.display()))?;

        if stored.version != STORAGE_VERSION {
            return Err(format!(
                "unsupported OAME agent identity storage version: {}",
                stored.version
            ));
        }

        Uuid::parse_str(&stored.installation_id)
            .map_err(|e| format!("invalid OAME installation_id: {e}"))?;

        let public_key = decode(stored.public_key.as_bytes(), Variant::Original)
            .map_err(|_| "invalid OAME installation public key encoding".to_owned())?;
        let protected_private_key = decode(stored.protected_private_key.as_bytes(), Variant::Original)
            .map_err(|_| "invalid OAME protected private key encoding".to_owned())?;
        let private_key = unprotect_machine(&protected_private_key)?;

        let identity = Self {
            installation_id: stored.installation_id,
            public_key,
            private_key,
            created_at: stored.created_at,
        };
        identity.validate()?;
        Ok(Some(identity))
    }

    pub fn installation_id(&self) -> &str {
        &self.installation_id
    }

    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, String> {
        let secret_key = sign::SecretKey::from_slice(&self.private_key)
            .ok_or_else(|| "invalid OAME installation private key".to_owned())?;
        Ok(sign::sign_detached(message, &secret_key).to_bytes().to_vec())
    }

    fn generate() -> Result<Self, String> {
        hbb_common::sodiumoxide::init()
            .map_err(|_| "failed to initialize sodiumoxide for OAME agent identity".to_owned())?;
        let (public_key, private_key) = sign::gen_keypair();
        let identity = Self {
            installation_id: Uuid::new_v4().to_string(),
            public_key: public_key.0.to_vec(),
            private_key: private_key.0.to_vec(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| format!("system clock error while creating OAME agent identity: {e}"))?
                .as_secs(),
        };
        identity.validate()?;
        Ok(identity)
    }

    fn validate(&self) -> Result<(), String> {
        Uuid::parse_str(&self.installation_id)
            .map_err(|e| format!("invalid OAME installation_id: {e}"))?;

        let public_key = sign::PublicKey::from_slice(&self.public_key)
            .ok_or_else(|| "invalid OAME installation public key".to_owned())?;
        let private_key = sign::SecretKey::from_slice(&self.private_key)
            .ok_or_else(|| "invalid OAME installation private key".to_owned())?;
        let signature = sign::sign_detached(SELF_CHECK_MESSAGE, &private_key);
        if !sign::verify_detached(&signature, SELF_CHECK_MESSAGE, &public_key) {
            return Err("OAME installation keypair self-check failed".to_owned());
        }
        Ok(())
    }

    fn store(&self) -> Result<(), String> {
        self.validate()?;
        let protected_private_key = protect_machine(&self.private_key)?;
        let stored = StoredAgentIdentity {
            version: STORAGE_VERSION,
            installation_id: self.installation_id.clone(),
            public_key: encode(&self.public_key, Variant::Original),
            protected_private_key: encode(&protected_private_key, Variant::Original),
            created_at: self.created_at,
        };
        let raw = serde_json::to_vec_pretty(&stored)
            .map_err(|e| format!("failed to serialize OAME agent identity: {e}"))?;
        let path = Self::storage_path();
        let parent = path
            .parent()
            .ok_or_else(|| "OAME agent identity storage path has no parent".to_owned())?;
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "failed to create OAME agent identity directory '{}': {e}",
                parent.display()
            )
        })?;

        let tmp_path = path.with_extension("json.tmp");
        fs::write(&tmp_path, raw).map_err(|e| {
            format!(
                "failed to write temporary OAME agent identity '{}': {e}",
                tmp_path.display()
            )
        })?;
        fs::rename(&tmp_path, &path).map_err(|e| {
            let _ = fs::remove_file(&tmp_path);
            format!(
                "failed to persist OAME agent identity '{}': {e}",
                path.display()
            )
        })?;
        Ok(())
    }

    fn storage_path() -> PathBuf {
        Config::path(STORAGE_FILE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_identity_has_valid_uuid_and_signature() {
        let identity = AgentIdentity::generate().expect("identity generation");
        Uuid::parse_str(identity.installation_id()).expect("installation_id must be UUID");

        let message = b"oame-agent-test";
        let signature = identity.sign(message).expect("signing");
        let signature = sign::Signature::from_slice(&signature).expect("signature bytes");
        let public_key = sign::PublicKey::from_slice(identity.public_key()).expect("public key");
        assert!(sign::verify_detached(&signature, message, &public_key));
    }

    #[test]
    fn generated_identities_are_distinct() {
        let first = AgentIdentity::generate().expect("first identity");
        let second = AgentIdentity::generate().expect("second identity");
        assert_ne!(first.installation_id(), second.installation_id());
        assert_ne!(first.public_key(), second.public_key());
    }

    #[test]
    fn private_key_round_trips_through_dpapi() {
        let identity = AgentIdentity::generate().expect("identity generation");
        let protected = protect_machine(&identity.private_key).expect("DPAPI protect");
        assert_ne!(protected, identity.private_key);
        let restored = unprotect_machine(&protected).expect("DPAPI unprotect");
        assert_eq!(restored, identity.private_key);
    }
}
