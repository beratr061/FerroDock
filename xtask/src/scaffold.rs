use std::fs;
use std::path::Path;

use time::OffsetDateTime;
use time::macros::format_description;

use crate::templates::{self, Names};

const NEXT_STEPS: &str = "next steps:
  1. add `pub mod {s};` to crates/domain/src/lib.rs
  2. add `mod {s};` and the route to crates/api/src/routes/mod.rs, then register the path in crates/api/src/openapi.rs
  3. add the feature route to web/src/app/router.tsx and fill in the locale files
  4. run `cargo sqlx prepare --workspace -- --all-targets` after writing queries";

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.starts_with(|c: char| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn write_new(root: &Path, relative: &str, content: &str) -> Result<(), String> {
    let path = root.join(relative);
    if path.exists() {
        return Err(format!("{relative} already exists; refusing to overwrite"));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("{relative}: {error}"))?;
    }
    fs::write(&path, content).map_err(|error| format!("{relative}: {error}"))?;
    println!("created {relative}");
    Ok(())
}

fn migration_name(names: &Names) -> Result<String, String> {
    let stamp = OffsetDateTime::now_utc()
        .format(format_description!(
            "[year][month][day][hour][minute][second]"
        ))
        .map_err(|error| error.to_string())?;
    Ok(format!(
        "crates/db/migrations/postgres/{stamp}_{}.sql",
        names.snake
    ))
}

fn backend_files(names: &Names) -> Result<Vec<(String, String)>, String> {
    let s = &names.snake;
    Ok(vec![
        (
            format!("crates/domain/src/{s}/mod.rs"),
            templates::domain_mod(),
        ),
        (
            format!("crates/domain/src/{s}/model.rs"),
            templates::domain_model(names),
        ),
        (
            format!("crates/domain/src/{s}/service.rs"),
            templates::domain_service(names),
        ),
        (
            format!("crates/domain/src/{s}/errors.rs"),
            templates::domain_errors(names),
        ),
        (
            format!("crates/api/src/routes/{s}.rs"),
            templates::api_route(names),
        ),
        (migration_name(names)?, templates::migration(names)),
    ])
}

fn frontend_files(names: &Names) -> Vec<(String, String)> {
    let (s, p) = (&names.snake, &names.pascal);
    vec![
        (
            format!("web/src/features/{s}/api.ts"),
            templates::web_api(names),
        ),
        (
            format!("web/src/features/{s}/{p}Page.tsx"),
            templates::web_page(names),
        ),
        (
            format!("web/src/features/{s}/index.ts"),
            templates::web_index(names),
        ),
        (format!("web/locales/en/{s}.json"), templates::locale(p)),
        (format!("web/locales/tr/{s}.json"), templates::locale(p)),
    ]
}

pub fn create(root: &Path, name: &str) -> Result<(), String> {
    if !valid_name(name) {
        return Err(format!("`{name}` is not a valid snake_case feature name"));
    }
    let names = Names::from_snake(name);
    let mut files = backend_files(&names)?;
    files.extend(frontend_files(&names));
    if let Some((existing, _)) = files.iter().find(|(path, _)| root.join(path).exists()) {
        return Err(format!("{existing} already exists; refusing to overwrite"));
    }
    for (path, content) in &files {
        write_new(root, path, content)?;
    }
    println!("{}", NEXT_STEPS.replace("{s}", name));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_feature_names() {
        assert!(valid_name("projects"));
        assert!(valid_name("volume_backups"));
        assert!(!valid_name("Projects"));
        assert!(!valid_name("../escape"));
        assert!(!valid_name(""));
    }

    #[test]
    fn derives_name_variants() {
        let names = Names::from_snake("volume_backups");
        assert_eq!(names.pascal, "VolumeBackups");
        assert_eq!(names.kebab, "volume-backups");
    }
}
