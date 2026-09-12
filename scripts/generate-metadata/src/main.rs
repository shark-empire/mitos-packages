//! Turns one already-built `.mpkg` into its `PackageMetadata` JSON entry
//! under `repositories/<channel>/metadata/<name>/<version>.json`. Run
//! after `mitos-build-package` and (if the package is signed)
//! `mitos-sign-package`, before `mitos-generate-index`.

use clap::Parser;
use mitos_packages_toolkit::{metadata, ToolkitError};
use mitos_pkg::package::archive::read_manifest;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "mitos-generate-metadata")]
struct Args {
    /// The built archive to describe.
    #[arg(long)]
    archive: PathBuf,

    /// Base URL this archive will be served from once published, e.g.
    /// `https://packages.mitos-os.org/stable`. The full download URL
    /// written into metadata is `<base-url>/packages/<archive filename>`.
    #[arg(long)]
    base_url: String,

    /// Detached signature file for this archive (as written by
    /// `mitos-sign-package`), if it's signed. Its contents are copied
    /// verbatim into the metadata's `signature` field.
    #[arg(long)]
    signature_file: Option<PathBuf>,

    /// Directory to write <name>/<version>.json into — normally
    /// repositories/<channel>/metadata.
    #[arg(long)]
    out: PathBuf,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mitos-generate-metadata: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args) -> mitos_packages_toolkit::Result<()> {
    let manifest = read_manifest(&args.archive)?;

    let filename = args
        .archive
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| {
            ToolkitError::Recipe(format!(
                "{}: not a valid archive filename",
                args.archive.display()
            ))
        })?;
    let url = format!(
        "{}/packages/{filename}",
        args.base_url.trim_end_matches('/')
    );

    let signature_hex = match &args.signature_file {
        Some(path) => Some(std::fs::read_to_string(path)?.trim().to_string()),
        None => None,
    };

    let meta = metadata::build_metadata(&args.archive, &manifest, url, signature_hex)?;
    let out_path = metadata::metadata_path(&args.out, &meta.name, &meta.version);
    metadata::save(&meta, &out_path)?;

    println!("mitos-generate-metadata: wrote {}", out_path.display());
    Ok(())
}
