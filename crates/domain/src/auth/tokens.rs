use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ferrodock_core::crypto::random_bytes;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

pub const API_KEY_PREFIX: &str = "fd_live_";
const TOKEN_BYTES: usize = 32;
const DISPLAY_PREFIX_LEN: usize = 12;

pub fn new_session_token() -> Zeroizing<String> {
    Zeroizing::new(URL_SAFE_NO_PAD.encode(random_bytes::<TOKEN_BYTES>()))
}

pub fn new_api_key() -> Zeroizing<String> {
    Zeroizing::new(format!(
        "{API_KEY_PREFIX}{}",
        URL_SAFE_NO_PAD.encode(random_bytes::<TOKEN_BYTES>())
    ))
}

pub fn display_prefix(api_key: &str) -> String {
    api_key.chars().take(DISPLAY_PREFIX_LEN).collect()
}

pub fn digest(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_unique_and_prefixed() {
        let first = new_api_key();
        let second = new_api_key();
        assert_ne!(first.as_str(), second.as_str());
        assert!(first.starts_with(API_KEY_PREFIX));
        assert_eq!(display_prefix(&first).len(), DISPLAY_PREFIX_LEN);
        assert_eq!(digest(&first).len(), 32);
        assert_ne!(new_session_token().as_str(), new_session_token().as_str());
    }
}
