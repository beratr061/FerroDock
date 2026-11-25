use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use ed25519_dalek::SigningKey;
use ferrodock_core::crypto::random_bytes;
use zeroize::Zeroizing;

const KEY_TYPE: &[u8] = b"ssh-ed25519";
const AUTH_MAGIC: &[u8] = b"openssh-key-v1\0";
const CIPHER_BLOCK: usize = 8;
const PEM_LINE: usize = 70;

#[derive(Debug)]
pub struct DeployKey {
    pub public_openssh: String,
    pub private_openssh: Zeroizing<String>,
}

fn put_string(buffer: &mut Vec<u8>, bytes: &[u8]) {
    let length = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
    buffer.extend_from_slice(&length.to_be_bytes());
    buffer.extend_from_slice(bytes);
}

fn public_blob(public: &[u8]) -> Vec<u8> {
    let mut blob = Vec::with_capacity(51);
    put_string(&mut blob, KEY_TYPE);
    put_string(&mut blob, public);
    blob
}

fn private_section(seed: &[u8; 32], public: &[u8], comment: &str) -> Zeroizing<Vec<u8>> {
    let check = random_bytes::<4>();
    let mut section = Zeroizing::new(Vec::with_capacity(160));
    section.extend_from_slice(&check);
    section.extend_from_slice(&check);
    put_string(&mut section, KEY_TYPE);
    put_string(&mut section, public);
    let mut keypair = Zeroizing::new(Vec::with_capacity(64));
    keypair.extend_from_slice(seed);
    keypair.extend_from_slice(public);
    put_string(&mut section, &keypair);
    put_string(&mut section, comment.as_bytes());
    let mut pad = 1u8;
    while section.len() % CIPHER_BLOCK != 0 {
        section.push(pad);
        pad += 1;
    }
    section
}

fn armor(body: &[u8]) -> Zeroizing<String> {
    let encoded = Zeroizing::new(STANDARD.encode(body));
    let mut pem = Zeroizing::new(String::from("-----BEGIN OPENSSH PRIVATE KEY-----\n"));
    for line in encoded.as_bytes().chunks(PEM_LINE) {
        pem.push_str(&String::from_utf8_lossy(line));
        pem.push('\n');
    }
    pem.push_str("-----END OPENSSH PRIVATE KEY-----\n");
    pem
}

pub fn generate_deploy_key(comment: &str) -> DeployKey {
    let seed = Zeroizing::new(random_bytes::<32>());
    let public = SigningKey::from_bytes(&seed).verifying_key().to_bytes();
    let blob = public_blob(&public);
    let mut body = Zeroizing::new(Vec::with_capacity(256));
    body.extend_from_slice(AUTH_MAGIC);
    put_string(&mut body, b"none");
    put_string(&mut body, b"none");
    put_string(&mut body, b"");
    body.extend_from_slice(&1u32.to_be_bytes());
    put_string(&mut body, &blob);
    put_string(&mut body, &private_section(&seed, &public, comment));
    DeployKey {
        public_openssh: format!("ssh-ed25519 {} {comment}", STANDARD.encode(&blob)),
        private_openssh: armor(&body),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_openssh_ed25519_pair() {
        let key = generate_deploy_key("ferrodock@shop");
        assert!(
            key.public_openssh
                .starts_with("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI")
        );
        assert!(key.public_openssh.ends_with(" ferrodock@shop"));
        assert!(key.private_openssh.starts_with(
            "-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQ"
        ));
        assert!(
            key.private_openssh
                .lines()
                .all(|line| line.len() <= PEM_LINE.max(36))
        );
        assert_ne!(
            generate_deploy_key("x").public_openssh,
            generate_deploy_key("x").public_openssh
        );
    }
}
