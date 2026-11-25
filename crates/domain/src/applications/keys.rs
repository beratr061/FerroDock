use ferrodock_core::AppResult;
use ferrodock_core::crypto::SecretBox;
use ferrodock_db::applications::DeployKeyRecord;
use ferrodock_infra::git::{GitTransport, generate_deploy_key, parse_repository_url};

use super::model::Source;

pub(super) fn needs_deploy_key(source: &Source) -> AppResult<bool> {
    match source {
        Source::Git(git) => Ok(parse_repository_url(&git.url)? == GitTransport::Ssh),
        Source::Image(_) => Ok(false),
    }
}

pub(super) fn new_deploy_key(secrets: &SecretBox, slug: &str) -> AppResult<DeployKeyRecord> {
    let key = generate_deploy_key(&format!("ferrodock@{slug}"));
    Ok(DeployKeyRecord {
        private_enc: secrets.encrypt(key.private_openssh.as_bytes())?,
        public: key.public_openssh,
    })
}

pub(super) fn settle_deploy_key(
    secrets: &SecretBox,
    source: &Source,
    slug: &str,
    current: Option<DeployKeyRecord>,
) -> AppResult<Option<DeployKeyRecord>> {
    if !needs_deploy_key(source)? {
        return Ok(None);
    }
    match current {
        Some(key) => Ok(Some(key)),
        None => new_deploy_key(secrets, slug).map(Some),
    }
}
