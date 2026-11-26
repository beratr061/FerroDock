use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ferrodock_core::crypto::random_bytes;
use ferrodock_core::{AppError, AppResult, Id};

use super::live::LiveSession;

const DETACHED_TTL: Duration = Duration::from_mins(10);
const EXITED_TTL: Duration = Duration::from_secs(30);
const SWEEP_INTERVAL: Duration = Duration::from_secs(30);
pub const MAX_SESSIONS_PER_USER: usize = 16;

type Sessions = Mutex<HashMap<String, Arc<LiveSession>>>;

#[derive(Debug, Clone, Default)]
pub struct SessionRegistry {
    sessions: Arc<Sessions>,
    sweeping: Arc<AtomicBool>,
}

fn lock(sessions: &Sessions) -> MutexGuard<'_, HashMap<String, Arc<LiveSession>>> {
    sessions.lock().unwrap_or_else(PoisonError::into_inner)
}

fn stale(session: &LiveSession) -> bool {
    let limit = if session.has_exited() {
        EXITED_TTL
    } else {
        DETACHED_TTL
    };
    session.idle_for().is_some_and(|idle| idle >= limit)
}

fn sweep(sessions: &Sessions) {
    lock(sessions).retain(|_, session| {
        let keep = !stale(session);
        if !keep {
            session.close();
        }
        keep
    });
}

async fn sweep_forever(sessions: Weak<Sessions>) {
    loop {
        tokio::time::sleep(SWEEP_INTERVAL).await;
        let Some(sessions) = sessions.upgrade() else {
            return;
        };
        sweep(&sessions);
    }
}

pub fn new_session_id() -> String {
    URL_SAFE_NO_PAD.encode(random_bytes::<32>())
}

impl SessionRegistry {
    pub fn reserve(&self, user_id: Id) -> AppResult<()> {
        sweep(&self.sessions);
        let open = lock(&self.sessions)
            .values()
            .filter(|session| session.user_id == user_id)
            .count();
        if open >= MAX_SESSIONS_PER_USER {
            return Err(AppError::conflict("TERMINAL_SESSION_LIMIT")
                .with_param("max", MAX_SESSIONS_PER_USER));
        }
        Ok(())
    }

    pub fn insert(&self, session: Arc<LiveSession>) {
        if !self.sweeping.swap(true, Ordering::SeqCst) {
            tokio::spawn(sweep_forever(Arc::downgrade(&self.sessions)));
        }
        lock(&self.sessions).insert(session.id.clone(), session);
    }

    pub fn find(&self, id: &str, user_id: Id, organization_id: Id) -> AppResult<Arc<LiveSession>> {
        lock(&self.sessions)
            .get(id)
            .filter(|session| {
                session.user_id == user_id && session.organization_id == organization_id
            })
            .cloned()
            .ok_or_else(|| AppError::not_found("TERMINAL_SESSION_NOT_FOUND"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::live::{Attachment, Owner};

    fn attached(registry: &SessionRegistry, owner: Owner) -> (String, Attachment) {
        let id = new_session_id();
        let sink = Box::pin(tokio::io::sink());
        let shell = LiveSession::new(id.clone(), owner, "exec".to_owned(), sink);
        let attachment = shell.attach();
        registry.insert(shell);
        (id, attachment)
    }

    #[tokio::test]
    async fn only_the_owner_can_resume_and_the_limit_holds() {
        let registry = SessionRegistry::default();
        let owner = Owner {
            user_id: Id::new(),
            organization_id: Id::new(),
        };
        let (id, first) = attached(&registry, owner);
        assert_eq!(id.len(), 43);
        let mine = registry.find(&id, owner.user_id, owner.organization_id);
        assert!(mine.is_ok());
        let stranger = registry.find(&id, Id::new(), owner.organization_id);
        let code = stranger.err().map(|e| e.code());
        assert_eq!(code, Some("TERMINAL_SESSION_NOT_FOUND"));
        let mut held = vec![first];
        for _ in 1..MAX_SESSIONS_PER_USER {
            held.push(attached(&registry, owner).1);
        }
        let limited = registry.reserve(owner.user_id).err().map(|e| e.code());
        assert_eq!(limited, Some("TERMINAL_SESSION_LIMIT"));
        assert!(registry.reserve(Id::new()).is_ok());
        assert_eq!(held.len(), MAX_SESSIONS_PER_USER);
    }
}
