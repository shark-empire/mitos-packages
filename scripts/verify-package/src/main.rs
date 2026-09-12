//! Verifies one built `.mpkg` against its own published metadata JSON
//! and a trusted-keys directory — exactly the check `mitos-pkg` itself
//! runs before installing, usable standalone as a CI/pre-publish gate
//! (see `mitos-publish-repository`, which runs this across a whole
//! channel) or for a maintainer to sanity-check a build locally.

use clap::Parser;
use mitos_packages_toolkit::metadata;
use mitos_pkg::package::archive::read_manifest;
use mitos_pkg::package::signature::verify_package;
use mitos_pkg::security::keys::KeyStore;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "mitos-verify-package")]
struct Args {
    /// The archive to verify.
    #[arg(long)]
    archive: PathBuf,

    /// The archive's published metadata JSON (as written by
    /// mitos-generate-metadata) — carries the expected whole-file
    /// sha256 and signature to check against.
    #[arg(long)]
    metadata: PathBuf,

    /// Directory of trusted `<signer>.pub` files — same format and
    /// meaning as mitos-pkg's own `trusted_keys_dir` config.
    #[arg(long)]
    trusted_keys: PathBuf,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(&args) {
        Ok(()) => {
            println!("mitos-verify-package: OK — {}", args.archive.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("mitos-verify-package: FAILED — {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args) -> mitos_packages_toolkit::Result<()> {
    let meta = metadata::load(&args.metadata)?;
    let manifest = read_manifest(&args.archive)?;
    let keystore = KeyStore::load_dir(&args.trusted_keys)?;

    verify_package(
        &args.archive,
        &meta.sha256,
        &manifest,
        &keystore,
        meta.signature.as_deref(),
    )?;
    Ok(())
}
