use ferrodock_core::{AppError, AppResult};
use zeroize::Zeroizing;

use super::model::Principal;
use super::password;
use super::service::AuthService;
use super::throttle::source_key;

fn wrong_password() -> AppError {
    AppError::forbidden().with_param("reason", "PASSWORD_INCORRECT")
}

impl AuthService {
    pub async fn confirm_password(
        &self,
        principal: &Principal,
        password: Zeroizing<String>,
        ip_address: Option<&str>,
    ) -> AppResult<()> {
        let user = self
            .repos
            .identity
            .find_user(principal.user_id)
            .await?
            .ok_or_else(AppError::unauthorized)?;
        let source = source_key(ip_address);
        if self.is_throttled(user.id, source).await? {
            password::dummy_verify(password).await;
            return Err(wrong_password());
        }
        if !password::verify(password, user.password_hash).await? {
            self.record_login_failure(user.id, source).await?;
            return Err(wrong_password());
        }
        self.clear_login_failures(user.id, source).await
    }
}
