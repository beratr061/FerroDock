use ferrodock_core::crypto::hex;
use hmac::{Hmac, KeyInit, Mac};
use serde_json::Value;
use sha2::Sha256;
use subtle::ConstantTimeEq;

#[derive(Debug, Clone, Copy, Default)]
pub struct Delivery<'a> {
    pub event: Option<&'a str>,
    pub hub_signature: Option<&'a str>,
    pub gitea_signature: Option<&'a str>,
    pub gitlab_token: Option<&'a str>,
    pub body: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Push {
    Branch { name: String, commit: String },
    Deleted,
    Tag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Push,
    Ping,
    Other,
}

fn hmac_hex(secret: &str, body: &[u8]) -> String {
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
        return String::new();
    };
    mac.update(body);
    hex(&mac.finalize().into_bytes())
}

fn same(expected: &str, given: &str) -> bool {
    !expected.is_empty() && bool::from(expected.as_bytes().ct_eq(given.trim().as_bytes()))
}

impl Delivery<'_> {
    pub fn is_signed_with(&self, secret: &str) -> bool {
        if let Some(header) = self.hub_signature {
            let given = header.trim().strip_prefix("sha256=").unwrap_or_default();
            return same(&hmac_hex(secret, self.body), given);
        }
        if let Some(header) = self.gitea_signature {
            return same(&hmac_hex(secret, self.body), &header.to_ascii_lowercase());
        }
        self.gitlab_token.is_some_and(|token| same(secret, token))
    }

    pub fn event(&self) -> Event {
        match self.event.map(str::trim) {
            Some("push" | "Push Hook") => Event::Push,
            Some("ping") => Event::Ping,
            _ => Event::Other,
        }
    }

    pub fn push(&self) -> Option<Push> {
        let payload: Value = serde_json::from_slice(self.body).ok()?;
        let reference = payload.get("ref")?.as_str()?;
        let commit = payload
            .get("after")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let deleted = payload.get("deleted").and_then(Value::as_bool) == Some(true)
            || (!commit.is_empty() && commit.bytes().all(|byte| byte == b'0'));
        if deleted {
            return Some(Push::Deleted);
        }
        let Some(name) = reference.strip_prefix("refs/heads/") else {
            return Some(Push::Tag);
        };
        Some(Push::Branch {
            name: name.to_owned(),
            commit: commit.to_owned(),
        })
    }
}
