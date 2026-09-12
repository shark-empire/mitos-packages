use thiserror::Error;

/// Every error the toolkit and the tools built on it can produce.
/// Deliberately thin: most of the interesting error information already
/// comes from `mitos_pkg::error::PkgError`'s own well-formatted messages
/// (checksum mismatches, invalid manifests, untrusted signers, ...) —
/// this type just adds the handful of failure modes specific to
/// *building* mitos-packages content rather than *consuming* it.
#[derive(Debug, Error)]
pub enum ToolkitError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Pkg(#[from] mitos_pkg::error::PkgError),

    /// A recipe that doesn't parse cleanly into a buildable package —
    /// e.g. an `inherits` template that doesn't exist, or a build step
    /// that exited non-zero. Carries a ready-to-print message rather
    /// than structured fields since these are always one-off, specific
    /// to the recipe file at hand, and never matched on programmatically
    /// (nothing here needs to distinguish *which* recipe error occurred,
    /// only report it).
    #[error("{0}")]
    Recipe(String),
}

pub type Result<T> = std::result::Result<T, ToolkitError>;
