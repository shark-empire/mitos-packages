//! Shared logic for mitos-packages' build tooling (the crates under
//! `scripts/`): parsing recipes, turning a built archive into metadata,
//! folding metadata into a repository index, and signing — all of it
//! built directly on `mitos-pkg`'s own types and functions
//! (`mitos_pkg::package`, `mitos_pkg::repository`, `mitos_pkg::security`)
//! rather than a second implementation of the archive/index/signature
//! formats that could drift out of sync with what `mitos-pkg` itself
//! reads. See `docs/repository-format.md` and `docs/package-format.md`
//! for the on-disk shapes these types round-trip.

pub mod error;
pub mod index;
pub mod metadata;
pub mod recipe;
pub mod sign;

pub use error::{Result, ToolkitError};
pub use recipe::{BuildSpec, Channel, Recipe, Source, Template};
