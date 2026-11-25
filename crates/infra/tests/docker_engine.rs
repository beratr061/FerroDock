use std::process::Command;
use std::time::Duration;

use ferrodock_core::AppResult;
use ferrodock_infra::docker::{ContainerAction, DockerEngine, LogQuery};
use futures_util::StreamExt;

const IMAGE: &str = "alpine:3.22";

struct TestContainer(String);

impl TestContainer {
    fn start(name: &str) -> Option<Self> {
        let _ = Command::new("docker").args(["rm", "-f", name]).output();
        let status = Command::new("docker")
            .args([
                "run",
                "-d",
                "--name",
                name,
                "--label",
                "ferrodock.managed=true",
                IMAGE,
            ])
            .args(["sh", "-c", "echo ready-out; echo ready-err >&2; sleep 300"])
            .status()
            .ok()?;
        status.success().then(|| Self(name.to_owned()))
    }
}

impl Drop for TestContainer {
    fn drop(&mut self) {
        let _ = Command::new("docker").args(["rm", "-f", &self.0]).output();
    }
}

async fn state_of(engine: &DockerEngine, name: &str) -> AppResult<String> {
    let containers = engine.list_containers(true).await?;
    Ok(containers
        .into_iter()
        .find(|container| container.name == name)
        .map(|container| container.state)
        .unwrap_or_default())
}

fn watch_for_die(engine: &DockerEngine, name: &'static str) -> tokio::task::JoinHandle<bool> {
    let mut events = engine.container_events();
    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            if let Ok(event) = event
                && event.name == name
                && event.action == "die"
            {
                return true;
            }
        }
        false
    })
}

async fn log_texts(engine: &DockerEngine, name: &str) -> AppResult<Vec<String>> {
    let lines: Vec<_> = engine
        .container_logs(
            name,
            LogQuery {
                tail: 10,
                follow: false,
            },
        )?
        .collect()
        .await;
    Ok(lines
        .into_iter()
        .filter_map(Result::ok)
        .map(|line| line.text)
        .collect())
}

#[tokio::test]
async fn manages_a_real_container() -> AppResult<()> {
    let name = "ferrodock-infra-test";
    let Some(_container) = TestContainer::start(name) else {
        return Err(ferrodock_core::AppError::validation(
            "DOCKER_NOT_AVAILABLE_FOR_TESTS",
        ));
    };
    let engine = DockerEngine::connect_local()?;
    assert_ne!(engine.info().await?.version, "");
    let listed = engine.list_containers(false).await?;
    assert!(
        listed
            .iter()
            .any(|c| c.name == name && c.managed && c.state == "running")
    );
    let died = watch_for_die(&engine, name);
    tokio::time::sleep(Duration::from_millis(500)).await;
    engine.apply(name, ContainerAction::Stop).await?;
    assert_eq!(state_of(&engine, name).await?, "exited");
    let observed = tokio::time::timeout(Duration::from_secs(20), died).await;
    assert!(matches!(observed, Ok(Ok(true))));
    engine.apply(name, ContainerAction::Stop).await?;
    engine.apply(name, ContainerAction::Start).await?;
    assert_eq!(state_of(&engine, name).await?, "running");
    let texts = log_texts(&engine, name).await?;
    assert!(texts.iter().any(|text| text.contains("ready-out")));
    assert!(texts.iter().any(|text| text.contains("ready-err")));
    Ok(())
}

#[tokio::test]
async fn rejects_unknown_and_malformed_references() -> AppResult<()> {
    let engine = DockerEngine::connect_local()?;
    let missing = engine
        .apply("ferrodock-does-not-exist", ContainerAction::Start)
        .await;
    assert!(missing.is_err_and(|e| e.code() == "CONTAINER_NOT_FOUND"));
    let malformed = engine.apply("../etc", ContainerAction::Start).await;
    assert!(malformed.is_err_and(|e| e.code() == "CONTAINER_REFERENCE_INVALID"));
    Ok(())
}

#[tokio::test]
async fn refuses_actions_on_control_plane_containers() -> AppResult<()> {
    let name = "ferrodock-infra-protected";
    let _ = Command::new("docker").args(["rm", "-f", name]).output();
    let started = Command::new("docker")
        .args(["run", "-d", "--name", name])
        .args([
            "--label",
            "com.docker.compose.project=ferrodock",
            IMAGE,
            "sleep",
            "300",
        ])
        .status()
        .is_ok_and(|status| status.success());
    let _cleanup = TestContainer(name.to_owned());
    assert!(started);
    let engine = DockerEngine::connect_local()?;
    let listed = engine.list_containers(false).await?;
    assert!(listed.iter().any(|c| c.name == name && c.protected));
    let refused = engine.apply(name, ContainerAction::Stop).await;
    assert!(refused.is_err_and(|e| {
        e.params()
            .get("reason")
            .is_some_and(|r| r == "CONTAINER_PROTECTED")
    }));
    assert_eq!(state_of(&engine, name).await?, "running");
    Ok(())
}
