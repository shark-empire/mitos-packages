# Reproducible builds

What "reproducible" means for a package built through this repository,
what actually holds today, and what's an open gap — written plainly
rather than claiming more than the tooling currently guarantees.

## What's pinned today

- **Source**: `recipe.toml`'s `source.rev`. A `rev` set to a specific
  commit or tag is fully pinned; a `rev` tracking a branch (`"main"`,
  the default) is *not* — re-running `mitos-build-package` tomorrow
  against a branch-tracking recipe can produce a different result if
  upstream has pushed since. See `POLICY.md`'s promotion criteria: a
  package being promoted to `stable` is exactly the point at which its
  `rev` should move from a branch to the specific commit/tag being
  promoted, for this reason.
- **Rust toolchain**: whatever `cargo`/`rustc` the build machine has
  installed — not currently pinned by this repository at all (no
  `rust-toolchain.toml` at the workspace root, and none is generated
  for a package's own build either). This is a real, open gap: the same
  recipe built with two different Rust versions is not guaranteed to
  produce byte-identical output, even if it produces *working* output
  both times.
- **Dependencies**: `cargo install --locked` (in every `recipes/*/cargo.toml`
  template) pins to whatever `Cargo.lock` the upstream source ships
  with — reproducible *given* that lockfile, but only as reproducible as
  the upstream project's own lockfile discipline.

## What "byte-identical" would additionally require

Full bit-for-bit reproducibility (build the same recipe twice, get the
identical `.mpkg`, checksum and all) needs, beyond the above:
build-timestamp elimination (tar entries and any embedded build-time
constants need a fixed timestamp, not "now"), a pinned Rust toolchain
version (not just a pinned `Cargo.lock`), and a normalized build
environment (locale, `$PATH`, and similar details that can leak into a
binary's embedded strings or debug info). None of this is implemented
yet — `mitos-build-package` does not currently pass any timestamp-
normalization or environment-scrubbing flags to `cargo`. Tracked as
future work, not silently assumed to already work.

## Why this matters for dependency resolution specifically

A recipe's declared `dependencies` (see `docs/package-policy.md`'s
under-declaration section) is only trustworthy if building the same
recipe reliably produces a binary that needs exactly that dependency
set — every time. A branch-tracking `rev` that silently gains a new
dependency upstream, without the recipe's `dependencies` list being
updated to match, is the concrete failure mode reproducibility (or the
lack of it) turns into a real, user-facing bug: `mitos-pkg install`
succeeding, then failing at runtime for a dependency nothing declared.
`tests/dependency-resolution/` includes fixtures for exactly this class
of scenario — see `tests/README.md`.

## Practical guidance until the gaps above close

- Pin `source.rev` to a commit or tag before promoting anything to
  `stable` (not just "should," per `POLICY.md` — the promotion PR
  review should treat a branch-tracking `rev` on a promotion candidate
  as a blocking issue).
- Re-run a build and diff the resulting `payload_sha256` (not the whole
  `.mpkg`, which will differ on timestamps even for identical payloads
  today) when in doubt about whether a "no source changes" rebuild
  actually produced the same thing.
