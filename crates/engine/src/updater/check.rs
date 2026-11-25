use semver::Version;
use serde::Serialize;

use super::manifest::ReleaseManifest;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum UpdateStatus {
    UpToDate {
        current: Version,
    },
    Available {
        current: Version,
        latest: Version,
        image: String,
        notes_url: String,
    },
    RequiresIntermediate {
        current: Version,
        latest: Version,
        min_from_version: Version,
    },
}

pub fn evaluate(current: &Version, manifest: &ReleaseManifest) -> UpdateStatus {
    if manifest.version <= *current {
        return UpdateStatus::UpToDate {
            current: current.clone(),
        };
    }
    if *current < manifest.min_from_version {
        return UpdateStatus::RequiresIntermediate {
            current: current.clone(),
            latest: manifest.version.clone(),
            min_from_version: manifest.min_from_version.clone(),
        };
    }
    UpdateStatus::Available {
        current: current.clone(),
        latest: manifest.version.clone(),
        image: manifest.pinned_image(),
        notes_url: manifest.notes_url.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::updater::manifest::tests::sample;

    #[test]
    fn classifies_versions() {
        let manifest = sample("0.3.0");
        assert!(matches!(
            evaluate(&Version::new(0, 3, 0), &manifest),
            UpdateStatus::UpToDate { .. }
        ));
        assert!(matches!(
            evaluate(&Version::new(0, 4, 0), &manifest),
            UpdateStatus::UpToDate { .. }
        ));
        let available = evaluate(&Version::new(0, 2, 1), &manifest);
        assert!(
            matches!(&available, UpdateStatus::Available { image, .. } if image.contains("@sha256:"))
        );
        let old = evaluate(&Version::new(0, 0, 9), &manifest);
        assert!(matches!(old, UpdateStatus::RequiresIntermediate { .. }));
    }
}
