//! Aggregates every `repositories/<channel>/metadata/<name>/<version>.json`
//! into one `mitos_pkg::repository::index::RepositoryIndex` — the exact
//! shape `mitos_pkg::service::PackageService::update` fetches and caches
//! locally on the install side. Building this by folding individually
//! written metadata files (rather than, say, a single script that both
//! builds *and* indexes in one step) is deliberate: it lets
//! `generate-index` be re-run at any time to reflect whatever metadata
//! already exists on disk, independent of which build produced it or
//! when — see `scripts/publish-repository` for the tool that ties a
//! full re-index into a pre-publish consistency check.

use crate::error::Result;
use mitos_pkg::repository::index::RepositoryIndex;
use mitos_pkg::repository::metadata::PackageMetadata;
use std::path::Path;

/// Walks every `*.json` file under `metadata_dir` and folds it into a
/// `RepositoryIndex`, sorting each package's versions ascending
/// (`RepositoryIndex` itself doesn't require any particular order, but a
/// stable order keeps `index.json` diffs in version control readable and
/// makes `latest()` trivially correct instead of relying on `max_by_key`
/// over hopefully-well-formed input).
pub fn build_index(metadata_dir: &Path) -> Result<RepositoryIndex> {
    let mut index = RepositoryIndex::default();
    if !metadata_dir.exists() {
        return Ok(index);
    }

    for entry in walkdir::WalkDir::new(metadata_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let text = std::fs::read_to_string(path)?;
        let meta: PackageMetadata = serde_json::from_str(&text)?;
        index
            .packages
            .entry(meta.name.clone())
            .or_default()
            .push(meta);
    }

    for versions in index.packages.values_mut() {
        versions.sort_by(|a, b| a.version.cmp(&b.version));
    }

    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_metadata_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/repositories/sample-metadata")
    }

    #[test]
    fn folds_multiple_versions_of_one_package() {
        let index =
            build_index(&fixture_metadata_dir()).expect("fixture metadata should build cleanly");
        let versions = index
            .packages
            .get("tests-fixture-pkg")
            .expect("fixture package should be present");
        assert_eq!(versions.len(), 2);
        assert!(
            versions[0].version < versions[1].version,
            "versions should be sorted ascending"
        );
    }

    #[test]
    fn missing_metadata_dir_yields_an_empty_index() {
        let index =
            build_index(Path::new("/does/not/exist")).expect("a missing directory is not an error");
        assert!(index.packages.is_empty());
    }
}
