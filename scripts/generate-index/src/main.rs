//! Aggregates every `repositories/<channel>/metadata/<name>/<version>.json`
//! into one `repositories/<channel>/index/index.json` — the file
//! `mitos_pkg::service::PackageService::update` actually fetches. Safe
//! to re-run any time; it always reflects whatever metadata currently
//! exists on disk, regardless of which build produced it or when.

use clap::Parser;
use mitos_packages_toolkit::index::build_index;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "mitos-generate-index")]
struct Args {
    /// Root of the repositories/ tree.
    #[arg(long, default_value = "repositories")]
    repo_root: PathBuf,

    /// Which channel to (re-)index: stable, testing, or community.
    #[arg(long)]
    channel: String,

    /// Sign the resulting index.json, writing index.json.sig alongside
    /// it — required if any repository consuming this channel is
    /// configured with a `signer` (see mitos-pkg's `RepoSource::Detailed`
    /// and docs/security.md here).
    #[arg(long)]
    sign_with: Option<PathBuf>,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mitos-generate-index: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args) -> mitos_packages_toolkit::Result<()> {
    let channel_dir = args.repo_root.join(&args.channel);
    let metadata_dir = channel_dir.join("metadata");
    let index_path = channel_dir.join("index").join("index.json");

    let index = build_index(&metadata_dir)?;
    let package_count = index.packages.len();
    let version_count: usize = index.packages.values().map(Vec::len).sum();
    index.save(&index_path)?;
    println!(
        "mitos-generate-index: wrote {} ({package_count} packages, {version_count} versions)",
        index_path.display()
    );

    if let Some(seed_path) = &args.sign_with {
        let seed = mitos_packages_toolkit::sign::load_seed(seed_path)?;
        let index_bytes = std::fs::read(&index_path)?;
        let signature_hex = mitos_packages_toolkit::sign::sign_index_bytes(&seed, &index_bytes);

        // Matches exactly what mitos-pkg's `PackageService::update` fetches
        // for a signed repository: `{index-url}.sig`, i.e. the whole
        // index filename with `.sig` appended — not the extension
        // replaced.
        let sig_file_name = format!(
            "{}.sig",
            index_path
                .file_name()
                .expect("index_path always ends in index.json")
                .to_string_lossy()
        );
        let sig_path = index_path.with_file_name(sig_file_name);
        std::fs::write(&sig_path, signature_hex)?;
        println!("mitos-generate-index: wrote {}", sig_path.display());
    }

    Ok(())
}
