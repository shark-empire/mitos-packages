//! Pre-publish consistency check + index refresh for one channel:
//! confirms every metadata entry's archive is actually present on disk
//! and matches its published sha256, then regenerates `index.json` the
//! same way `mitos-generate-index` does. Meant as the final gate
//! `ci/repository.yml` runs before a channel's `repositories/<channel>/`
//! directory is synced anywhere — see `docs/repository-format.md`.

use clap::Parser;
use mitos_packages_toolkit::index::build_index;
use mitos_packages_toolkit::{sign, ToolkitError};
use mitos_pkg::security::checksum;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "mitos-publish-repository")]
struct Args {
    #[arg(long, default_value = "repositories")]
    repo_root: PathBuf,

    /// Which channel to validate and (re-)index.
    #[arg(long)]
    channel: String,

    /// Sign the regenerated index.json, writing index.json.sig alongside
    /// it. Only reached if validation finds no problems.
    #[arg(long)]
    sign_with: Option<PathBuf>,

    /// Stop at the first problem instead of collecting and reporting
    /// every one found.
    #[arg(long)]
    fail_fast: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mitos-publish-repository: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args) -> mitos_packages_toolkit::Result<()> {
    let channel_dir = args.repo_root.join(&args.channel);
    let metadata_dir = channel_dir.join("metadata");
    let packages_dir = channel_dir.join("packages");
    let signatures_dir = channel_dir.join("signatures");
    let index_path = channel_dir.join("index").join("index.json");

    let index = build_index(&metadata_dir)?;
    let mut problems = Vec::new();

    'validate: for versions in index.packages.values() {
        for meta in versions {
            let filename = meta.url.rsplit('/').next().unwrap_or(meta.url.as_str());
            let archive_path = packages_dir.join(filename);

            if !archive_path.exists() {
                problems.push(format!(
                    "{} {}: archive not found at {}",
                    meta.name,
                    meta.version,
                    archive_path.display()
                ));
                if args.fail_fast {
                    break 'validate;
                }
                continue;
            }

            match std::fs::read(&archive_path) {
                Ok(bytes) => {
                    if let Err(e) = checksum::verify(&bytes, &meta.sha256, &meta.name) {
                        problems.push(format!("{} {}: {e}", meta.name, meta.version));
                        if args.fail_fast {
                            break 'validate;
                        }
                    }
                }
                Err(e) => {
                    problems.push(format!(
                        "{} {}: could not read {}: {e}",
                        meta.name,
                        meta.version,
                        archive_path.display()
                    ));
                    if args.fail_fast {
                        break 'validate;
                    }
                }
            }

            if meta.signature.is_some() {
                let sig_filename = format!("{filename}.sig");
                if !signatures_dir.join(&sig_filename).exists() {
                    println!(
                        "mitos-publish-repository: note: {} {} is signed in metadata but {}/{sig_filename} doesn't exist (informational — the signature is already embedded in metadata, this file is just a standalone copy of it)",
                        meta.name,
                        meta.version,
                        signatures_dir.display()
                    );
                }
            }
        }
    }

    let package_count = index.packages.len();
    let version_count: usize = index.packages.values().map(Vec::len).sum();

    if !problems.is_empty() {
        eprintln!(
            "mitos-publish-repository: {} problem(s) found in channel '{}':",
            problems.len(),
            args.channel
        );
        for problem in &problems {
            eprintln!("  - {problem}");
        }
        return Err(ToolkitError::Recipe(format!(
            "channel '{}' failed pre-publish validation",
            args.channel
        )));
    }

    index.save(&index_path)?;
    println!(
        "mitos-publish-repository: channel '{}' OK — {package_count} packages, {version_count} versions, wrote {}",
        args.channel,
        index_path.display()
    );

    if let Some(seed_path) = &args.sign_with {
        let seed = sign::load_seed(seed_path)?;
        let index_bytes = std::fs::read(&index_path)?;
        let signature_hex = sign::sign_index_bytes(&seed, &index_bytes);
        let sig_file_name = format!(
            "{}.sig",
            index_path
                .file_name()
                .expect("index_path always ends in index.json")
                .to_string_lossy()
        );
        let sig_path = index_path.with_file_name(sig_file_name);
        std::fs::write(&sig_path, signature_hex)?;
        println!("mitos-publish-repository: wrote {}", sig_path.display());
    }

    Ok(())
}
