//! Stage one of the two-stage build pipeline (see `scripts/sign-package`
//! for stage two, and `docs/maintainer-guide.md` for the full picture):
//! clone a recipe's source, run its build steps into a staged
//! `payload/`, and hand the result to `mitos_pkg::package::build::build_package`
//! to produce a `.mpkg`. Deliberately does *not* sign by default — a
//! build worker that only ever runs `git clone` + a build script from a
//! recipe file shouldn't also need access to a production signing key.

use clap::Parser;
use mitos_packages_toolkit::{Recipe, ToolkitError};
use mitos_pkg::package::build::build_package;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// Fetches a recipe's source over git, runs its build steps, and
/// packages the result into a `.mpkg`. Run from the repository root —
/// `--recipes-root` and `--repo-root` default to `recipes/` and
/// `repositories/` relative to the current directory.
#[derive(Parser)]
#[command(name = "mitos-build-package")]
struct Args {
    /// Path to the package's recipe (packages/<category>/<name>/recipe.toml).
    /// <category> and <name> are read from this path, not re-specified.
    #[arg(long)]
    recipe: PathBuf,

    /// Root of the recipes/ tree, for resolving `inherits` templates.
    #[arg(long, default_value = "recipes")]
    recipes_root: PathBuf,

    /// Where to write the built .mpkg. Defaults to
    /// <repo-root>/<channel>/packages — wherever the recipe's own
    /// `channel` field says it belongs.
    #[arg(long)]
    out: Option<PathBuf>,

    /// Root of the repositories/ tree, used to derive --out when it
    /// isn't given explicitly.
    #[arg(long, default_value = "repositories")]
    repo_root: PathBuf,

    /// Scratch directory for the git checkout and staged payload/.
    /// Defaults to target/mitos-packages-build/<name>.
    #[arg(long)]
    workdir: Option<PathBuf>,

    /// Reuse an existing checkout under <workdir>/src instead of
    /// cloning fresh — useful for iterating on build steps locally
    /// without re-cloning every run.
    #[arg(long)]
    skip_fetch: bool,

    /// Also sign the built archive's payload hash with this key file,
    /// printing the signature. Most CI pipelines should leave this
    /// unset and run `mitos-sign-package` as a separate stage instead —
    /// see docs/security.md for why build and sign are split.
    #[arg(long)]
    sign_with: Option<PathBuf>,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mitos-build-package: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args) -> mitos_packages_toolkit::Result<()> {
    let (category, name) = category_and_name(&args.recipe)?;
    let recipe = Recipe::load(&args.recipe, &category, &name)?.resolve(&args.recipes_root)?;

    let workdir = args
        .workdir
        .clone()
        .unwrap_or_else(|| PathBuf::from("target/mitos-packages-build").join(&recipe.name));
    let src_dir = workdir.join("src");
    let stage_dir = workdir.join("stage");
    let payload_dir = stage_dir.join("payload");

    if !(args.skip_fetch && src_dir.exists()) {
        fetch_source(&recipe, &src_dir)?;
    } else {
        println!(
            "mitos-build-package: reusing existing checkout at {}",
            src_dir.display()
        );
    }

    let _ = std::fs::remove_dir_all(&payload_dir);
    std::fs::create_dir_all(&payload_dir)?;
    run_build_steps(&recipe, &src_dir, &payload_dir)?;

    // build_package expects pkg.json as a *sibling* of payload/, both
    // directly under one source_dir — that's stage_dir here, kept
    // separate from src_dir (the git checkout) so build steps never
    // have to know or care where their output ends up being packaged
    // from, only where $MITOS_PAYLOAD_DIR points.
    let spec = recipe.to_package_spec();
    std::fs::write(stage_dir.join("pkg.json"), serde_json::to_vec_pretty(&spec)?)?;

    let out_dir = args
        .out
        .clone()
        .unwrap_or_else(|| args.repo_root.join(recipe.channel.as_str()).join("packages"));

    let sign_seed = match &args.sign_with {
        Some(path) => Some(mitos_packages_toolkit::sign::load_seed(path)?),
        None => None,
    };

    let output = build_package(&stage_dir, &out_dir, sign_seed.as_ref())?;

    println!(
        "mitos-build-package: built {} (payload sha256 {})",
        output.archive_path.display(),
        output.manifest.payload_sha256
    );
    if let Some(sig) = &output.signature_hex {
        println!("mitos-build-package: signature: {sig}");
        println!(
            "mitos-build-package: publish this via `mitos-sign-package` (or by hand) under {}/signatures/",
            args.repo_root.join(recipe.channel.as_str()).display()
        );
    }

    Ok(())
}

/// Recovers `(category, name)` from a `packages/<category>/<name>/recipe.toml`
/// path so they don't have to be passed redundantly on the command line.
fn category_and_name(recipe_path: &Path) -> mitos_packages_toolkit::Result<(String, String)> {
    let recipe_dir = recipe_path.parent().ok_or_else(|| {
        ToolkitError::Recipe(format!(
            "{}: expected packages/<category>/<name>/recipe.toml",
            recipe_path.display()
        ))
    })?;
    let name = recipe_dir
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            ToolkitError::Recipe(format!(
                "{}: cannot determine package name from path",
                recipe_path.display()
            ))
        })?
        .to_string();
    let category = recipe_dir
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            ToolkitError::Recipe(format!(
                "{}: cannot determine category from path",
                recipe_path.display()
            ))
        })?
        .to_string();
    Ok((category, name))
}

fn fetch_source(recipe: &Recipe, src_dir: &Path) -> mitos_packages_toolkit::Result<()> {
    if src_dir.exists() {
        std::fs::remove_dir_all(src_dir)?;
    }
    if let Some(parent) = src_dir.parent() {
        std::fs::create_dir_all(parent)?;
    }

    println!(
        "mitos-build-package: cloning {} @ {} into {}",
        recipe.source.git,
        recipe.source.rev,
        src_dir.display()
    );

    let status = Command::new("git")
        .args(["clone", "--depth", "1", "--branch", &recipe.source.rev])
        .arg(&recipe.source.git)
        .arg(src_dir)
        .status()?;

    if !status.success() {
        return Err(ToolkitError::Recipe(format!(
            "git clone of {} failed ({status})",
            recipe.source.git
        )));
    }
    Ok(())
}

fn run_build_steps(
    recipe: &Recipe,
    src_dir: &Path,
    payload_dir: &Path,
) -> mitos_packages_toolkit::Result<()> {
    if recipe.build.steps.is_empty() {
        return Err(ToolkitError::Recipe(format!(
            "{}: no build steps — set build.steps directly or inherits = \"<template>\"",
            recipe.name
        )));
    }

    for step in &recipe.build.steps {
        println!("mitos-build-package: $ {step}");
        let status = Command::new("sh")
            .arg("-c")
            .arg(step)
            .current_dir(src_dir)
            .env("MITOS_PAYLOAD_DIR", payload_dir)
            .status()?;
        if !status.success() {
            return Err(ToolkitError::Recipe(format!(
                "{}: build step failed ({status}): {step}",
                recipe.name
            )));
        }
    }
    Ok(())
}
