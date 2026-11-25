use ferrodock_core::Id;

pub fn short_key(id: Id, length: usize) -> String {
    let simple = id.as_uuid().simple().to_string();
    simple[simple.len().saturating_sub(length)..].to_owned()
}

pub fn application_key(id: Id) -> String {
    short_key(id, 12)
}

pub fn compose_project(application_id: Id) -> String {
    format!("fd-{}", application_key(application_id))
}

pub fn network_alias(application_id: Id) -> String {
    format!("fd-{}", application_key(application_id))
}
