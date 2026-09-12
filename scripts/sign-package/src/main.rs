//! Stage two of the build pipeline (see `scripts/build-package` for
//! stage one): signs an already-built `.mpkg`'s `payload_sha256` (read
//! from its embedded manifest, never recomputed here — the manifest
//! *is* the thing being attested to) and writes the hex signature to
//! `<out>/<name>-<version>.mpkg.sig`. Kept as its own stage, separate
//! from the build itself, so a build worker never needs access to
//! whatever key ends up on the signature — see docs/security.md.

use clap::Parser;
use mitos_packages_toolkit::{sign, ToolkitError};
use mitos_pkg::package::archive::read_manifest;
use mitos_pkg::package::format::package_filename;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "mitos-sign-package")]
struct Args {
    /// The built archive to sign.
    #[arg(long)]
    archive: PathBuf,

    /// Hex-encoded 32-byte Ed25519 seed file — same convention as
    /// `mitos-pkg build --sign-with`.
    #[arg(long)]
    sign_with: PathBuf,

    /// Directory to write <name>-<version>.mpkg.sig into — normally
    /// repositories/<channel>/signatures.
    #[arg(long)]
    out: PathBuf,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mitos-sign-package: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args) -> mitos_packages_toolkit::Result<()> {
    let manifest = read_manifest(&args.archive)?;

    let Some(signer) = &manifest.signer else {
        return Err(ToolkitError::Recipe(format!(
            "{}: manifest has no \"signer\" set — nothing names who this signature should be attributed to",
            args.archive.display()
        )));
    };

    let seed = sign::load_seed(&args.sign_with)?;
    let signature_hex = sign::sign_package_payload(&seed, &manifest.payload_sha256);

    std::fs::create_dir_all(&args.out)?;
    let sig_filename = format!("{}.sig", package_filename(&manifest.name, &manifest.version));
    let sig_path = args.out.join(&sig_filename);
    std::fs::write(&sig_path, signature_hex)?;

    println!(
        "mitos-sign-package: signed {} for signer '{signer}' -> {}",
        args.archive.display(),
        sig_path.display()
    );
    Ok(())
}
