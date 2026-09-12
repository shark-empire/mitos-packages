# Contributing

## Adding a new package

1. Pick a category (`core`, `system`, `desktop`, `development`, or
   `applications` — see each `packages/<category>/` for what's already
   there; `applications/` is for anything that isn't a MITOS component
   itself).
2. `mkdir -p packages/<category>/<name>` and write `recipe.toml` there —
   see `docs/package-format.md` for every field, or copy a similar
   existing recipe as a starting point (most are a dozen lines).
3. Most packages can just set `inherits = "cargo"` (or `"cargo-lib"` for
   a library-only crate) rather than writing `build.steps` from scratch
   — see `recipes/README.md`.
4. Build it locally and sanity-check the result before opening a PR:
   ```sh
   cargo build --release
   target/release/mitos-build-package --recipe packages/<category>/<name>/recipe.toml
   ```
5. New packages start on the `testing` channel (`recipe.toml`'s
   `channel` field) — see `POLICY.md` for what promotes a package to
   `stable`.
6. Open a PR. `ci/package-build.yml` builds it and `ci/package-test.yml`
   runs the dependency-resolution and format checks under `tests/`
   against it automatically.

## Updating an existing package (a version bump)

Bump `version` in its `recipe.toml`, update `source.rev` if it should
track a new tag/commit, and open a PR the same way — CI builds both the
old and new version and confirms `mitos-pkg upgrade` would actually
consider it an upgrade (`docs/reproducible-builds.md` covers the
version-monotonicity check this relies on).

## Working on the tooling itself

`toolkit/` and `scripts/*` are a normal Cargo workspace:

```sh
cargo build
cargo test --workspace
```

`tests/` holds the fixtures those tests run against (sample recipes,
metadata, and known dependency-resolution scenarios) rather than
generated content — see `tests/README.md`. A change to `toolkit/` that
alters the on-disk recipe/metadata/index format needs a matching update
to `docs/package-format.md` and/or `docs/repository-format.md` in the
same PR; those documents are the contract every recipe author and every
`mitos-pkg` install depends on, not just a description of what the code
currently happens to do.

## Commit / PR conventions

- One package addition or version bump per PR — keeps `ci/package-build.yml`'s
  failure attributable to one thing.
- Reference the upstream commit/tag a version bump tracks in the PR
  description, not just in `recipe.toml`'s `source.rev`.
- Tooling changes (`toolkit/`, `scripts/`) that touch signing or
  verification code get an extra pass — see `SECURITY.md` and
  `docs/security.md` before opening that kind of PR, and consider
  whether it should go through a private report instead if it's fixing
  a real vulnerability rather than hardening in general.

## Maintainer / developer guides

For anything longer than fits here: `docs/maintainer-guide.md` (writing
recipes, requesting a community signer) and `docs/developer-guide.md`
(working on `toolkit/`/`scripts/`, the test fixtures under `tests/`, CI).
