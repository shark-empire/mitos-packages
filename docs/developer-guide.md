# Developer guide

For working on `toolkit/` or `scripts/*` themselves, rather than adding
or updating a package — see `docs/maintainer-guide.md` for that instead.

## Layout recap

```
toolkit/src/
  recipe.rs    Recipe/Template/Channel/Source/BuildSpec — recipe.toml's schema
  metadata.rs  built archive -> PackageMetadata
  index.rs     metadata/ -> RepositoryIndex
  sign.rs      Ed25519 signing, matching mitos-pkg's exact conventions
  error.rs     ToolkitError

scripts/<name>/src/main.rs   one thin CLI binary each, all built on toolkit/
```

Every script is deliberately small — argument parsing plus a `run()`
function that's mostly calls into `toolkit`. If a script's `main.rs` is
growing real logic of its own (not just orchestration), that logic
probably belongs in `toolkit/` instead, where it's testable without
spawning a subprocess.

## Why this depends on `mitos-pkg` as a library

Every format this repository produces — the `.mpkg` archive,
`PackageMetadata`, `RepositoryIndex`, the Ed25519 signing convention —
already has a canonical implementation in `mitos-pkg` (the install
side). `toolkit` imports that crate and calls its functions directly
rather than re-implementing any of it, for one reason: two
implementations of the same format *will* drift apart eventually, and
when they do, whichever one is "wrong" only finds out at install time,
on someone's actual system, in the worst possible way. A change to any
of these formats happens in `mitos-pkg` first; `toolkit` picks it up by
bumping its `mitos-pkg` git dependency, not by hand-porting the change.

## Building and testing

```sh
cargo build --workspace
cargo test --workspace
```

`cargo test` currently exercises `toolkit`'s pure functions
(`recipe::Recipe::resolve`, `index::build_index`, the `sign` module)
against fixtures under `tests/` (see `tests/README.md`) rather than
running the CLI binaries end to end — an end-to-end test needs a real
git clone and a real build, which is what `ci/package-build.yml`
actually does against real recipes, not something worth mocking out
in-repo.

## Adding a new script

1. `mkdir -p scripts/<name>/src`.
2. `scripts/<name>/Cargo.toml` — copy an existing one's shape (package
   name `mitos-<name>`, depends on `mitos-packages-toolkit.workspace = true`
   and whatever else it actually calls directly).
3. Add `"scripts/<name>"` to the root `Cargo.toml`'s `[workspace]`
   `members`.
4. Keep `main()` to argument parsing + calling `run(&args) -> mitos_packages_toolkit::Result<()>`,
   printing the error and returning `ExitCode::FAILURE` on `Err` — every
   existing script follows this shape; a new one that doesn't makes the
   set inconsistent for no benefit.

## Adding a new toolkit module

Ask first whether it's really toolkit's concern or `mitos-pkg`'s — if
it's about the archive/index/signature *format itself*, it almost
certainly belongs upstream in `mitos-pkg`, with `toolkit` just calling
the new function once it exists there. `toolkit` is for logic specific
to *producing content in that format from a recipe* (parsing recipes,
folding metadata into an index, orchestrating a build) — not a place to
extend the format itself.
