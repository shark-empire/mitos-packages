# tests/

Fixtures for `toolkit`'s unit tests (`cargo test --workspace`, run by
`ci/package-test.yml`) — not a test runner of its own, and not a place
generated output ever gets written to. Everything under here is
committed, hand-written, and small on purpose.

| Directory | Used by | Status |
|---|---|---|
| `packages/` | `toolkit::recipe` tests | Exercised today — see `toolkit/src/recipe.rs`'s `#[cfg(test)] mod tests`. |
| `metadata/` | `toolkit::metadata` tests | Exercised today. |
| `repositories/` | `toolkit::index` tests | Exercised today. |
| `signatures/` | `toolkit::sign` tests | Exercised today. Contains `test-only-seed.hex` — **32 zero bytes, hex-encoded, not a real key** — see the warning in that file and in `toolkit/src/sign.rs`'s tests. Never use it for anything but a test fixture. |
| `dependency-resolution/` | Not exercised by any test yet | See below. |

## Naming fixtures

`tests-fixture-*` for anything that needs a package name (avoids ever
colliding with a real package name under `packages/` at the repository
root). None of these fixtures point at a real git repository or a real
signing key — `source.git` values like `https://example.invalid/...`
are deliberate (`.invalid` is the TLD reserved by RFC 2606 specifically
for addresses that are guaranteed not to resolve).

## `dependency-resolution/`

Deliberately empty of active fixtures right now, for an honest reason:
dependency *resolution* is `mitos-pkg`'s job (`dependency::resolver::Resolver`),
not this repository's — `toolkit` only reads a recipe's declared
`dependencies` list and copies it into `pkg.json`; it never resolves a
dependency graph itself. This directory exists for two things once
they're built out, neither of which exists yet:

1. Scenario fixtures (diamond dependencies, a version conflict, a
   `provides`-satisfied dependency) that a *future* integration test
   could feed to a temporary `mitos-pkg` install pointed at a fixture
   repository built from this repo's own tooling — closing the loop
   between "this repository's recipes declare dependencies correctly"
   and "mitos-pkg actually resolves them the way the recipe author
   expected."
2. A place for a maintainer to manually sanity-check a new package's
   declared `dependencies` against a known-tricky scenario before
   opening a PR, per `docs/package-policy.md`'s under-declaration
   warning — usable today even without an automated test consuming it.

Tracked as follow-up work in `docs/developer-guide.md`, not silently
assumed to already be covered.
