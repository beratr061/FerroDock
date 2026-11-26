use ferrodock_core::{AppError, AppResult};
use ferrodock_db::applications::{ApplicationRecord, GitSourceRecord, SourceRecord};
use ferrodock_db::deployments::{DeploymentArtifact, DeploymentKind, DeploymentRecord};
use ferrodock_infra::deploy_log::DeployLog;
use ferrodock_infra::docker::{ComposeUp, ImageBuild, build_image, compose_up};
use ferrodock_infra::git::{CheckoutSpec, checkout, contained_path};
use tokio_util::sync::CancellationToken;

use super::context::{DeployContext, Names};

pub enum Prepared {
    Image(DeploymentArtifact),
    Compose(DeploymentArtifact),
}

pub struct Job<'a> {
    pub ctx: &'a DeployContext,
    pub application: &'a ApplicationRecord,
    pub deployment: &'a DeploymentRecord,
    pub names: &'a Names,
    pub variables: &'a [(String, String)],
    pub log: &'a DeployLog,
    pub cancel: &'a CancellationToken,
}

impl Job<'_> {
    pub async fn prepare(&self) -> AppResult<Prepared> {
        if self.deployment.kind == DeploymentKind::Rollback {
            return self.rollback_target().await.map(Prepared::Image);
        }
        match &self.application.source {
            SourceRecord::Image(reference) => self.pull(reference).await.map(Prepared::Image),
            SourceRecord::Git(git) if git.build_kind == "compose" => {
                self.compose(git).await.map(Prepared::Compose)
            }
            SourceRecord::Git(git) => self.build(git).await.map(Prepared::Image),
        }
    }

    async fn rollback_target(&self) -> AppResult<DeploymentArtifact> {
        let missing = || AppError::validation("ROLLBACK_TARGET_UNAVAILABLE");
        let target_id = self.deployment.rollback_of.ok_or_else(missing)?;
        let target = self
            .ctx
            .deployments
            .find(target_id)
            .await?
            .ok_or_else(missing)?;
        let image_id = target
            .image_id
            .filter(|_| target.application_id == self.application.id);
        let image_id = image_id.ok_or_else(missing)?;
        self.ctx
            .docker
            .image_id(&image_id)
            .await
            .map_err(|_| missing())?;
        self.log
            .step(&format!("Rolling back to deployment {}", target.id))
            .await;
        Ok(DeploymentArtifact {
            commit_sha: target.commit_sha,
            image_ref: target.image_ref.unwrap_or_else(|| image_id.clone()),
            image_id,
        })
    }

    async fn pull(&self, reference: &str) -> AppResult<DeploymentArtifact> {
        self.ctx
            .docker
            .pull_image(reference, self.log, self.cancel)
            .await?;
        Ok(DeploymentArtifact {
            commit_sha: None,
            image_ref: reference.to_owned(),
            image_id: self.ctx.docker.image_id(reference).await?,
        })
    }

    fn private_key(&self) -> AppResult<Option<zeroize::Zeroizing<Vec<u8>>>> {
        self.application
            .deploy_key
            .as_ref()
            .map(|key| self.ctx.secrets.decrypt(&key.private_enc))
            .transpose()
    }

    async fn fetch(
        &self,
        git: &GitSourceRecord,
        destination: &std::path::Path,
    ) -> AppResult<String> {
        let key = self.private_key()?;
        let known_hosts = self.ctx.known_hosts();
        let spec = CheckoutSpec {
            url: &git.url,
            branch: &git.branch,
            destination,
            private_key: key.as_deref().map(Vec::as_slice),
            known_hosts: &known_hosts,
        };
        let commit = checkout(spec, self.log, self.cancel).await?;
        self.log.line(&format!("Checked out {commit}")).await;
        Ok(commit)
    }

    async fn build(&self, git: &GitSourceRecord) -> AppResult<DeploymentArtifact> {
        let workspace = self.ctx.build_dir(self.deployment.id);
        let result = self.build_in(git, &workspace).await;
        let _ = tokio::fs::remove_dir_all(&workspace).await;
        result
    }

    async fn build_in(
        &self,
        git: &GitSourceRecord,
        workspace: &std::path::Path,
    ) -> AppResult<DeploymentArtifact> {
        let commit = self.fetch(git, workspace).await?;
        let tag = self.names.image_tag();
        let application_id = self.application.id.to_string();
        let build = ImageBuild {
            dockerfile: &contained_path(workspace, &git.build_path)?,
            context: &contained_path(workspace, &git.build_context)?,
            tag: &tag,
            labels: &[
                ("ferrodock.application", &application_id),
                ("ferrodock.commit", &commit),
            ],
        };
        build_image(build, self.log, self.cancel).await?;
        Ok(DeploymentArtifact {
            image_id: self.ctx.docker.image_id(&tag).await?,
            commit_sha: Some(commit),
            image_ref: tag,
        })
    }

    async fn compose(&self, git: &GitSourceRecord) -> AppResult<DeploymentArtifact> {
        let directory = self.ctx.compose_dir(self.names);
        let commit = self.fetch(git, &directory).await?;
        let project = self.names.compose_project();
        let compose = ComposeUp {
            project: &project,
            variables: self.variables,
            file: &contained_path(&directory, &git.build_path)?,
            directory: &directory,
            application_id: self.application.id,
            trusted: self.application.compose_trusted,
        };
        self.ctx.docker.ensure_network().await?;
        compose_up(&self.ctx.docker, compose, self.log, self.cancel).await?;
        Ok(DeploymentArtifact {
            image_ref: format!("compose:{project}"),
            image_id: commit.clone(),
            commit_sha: Some(commit),
        })
    }
}
