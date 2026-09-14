# mitos-packages

Recipes, build tooling, and the served package repositories for MITOS.
If [`mitos-pkg`](https://github.com/shark-empire/mitos-pkg) is the
*client* — the tool that installs, removes, and upgrades packages on a
running system — this is the *supply side*: where a package's source
gets turned into a signed `.mpkg` and published to the `index.json`
`mitos-pkg update` fetches.

Nothing here reinvents `mitos-pkg`'s own archive, index, or signature
formats. Every tool in `scripts/` is a thin Rust binary built on
[`toolkit/`](toolkit), which itself is built directly on `mitos-pkg`'s
own `package`, `repository`, and `security` modules — the same
`Manifest`, `PackageMetadata`, `RepositoryIndex`, checksum, and Ed25519
signing code the install side uses, not a second copy of it. See
`docs/package-format.md` and `docs/repository-format.md` for the exact
shapes, and `toolkit/`'s own doc comments for how the two projects share
code.

## Layout

```
packages/<category>/<name>/recipe.toml   what to build, and how
recipes/<category>/<template>.toml       shared build-step templates (inherits = "...")
repositories/<channel>/                  generated: packages/, metadata/, index/, signatures/
scripts/<tool>/                          the Rust CLI tools that turn one into the other
toolkit/                                 the shared Rust library scripts/ is built on
toolchains/<arch>/                       cross-compilation settings, one day
keys/                                    signing-key *documentation* — never actual keys
.github/workflows/                       the pipeline that runs all of this on every push
docs/                                    the long-form version of everything below
tests/                                   fixtures for toolkit's + scripts'/ integration tests
```

Three channels, under `repositories/`: `stable`, `testing`, `community`
— see [`POLICY.md`](POLICY.md) for what each one means and how a
package moves between them.

## Quickstart: building one package by hand

Everything below assumes you're at the repository root and have `git`,
`cargo`, and (obviously) a working Rust toolchain — the same one
`mitos-pkg` itself targets.

```sh
# Build the tools once.
cargo build --release

BIN=target/release

# Stage 1: clone mitos-init's source, run its build steps, produce a .mpkg.
$BIN/mitos-build-package --recipe packages/core/mitos-init/recipe.toml

# Stage 2 (optional — skip if the package has no `signer` set yet):
# sign the archive's payload hash with a release key.
$BIN/mitos-sign-package \
    --archive repositories/stable/packages/mitos-init-0.1.0.mpkg \
    --sign-with /path/to/seed.hex \
    --out repositories/stable/signatures

# Publish the archive's metadata entry — the download URL, checksum,
# and (if signed) signature that go into the index.
$BIN/mitos-generate-metadata \
    --archive repositories/stable/packages/mitos-init-0.1.0.mpkg \
    --base-url https://packages.mitos-os.org/stable \
    --signature-file repositories/stable/signatures/mitos-init-0.1.0.mpkg.sig \
    --out repositories/stable/metadata

# Fold every package's metadata into index.json (re-run any time; it
# always reflects whatever's currently in metadata/, regardless of what
# produced it).
$BIN/mitos-generate-index --channel stable

# Or, instead of the last step: validate every metadata entry actually
# matches a real archive on disk *and* regenerate the index in one go —
# what .github/workflows/repository.yml actually runs before anything gets published.
$BIN/mitos-publish-repository --channel stable
```

At that point, `repositories/stable/index/index.json` is exactly what a
`mitos-pkg` install pointed at this repository (as a `RepoSource`) would
fetch on `mitos-pkg update`. See
[`docs/maintainer-guide.md`](docs/maintainer-guide.md) for the full
walkthrough, including writing a *new* recipe from scratch, and
[`docs/developer-guide.md`](docs/developer-guide.md) for working on the
tooling itself (`toolkit/`, `scripts/`).

## The tools, briefly

| Tool | What it does |
|---|---|
| `mitos-build-package` | Clones a recipe's source, runs its build steps, produces a `.mpkg`. |
| `mitos-sign-package` | Signs an already-built archive's payload hash — a separate stage from building, deliberately (see `docs/security.md`). |
| `mitos-generate-metadata` | Turns one built archive into its `index.json` entry. |
| `mitos-generate-index` | Folds a channel's metadata into `index.json`, optionally signed. |
| `mitos-verify-package` | Standalone check of an archive against its metadata + a trusted-keys directory. |
| `mitos-publish-repository` | Validates a whole channel, then does what `mitos-generate-index` does — the CI publish gate. |

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for adding or updating a
package, [`POLICY.md`](POLICY.md) for what's required to get into
`stable`, and [`SECURITY.md`](SECURITY.md) to report a vulnerability
rather than filing a public issue for it.
