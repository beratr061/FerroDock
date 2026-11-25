use std::time::Duration;

use ferrodock_infra::docker::DockerEngine;
use ferrodock_infra::realtime::{Hub, Signal};
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

const RETRY_DELAY: Duration = Duration::from_secs(10);
const COALESCE: Duration = Duration::from_millis(300);

pub fn forward(docker: DockerEngine, hub: Hub, shutdown: CancellationToken) {
    tokio::spawn(async move {
        while !shutdown.is_cancelled() {
            relay(&docker, &hub, &shutdown).await;
            tokio::select! {
                () = shutdown.cancelled() => return,
                () = tokio::time::sleep(RETRY_DELAY) => {}
            }
        }
    });
}

async fn relay(docker: &DockerEngine, hub: &Hub, shutdown: &CancellationToken) {
    let mut events = docker.container_events();
    loop {
        let next = tokio::select! {
            () = shutdown.cancelled() => return,
            next = events.next() => next,
        };
        if !matches!(next, Some(Ok(_))) {
            return;
        }
        let window = tokio::time::sleep(COALESCE);
        tokio::pin!(window);
        loop {
            tokio::select! {
                () = &mut window => break,
                more = events.next() => if !matches!(more, Some(Ok(_))) { return },
            }
        }
        hub.publish(None, Signal::ContainersChanged);
    }
}
