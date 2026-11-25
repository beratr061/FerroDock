use std::path::Path;

use ferrodock_core::{AppError, AppResult};

pub const IMAGE_VARIABLE: &str = "FERRODOCK_IMAGE";

pub fn image_of(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        (key.trim() == IMAGE_VARIABLE).then(|| value.trim().to_owned())
    })
}

pub fn with_image(contents: &str, image: &str) -> String {
    let mut replaced = false;
    let mut lines: Vec<String> = contents
        .lines()
        .map(|line| match line.split_once('=') {
            Some((key, _)) if key.trim() == IMAGE_VARIABLE => {
                replaced = true;
                format!("{IMAGE_VARIABLE}={image}")
            }
            _ => line.to_owned(),
        })
        .collect();
    if !replaced {
        lines.push(format!("{IMAGE_VARIABLE}={image}"));
    }
    lines.join("\n") + "\n"
}

pub async fn set_image(path: &Path, image: &str) -> AppResult<Option<String>> {
    let contents = tokio::fs::read_to_string(path)
        .await
        .map_err(AppError::internal)?;
    let previous = image_of(&contents);
    let staging = path.with_extension("env.tmp");
    tokio::fs::write(&staging, with_image(&contents, image))
        .await
        .map_err(AppError::internal)?;
    let permissions = tokio::fs::metadata(path)
        .await
        .map_err(AppError::internal)?
        .permissions();
    tokio::fs::set_permissions(&staging, permissions)
        .await
        .map_err(AppError::internal)?;
    tokio::fs::rename(&staging, path)
        .await
        .map_err(AppError::internal)?;
    Ok(previous)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_or_appends_the_image_line() {
        let env = "FERRODOCK_PANEL_PORT=3100\nFERRODOCK_IMAGE=ghcr.io/x/ferrodock:0.1.1\nFERRODOCK_DOCKER_GID=999\n";
        assert_eq!(image_of(env).as_deref(), Some("ghcr.io/x/ferrodock:0.1.1"));
        let updated = with_image(env, "ghcr.io/x/ferrodock@sha256:abc");
        assert_eq!(
            image_of(&updated).as_deref(),
            Some("ghcr.io/x/ferrodock@sha256:abc")
        );
        assert!(updated.contains("FERRODOCK_DOCKER_GID=999"));
        assert_eq!(updated.matches(IMAGE_VARIABLE).count(), 1);
        let appended = with_image("A=1\n", "img");
        assert_eq!(appended, "A=1\nFERRODOCK_IMAGE=img\n");
    }
}
