//! Turns an already-built `.mpkg` archive into the
//! `mitos_pkg::repository::metadata::PackageMetadata` entry that gets
//! aggregated into a channel's `index.json` (see `index.rs`) — one JSON
//! file per package version under `repositories/<channel>/metadata/`.

use crate::error::Result;
use mitos_pkg::package::manifest::Manifest;
use mitos_pkg::repository::metadata::PackageMetadata;
use mitos_pkg::security::checksum;
use std::path::Path;

/// Builds a `PackageMetadata` for one archive already on disk.
///
/// `url` is wherever this exact file will be served from once published
/// — the toolkit has no opinion on that (it depends on the channel's
/// real base URL, which only the caller/CI config knows), so callers
/// build it themselves, typically `{base_url}/packages/{filename}`.
///
/// `signature_hex`, if given, is exactly what
/// `mitos_pkg::package::signature::verify_package` will check against —
/// see `sign.rs::sign_package_payload` for how it's produced.
pub fn build_metadata(
    archive_path: &Path,
    manifest: &Manifest,
    url: String,
    signature_hex: Option<String>,
) -> Result<PackageMetadata> {
    let sha256 = checksum::hash_file(archive_path)?;
    let size_bytes = std::fs::metadata(archive_path)?.len();

    Ok(PackageMetadata {
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        description: manifest.description.clone(),
        dependencies: manifest.dependencies.clone(),
        provides: manifest.provides.clone(),
        conflicts: manifest.conflicts.clone(),
        url,
        sha256,
        signature: signature_hex,
        size_bytes,
        arch: vec![String::from("any")],              // or "x86_64", "aarch64", etc.
        essential: false,
        installed_size_bytes: 0, 
        priority: String::from("0"),
    })
}

/// Writes one `PackageMetadata` to
/// `repositories/<channel>/metadata/<name>/<version>.json`, creating
/// parent directories as needed.
pub fn save(metadata: &PackageMetadata, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_vec_pretty(metadata)?)?;
    Ok(())
}

pub fn load(path: &Path) -> Result<PackageMetadata> {
    let text = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

/// The canonical path one package version's metadata file lives at,
/// relative to a channel's `metadata/` directory — kept in one place so
/// every tool that writes or reads it (`generate-metadata`,
/// `generate-index`, `publish-repository`) agrees on the layout.
pub fn metadata_path(metadata_dir: &Path, name: &str, version: &semver::Version) -> std::path::PathBuf {
    metadata_dir.join(name).join(format!("{version}.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/metadata/sample.json")
    }

    #[test]
    fn loads_the_sample_fixture() {
        let meta = load(&fixture_path()).expect("sample.json should load");
        assert_eq!(meta.name, "tests-fixture-pkg");
        assert_eq!(meta.sha256.len(), 64);
    }

    #[test]
    fn save_then_load_round_trips() {
        let meta = load(&fixture_path()).unwrap();
        let dir = std::env::temp_dir().join(format!(
            "mitos-packages-toolkit-test-{}",
            std::process::id()
        ));
        let out_path = metadata_path(&dir, &meta.name, &meta.version);

        save(&meta, &out_path).expect("save should succeed");
        let reloaded = load(&out_path).expect("reload should succeed");

        assert_eq!(reloaded.name, meta.name);
        assert_eq!(reloaded.version, meta.version);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
