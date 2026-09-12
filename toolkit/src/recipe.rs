//! `Recipe` is what a maintainer hand-writes at
//! `packages/<category>/<name>/recipe.toml`. It's deliberately a
//! superset of `mitos_pkg::package::spec::PackageSpec` — everything
//! `PackageSpec` needs, plus where to fetch source from and how to build
//! it — so `scripts/build-package` can turn one file into a `pkg.json` +
//! `payload/` for `mitos_pkg::package::build::build_package` to consume,
//! instead of maintainers hand-writing both a recipe and a `pkg.json`
//! that have to be kept in sync by hand.

use crate::error::{Result, ToolkitError};
use mitos_pkg::dependency::version::Dependency;
use mitos_pkg::package::spec::PackageSpec;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Where a recipe's source comes from. Only git today — MITOS components
/// are all git-hosted (see `keys/README.md` and the per-package recipes
/// under `packages/`) — but kept as its own struct rather than a bare
/// URL string so a `path = "..."` (local/vendored source) or `tarball =
/// "..."` variant can be added later without changing every recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub git: String,
    /// Branch, tag, or commit to check out. Defaults to `"main"` rather
    /// than being required, since most recipes track their package's
    /// default branch day to day and only pin a specific tag/commit when
    /// cutting a release — see `docs/maintainer-guide.md`.
    #[serde(default = "default_rev")]
    pub rev: String,
}

fn default_rev() -> String {
    "main".to_string()
}

/// Which repository channel a built package publishes to. Mirrors
/// `repositories/{stable,testing,community}/` — see `POLICY.md` for what
/// promotes a package from one channel to the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    /// New packages and version bumps land here first.
    #[default]
    Testing,
    /// Promoted from `testing` after soak time + passing `ci/package-test.yml`
    /// — see `POLICY.md`.
    Stable,
    /// Community-maintained packages: same format and verification, a
    /// separate trust root (see `keys/README.md`) and no promotion path
    /// to/from `stable`.
    Community,
}

impl Channel {
    pub fn as_str(self) -> &'static str {
        match self {
            Channel::Stable => "stable",
            Channel::Testing => "testing",
            Channel::Community => "community",
        }
    }
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The commands that turn a checked-out source tree into a populated
/// `payload/` directory. Each entry is run with `sh -c`, cwd set to the
/// checked-out source, and `$MITOS_PAYLOAD_DIR` pointing at the
/// (already-created, initially empty) payload directory a step should
/// install its output files into — see `docs/package-format.md`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildSpec {
    /// Informational build-system identifier ("cargo", "meson", "make",
    /// "custom"). Not interpreted by the toolkit itself — its only real
    /// effect is which `recipes/<category>/<system>.toml` template
    /// `inherits` resolves against.
    #[serde(default)]
    pub system: Option<String>,
    /// Left empty when `inherits` is set and the inherited template's
    /// steps are sufficient as-is; a recipe with steps of its own here
    /// always wins over its template's — see `Recipe::resolve`.
    #[serde(default)]
    pub steps: Vec<String>,
}

/// One package's build recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub version: Version,
    pub description: String,
    /// Must match the directory this recipe lives under
    /// (`packages/<category>/<name>/`) — `Recipe::load` checks this, so
    /// a copy-pasted recipe that wasn't updated fails loudly instead of
    /// quietly filing itself under the wrong category's index.
    pub category: String,
    #[serde(default)]
    pub channel: Channel,
    /// Name of a `recipes/<category>/<template>.toml` to pull default
    /// `build.steps` from when this recipe doesn't specify its own —
    /// see `recipes/README.md`.
    #[serde(default)]
    pub inherits: Option<String>,
    pub source: Source,
    #[serde(default)]
    pub build: BuildSpec,
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
    #[serde(default)]
    pub provides: Vec<String>,
    #[serde(default)]
    pub conflicts: Vec<String>,
    /// Signer name expected in `keys/` — see `keys/README.md`. Left
    /// unset, a package still builds and publishes, just unsigned
    /// (`PackageService`/`mitos-pkg` will refuse to install it from a
    /// repository that requires signing — see `docs/security.md`).
    #[serde(default)]
    pub signer: Option<String>,
}

/// A shared, reusable default `BuildSpec` for one build system —
/// `recipes/<category>/<name>.toml`. Individual package recipes set
/// `inherits = "<name>"` instead of repeating the same handful of
/// `cargo build --release` / `install -Dm755 ...` lines across every
/// Rust-based MITOS component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub system: String,
    pub steps: Vec<String>,
}

impl Recipe {
    /// Loads and validates one `recipe.toml`. `category` and `name` are
    /// the directory it was found under (`packages/<category>/<name>/`)
    /// — pass what the caller already knows from the path rather than
    /// re-deriving it here, and this checks the file's own `category`/
    /// `name` fields agree with it.
    pub fn load(path: &Path, expected_category: &str, expected_name: &str) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let recipe: Recipe = toml::from_str(&text)?;

        if recipe.category != expected_category {
            return Err(ToolkitError::Recipe(format!(
                "{}: category '{}' doesn't match its directory (packages/{expected_category}/{expected_name}/)",
                path.display(),
                recipe.category
            )));
        }
        if recipe.name != expected_name {
            return Err(ToolkitError::Recipe(format!(
                "{}: name '{}' doesn't match its directory (packages/{expected_category}/{expected_name}/)",
                path.display(),
                recipe.name
            )));
        }

        Ok(recipe)
    }

    /// Resolves `inherits`, if set and `build.steps` is still empty, by
    /// loading `recipes/<category>/<inherits>.toml` and adopting its
    /// steps (and `build.system`, if this recipe didn't set one either).
    /// A recipe's own `build.steps`, once non-empty, is never
    /// overridden — `inherits` supplies a default, not a forced
    /// override, so a package that needs one extra `patch` step before
    /// the usual cargo build can still write its own full `steps` list
    /// without fighting the template.
    pub fn resolve(mut self, recipes_root: &Path) -> Result<Self> {
        let Some(template_name) = self.inherits.clone() else {
            return Ok(self);
        };
        if !self.build.steps.is_empty() {
            return Ok(self);
        }

        let template_path = recipes_root
            .join(&self.category)
            .join(format!("{template_name}.toml"));
        let text = std::fs::read_to_string(&template_path).map_err(|e| {
            ToolkitError::Recipe(format!(
                "{}: inherits '{template_name}' but {} could not be read: {e}",
                self.name,
                template_path.display()
            ))
        })?;
        let template: Template = toml::from_str(&text)?;

        self.build.steps = template.steps;
        if self.build.system.is_none() {
            self.build.system = Some(template.system);
        }
        Ok(self)
    }

    /// The `PackageSpec` (`pkg.json`) `mitos_pkg::package::build::build_package`
    /// expects at the root of its `source_dir` argument.
    pub fn to_package_spec(&self) -> PackageSpec {
        PackageSpec {
            name: self.name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
            dependencies: self.dependencies.clone(),
            provides: self.provides.clone(),
            conflicts: self.conflicts.clone(),
            signer: self.signer.clone(),
        }
    }

    /// The `.mpkg` filename this recipe builds to, e.g.
    /// `mitos-init-0.1.0.mpkg` — reuses `mitos-pkg`'s own naming
    /// function so the two projects can never disagree on it.
    pub fn archive_filename(&self) -> String {
        mitos_pkg::package::format::package_filename(&self.name, &self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/packages")
            .join(name)
    }

    fn recipes_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../recipes")
    }

    #[test]
    fn loads_a_minimal_recipe() {
        let recipe = Recipe::load(
            &fixture("minimal-valid.recipe.toml"),
            "core",
            "tests-fixture-pkg",
        )
        .expect("minimal recipe should load");
        assert_eq!(recipe.name, "tests-fixture-pkg");
        assert_eq!(recipe.channel, Channel::Testing);
        assert!(
            recipe.build.steps.is_empty(),
            "build.steps should still be empty before resolve() runs"
        );
    }

    #[test]
    fn rejects_a_category_mismatch() {
        let result = Recipe::load(
            &fixture("category-mismatch.recipe.toml"),
            "core",
            "tests-fixture-pkg",
        );
        assert!(
            result.is_err(),
            "a recipe whose category field disagrees with its directory must be rejected"
        );
    }

    #[test]
    fn resolve_pulls_in_the_cargo_template() {
        let recipe = Recipe::load(
            &fixture("minimal-valid.recipe.toml"),
            "core",
            "tests-fixture-pkg",
        )
        .unwrap()
        .resolve(&recipes_root())
        .expect("resolving against the real recipes/core/cargo.toml should succeed");
        assert!(
            !recipe.build.steps.is_empty(),
            "inherits should have populated build.steps from recipes/core/cargo.toml"
        );
        assert_eq!(recipe.build.system.as_deref(), Some("cargo"));
    }

    #[test]
    fn explicit_steps_win_over_inherits() {
        let recipe = Recipe::load(
            &fixture("explicit-steps.recipe.toml"),
            "core",
            "tests-fixture-pkg-explicit",
        )
        .unwrap()
        .resolve(&recipes_root())
        .expect("resolve should be a no-op once build.steps is already set");
        assert_eq!(
            recipe.build.steps,
            vec!["echo this-is-the-explicit-step".to_string()]
        );
    }
}
