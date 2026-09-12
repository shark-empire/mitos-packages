//! Thin wrappers around `mitos_pkg::security::signature`'s Ed25519
//! primitives, matching exactly what `mitos_pkg::package::signature::
//! verify_package` and `verify_index` check on the install side — a
//! signature produced here needs to be byte-for-byte what those
//! functions expect, so this deliberately doesn't add any convention of
//! its own.

use crate::error::{Result, ToolkitError};
use mitos_pkg::security::{checksum, signature};
use std::path::Path;

/// Reads a 32-byte Ed25519 seed from a hex-encoded text file — the same
/// convention `mitos-pkg build --sign-with` uses (see that project's
/// `main.rs::load_seed`), so one key file works unmodified with both
/// tools. Generate one with e.g. `openssl rand -hex 32 > seed.hex` —
/// signing is deterministic and needs no RNG at sign time, only when the
/// seed itself is first created.
pub fn load_seed(path: &Path) -> Result<[u8; 32]> {
    let hex_str = std::fs::read_to_string(path)?;
    let bytes = hex::decode(hex_str.trim()).map_err(|e| {
        ToolkitError::Recipe(format!("seed at {} is not valid hex: {e}", path.display()))
    })?;
    bytes.try_into().map_err(|_| {
        ToolkitError::Recipe(format!(
            "seed at {} must be exactly 32 bytes",
            path.display()
        ))
    })
}

/// Signs `message` with `seed`, hex-encoding the resulting 64-byte
/// signature — the exact encoding published in
/// `PackageMetadata::signature` and in a channel's `index.json.sig`.
pub fn sign_hex(seed: &[u8; 32], message: &[u8]) -> String {
    hex::encode(signature::sign(seed, message))
}

/// Signs a package's `payload_sha256` — exactly the message
/// `package::signature::verify_package` checks a package's signature
/// against (see `manifest.rs::Manifest::payload_sha256`'s docs in
/// mitos-pkg for why that value, specifically, is what gets signed
/// rather than the whole archive's bytes).
pub fn sign_package_payload(seed: &[u8; 32], payload_sha256: &str) -> String {
    sign_hex(seed, payload_sha256.as_bytes())
}

/// Signs a repository index's raw file bytes — exactly what
/// `package::signature::verify_index` checks a `{index-url}.sig` against:
/// not the bytes themselves, but the hex SHA-256 digest of them.
pub fn sign_index_bytes(seed: &[u8; 32], index_json: &[u8]) -> String {
    let digest = checksum::hash_bytes(index_json);
    sign_hex(seed, digest.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `tests/signatures/test-only-seed.hex` is 32 zero bytes, hex
    /// encoded — a fixture, never a real key. See `tests/README.md`.
    fn test_seed_path() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/signatures/test-only-seed.hex")
    }

    #[test]
    fn load_seed_reads_exactly_32_bytes() {
        let seed = load_seed(&test_seed_path()).expect("test-only-seed.hex should load");
        assert_eq!(seed, [0u8; 32]);
    }

    #[test]
    fn signing_is_deterministic() {
        let seed = load_seed(&test_seed_path()).unwrap();
        let a = sign_hex(&seed, b"some message");
        let b = sign_hex(&seed, b"some message");
        assert_eq!(
            a, b,
            "signing the same message with the same seed must be deterministic — Ed25519 has no randomness at sign time"
        );
        assert_eq!(
            a.len(),
            128,
            "a hex-encoded 64-byte Ed25519 signature is 128 hex characters"
        );
    }

    #[test]
    fn different_messages_sign_differently() {
        let seed = load_seed(&test_seed_path()).unwrap();
        let a = sign_package_payload(&seed, "aaaa");
        let b = sign_package_payload(&seed, "bbbb");
        assert_ne!(a, b);
    }
}
