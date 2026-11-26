use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use ferrodock_core::Id;
use ferrodock_infra::docker::{DockerEngine, ShellInput, ShellOutput};
use futures_util::StreamExt;
use tokio::sync::{Notify, broadcast};

use super::replay::{REPLAY_CAPACITY, ReplayBuffer};

const EVENT_BACKLOG: usize = 512;

#[derive(Debug, Clone)]
pub enum TerminalEvent {
    Output(Arc<[u8]>),
    Exit(Option<i64>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellExit {
    pub code: Option<i64>,
}

#[derive(Debug)]
struct Shared {
    replay: ReplayBuffer,
    exit: Option<ShellExit>,
    attached: usize,
    idle_since: Instant,
}

pub struct LiveSession {
    pub id: String,
    pub user_id: Id,
    pub organization_id: Id,
    pub exec_id: String,
    pub input: tokio::sync::Mutex<ShellInput>,
    events: broadcast::Sender<TerminalEvent>,
    shared: Mutex<Shared>,
    closed: Notify,
}

impl std::fmt::Debug for LiveSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiveSession")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct Attachment {
    pub session: Arc<LiveSession>,
    pub replay: Vec<u8>,
    pub exit: Option<ShellExit>,
    pub events: broadcast::Receiver<TerminalEvent>,
}

impl Drop for Attachment {
    fn drop(&mut self) {
        let mut shared = self.session.lock();
        shared.attached = shared.attached.saturating_sub(1);
        shared.idle_since = Instant::now();
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Owner {
    pub user_id: Id,
    pub organization_id: Id,
}

impl LiveSession {
    pub fn new(id: String, owner: Owner, exec_id: String, input: ShellInput) -> Arc<Self> {
        Arc::new(Self {
            id,
            user_id: owner.user_id,
            organization_id: owner.organization_id,
            exec_id,
            input: tokio::sync::Mutex::new(input),
            events: broadcast::channel(EVENT_BACKLOG).0,
            shared: Mutex::new(Shared {
                replay: ReplayBuffer::new(REPLAY_CAPACITY),
                exit: None,
                attached: 0,
                idle_since: Instant::now(),
            }),
            closed: Notify::new(),
        })
    }

    fn lock(&self) -> MutexGuard<'_, Shared> {
        self.shared.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn attach(self: &Arc<Self>) -> Attachment {
        let mut shared = self.lock();
        shared.attached += 1;
        Attachment {
            session: self.clone(),
            replay: shared.replay.snapshot(),
            exit: shared.exit,
            events: self.events.subscribe(),
        }
    }

    pub fn idle_for(&self) -> Option<std::time::Duration> {
        let shared = self.lock();
        (shared.attached == 0).then(|| shared.idle_since.elapsed())
    }

    pub fn has_exited(&self) -> bool {
        self.lock().exit.is_some()
    }

    pub fn close(&self) {
        self.closed.notify_one();
    }

    fn record(&self, event: TerminalEvent) {
        let mut shared = self.lock();
        match &event {
            TerminalEvent::Output(bytes) => shared.replay.push(bytes),
            TerminalEvent::Exit(code) => shared.exit = Some(ShellExit { code: *code }),
        }
        let _ = self.events.send(event);
    }

    pub async fn pump(self: Arc<Self>, mut output: ShellOutput, docker: DockerEngine) {
        loop {
            tokio::select! {
                () = self.closed.notified() => return,
                chunk = output.next() => match chunk {
                    Some(Ok(bytes)) => self.record(TerminalEvent::Output(bytes.into())),
                    _ => break,
                },
            }
        }
        let code = docker.shell_exit_code(&self.exec_id).await;
        self.record(TerminalEvent::Exit(code));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> Arc<LiveSession> {
        let owner = Owner {
            user_id: Id::new(),
            organization_id: Id::new(),
        };
        LiveSession::new(
            "s".to_owned(),
            owner,
            "exec".to_owned(),
            Box::pin(tokio::io::sink()),
        )
    }

    #[tokio::test]
    async fn late_attachments_replay_output_and_see_the_exit() {
        let live = session();
        let mut early = live.attach();
        live.record(TerminalEvent::Output(Arc::from(&b"$ ls\n"[..])));
        assert!(matches!(
            early.events.recv().await,
            Ok(TerminalEvent::Output(_))
        ));
        assert_eq!(live.idle_for(), None);
        drop(early);
        assert!(live.idle_for().is_some());
        live.record(TerminalEvent::Exit(Some(0)));
        let late = live.attach();
        assert_eq!(late.replay, b"$ ls\n");
        assert_eq!(late.exit, Some(ShellExit { code: Some(0) }));
        assert!(live.has_exited());
    }
}
