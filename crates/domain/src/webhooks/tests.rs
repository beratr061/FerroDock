use super::delivery::{Delivery, Event, Push};

const SECRET: &str = "It's a Secret to Everybody";
const BODY: &[u8] = b"Hello, World!";
const GITHUB_EXAMPLE: &str = "757107ea0eb2509fc211221cce984b8a37570b6d7586c22c46f4379c8b043e17";

#[test]
fn github_signature_matches_the_documented_example() {
    let header = format!("sha256={GITHUB_EXAMPLE}");
    let delivery = Delivery {
        hub_signature: Some(&header),
        body: BODY,
        ..Delivery::default()
    };
    assert!(delivery.is_signed_with(SECRET));
    assert!(!delivery.is_signed_with("another secret"));
}

#[test]
fn gitea_and_gitlab_headers_are_accepted() {
    let gitea = Delivery {
        gitea_signature: Some(GITHUB_EXAMPLE),
        body: BODY,
        ..Delivery::default()
    };
    assert!(gitea.is_signed_with(SECRET));
    let gitlab = Delivery {
        gitlab_token: Some(SECRET),
        body: BODY,
        ..Delivery::default()
    };
    assert!(gitlab.is_signed_with(SECRET));
    assert!(!gitlab.is_signed_with("It's a Secret to Everybody!"));
}

#[test]
fn unsigned_or_tampered_deliveries_are_refused() {
    assert!(!Delivery::default().is_signed_with(SECRET));
    let header = format!("sha256={GITHUB_EXAMPLE}");
    let tampered = Delivery {
        hub_signature: Some(&header),
        body: b"Hello, World?",
        ..Delivery::default()
    };
    assert!(!tampered.is_signed_with(SECRET));
    let empty = Delivery {
        hub_signature: Some("sha256="),
        body: BODY,
        ..Delivery::default()
    };
    assert!(!empty.is_signed_with(SECRET));
}

fn push(body: &str) -> Option<Push> {
    Delivery {
        body: body.as_bytes(),
        ..Delivery::default()
    }
    .push()
}

#[test]
fn push_payloads_are_read() {
    assert_eq!(
        push(r#"{"ref":"refs/heads/main","after":"abc123"}"#),
        Some(Push::Branch {
            name: "main".to_owned(),
            commit: "abc123".to_owned()
        })
    );
    assert_eq!(
        push(r#"{"ref":"refs/tags/v1","after":"abc"}"#),
        Some(Push::Tag)
    );
    assert_eq!(
        push(r#"{"ref":"refs/heads/main","after":"0000000000000000000000000000000000000000"}"#),
        Some(Push::Deleted)
    );
    assert_eq!(push("not json"), None);
}

#[test]
fn events_are_named_per_provider() {
    let named = |event| Delivery {
        event: Some(event),
        ..Delivery::default()
    };
    assert_eq!(named("push").event(), Event::Push);
    assert_eq!(named("Push Hook").event(), Event::Push);
    assert_eq!(named("ping").event(), Event::Ping);
    assert_eq!(named("pull_request").event(), Event::Other);
}
