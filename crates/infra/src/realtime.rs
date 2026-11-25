use std::sync::Arc;
use std::time::{Duration, Instant};

use ferrodock_core::Id;
use serde::Serialize;
use tokio::sync::{Mutex, broadcast};

const HUB_CAPACITY: usize = 512;
const LOG_SIGNAL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Signal {
    DeploymentLog {
        deployment_id: Id,
    },
    DeploymentStatus {
        application_id: Id,
        deployment_id: Id,
    },
    ContainersChanged,
    Resync,
}

#[derive(Debug, Clone)]
pub struct Envelope {
    pub organization_id: Option<Id>,
    pub signal: Signal,
}

impl Envelope {
    pub fn visible_to(&self, organization_id: Id) -> bool {
        self.organization_id
            .is_none_or(|owner| owner == organization_id)
    }
}

#[derive(Debug, Clone)]
pub struct Hub {
    sender: broadcast::Sender<Envelope>,
}

impl Default for Hub {
    fn default() -> Self {
        Self::new()
    }
}

impl Hub {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(HUB_CAPACITY);
        Self { sender }
    }

    pub fn publish(&self, organization_id: Option<Id>, signal: Signal) {
        let _ = self.sender.send(Envelope {
            organization_id,
            signal,
        });
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Envelope> {
        self.sender.subscribe()
    }
}

#[derive(Debug, Default)]
struct Throttle {
    last: Option<Instant>,
    trailing: bool,
}

#[derive(Debug, Clone)]
pub struct LogSignal {
    hub: Hub,
    organization_id: Option<Id>,
    deployment_id: Id,
    throttle: Arc<Mutex<Throttle>>,
}

impl LogSignal {
    pub fn new(hub: Hub, organization_id: Option<Id>, deployment_id: Id) -> Self {
        Self {
            hub,
            organization_id,
            deployment_id,
            throttle: Arc::new(Mutex::new(Throttle::default())),
        }
    }

    fn send(&self) {
        let signal = Signal::DeploymentLog {
            deployment_id: self.deployment_id,
        };
        self.hub.publish(self.organization_id, signal);
    }

    pub async fn appended(&self, force: bool) {
        let mut throttle = self.throttle.lock().await;
        let due = throttle
            .last
            .is_none_or(|sent| sent.elapsed() >= LOG_SIGNAL_INTERVAL);
        if force || due {
            throttle.last = Some(Instant::now());
            self.send();
        } else if !throttle.trailing {
            throttle.trailing = true;
            let signal = self.clone();
            tokio::spawn(async move {
                tokio::time::sleep(LOG_SIGNAL_INTERVAL).await;
                let mut throttle = signal.throttle.lock().await;
                throttle.trailing = false;
                throttle.last = Some(Instant::now());
                signal.send();
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn throttles_log_signals_and_scopes_by_organization() {
        let hub = Hub::new();
        let mut receiver = hub.subscribe();
        let organization = Id::new();
        let signal = LogSignal::new(hub.clone(), Some(organization), Id::new());
        signal.appended(false).await;
        signal.appended(false).await;
        signal.appended(false).await;
        signal.appended(true).await;
        let first = receiver.try_recv().ok();
        let second = receiver.try_recv().ok();
        assert!(receiver.try_recv().is_err());
        tokio::time::sleep(LOG_SIGNAL_INTERVAL * 2).await;
        assert!(receiver.try_recv().is_ok());
        assert!(receiver.try_recv().is_err());
        assert!(first.is_some_and(|e| e.visible_to(organization) && !e.visible_to(Id::new())));
        assert!(second.is_some());
        hub.publish(None, Signal::ContainersChanged);
        assert!(receiver.try_recv().is_ok_and(|e| e.visible_to(Id::new())));
    }
}
