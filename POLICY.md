# Packaging Policy

Short version of [`docs/package-policy.md`](docs/package-policy.md),
which has the full detail — this page is what to skim before opening a
PR, that one is what to read before disputing a rejected one.

## Channels

| Channel | What it means | Who can publish to it |
|---|---|---|
| `stable` | Built, signed, soaked in `testing`, no known regressions. What `mitos-pkg`'s default repository config points at. | Maintainers, after promotion (below). |
| `testing` | Builds and passes `.github/workflows/package-test.yml`. Where every new package and version bump starts. | Any accepted PR. |
| `community` | Same build/verification pipeline, separate trust root (see `keys/README.md`) — not held to the same review bar as `core`/`system`/`desktop`/`development`, and never auto-promoted to `stable`. | Package maintainers registered for a community signer name. |

## Promotion: `testing` → `stable`

A package promotes when, since it last changed:

1. It's built successfully in `.github/workflows/package-build.yml` on every architecture
   `toolchains/` currently lists as in-CI (`x86_64` only as of this
   writing — see `toolchains/README.md`).
2. It's sat in `testing` for at least 7 days with no regression reports
   against it.
3. It's signed (`recipe.toml` has a `signer` set and
   `mitos-sign-package` has actually been run — an unsigned package
   doesn't promote, full stop, regardless of how long it's soaked).
4. A maintainer other than whoever last changed it approves the
   promotion PR (bumping `channel = "testing"` to `channel = "stable"`
   in its `recipe.toml`).

## What every package must have, regardless of channel

- A `recipe.toml` that validates against `docs/package-format.md` —
  `mitos-build-package` refusing to build it is a policy failure, not
  just a technical one.
- A `description` that says what the package *is*, not just its name
  restated.
- An accurate `dependencies` list — `docs/reproducible-builds.md` covers
  why an under-declared dependency is a build-reproducibility bug, not
  just a metadata nicety.
- A license its upstream source actually carries, noted in the PR
  description (this repository doesn't currently encode license
  metadata in `recipe.toml` itself — see the open item in
  `docs/package-format.md`).

## Naming and conflicts

Package names are unique across every channel combined — `testing` and
`stable` can both have a `mitos-shell`, but `community` can't introduce
a second, unrelated package also named `mitos-shell`. `provides` exists
precisely so a `community` alternative implementation of something can
coexist without a name collision — see `docs/package-format.md`.

## Removal / deprecation

A package that's failed CI for 30 consecutive days, or whose upstream
source has been gone for 90 days, gets removed from whichever channel(s)
it's in — not "held" indefinitely as a broken entry other packages might
still declare a dependency on. Removing a package that something else
still depends on requires resolving that dependency first (drop it,
replace it, or remove the dependent too) in the same PR.
