# Package format

Two formats matter here: `recipe.toml` (what a maintainer writes) and
`.mpkg` (what `mitos-build-package` produces from it, and what
`mitos-pkg` installs). This document covers both, plus how the toolkit
converts one into the other.

## `recipe.toml`

Lives at `packages/<category>/<name>/recipe.toml`. Parsed by
`toolkit::recipe::Recipe` (`toolkit/src/recipe.rs`) — that file's doc
comments are the authoritative source if this document and the code
ever disagree.

```toml
name = "mitos-example"
version = "0.1.0"
description = "One line describing what this package is."
category = "system"        # must match the directory it's in
channel = "testing"        # "stable" | "testing" | "community" — default: testing
inherits = "cargo"         # optional — see recipes/README.md

[source]
git = "https://github.com/shark-empire/mitos-example"
rev = "main"                # branch, tag, or commit — default: "main"

[build]
# Left out entirely (or steps = []) when `inherits` supplies them.
# system = "cargo"          # informational only
# steps = ["cargo install --path . --root \"$MITOS_PAYLOAD_DIR/usr\" --locked"]

[[dependencies]]
name = "mitos-utils"
version = ">=0.1.0"        # a semver VersionReq — same syntax mitos-pkg's
                            # own dependency::version::Dependency uses

provides = []               # names this package can stand in for
conflicts = []               # names this package cannot coexist with
# signer = "mitos-release"  # who signs this — see keys/README.md
```

Every field beyond `name`/`version`/`description`/`category`/`source`
has a default (see the struct's `#[serde(default = ...)]` attributes) —
a minimal recipe is genuinely about eight lines.

### Fields

| Field | Type | Notes |
|---|---|---|
| `name` | string | Must match the directory name. |
| `version` | semver | `MAJOR.MINOR.PATCH`, no `v` prefix. |
| `description` | string | One line; shown by `mitos-pkg info`/`search`. |
| `category` | string | Must match the directory it's under. |
| `channel` | `stable` \| `testing` \| `community` | Which `repositories/<channel>/` this builds into. |
| `inherits` | string, optional | A `recipes/<category>/<this>.toml` template — see `recipes/README.md`. |
| `source.git` | URL | Cloned with `git clone --depth 1 --branch <rev>`. |
| `source.rev` | string | Branch, tag, or commit. Default `"main"`. |
| `build.system` | string, optional | Informational; not interpreted. |
| `build.steps` | list of shell commands | Run via `sh -c`, cwd = the clone, `$MITOS_PAYLOAD_DIR` set — see below. |
| `dependencies` | list of `{name, version}` | Same shape as `mitos-pkg`'s own `Dependency`. |
| `provides` | list of strings | Same meaning as in `mitos-pkg`'s `PackageSpec`. |
| `conflicts` | list of strings | Same. |
| `signer` | string, optional | A name from `keys/README.md`'s table. |

### `build.steps` and `$MITOS_PAYLOAD_DIR`

Each step is a shell command. Before the first step runs,
`mitos-build-package` has already created an empty directory and set
`$MITOS_PAYLOAD_DIR` to its absolute path — a step installs its output
by writing there, using the same layout the final package should have
relative to `/` (e.g. a binary belongs at
`$MITOS_PAYLOAD_DIR/usr/bin/mitos-example`). Steps run with cwd set to
the freshly cloned source, not `$MITOS_PAYLOAD_DIR` — `cd` explicitly if
a step needs to run somewhere else.

**Known gap:** there's currently no declared "license" field — see
`POLICY.md`'s note on this. A recipe with an actual license mismatch
between what it declares in its PR description and what upstream
actually ships won't be caught by any tool here today, only by review.

## `.mpkg` (unchanged from `mitos-pkg`)

`mitos-build-package` doesn't invent its own archive format — it calls
`mitos_pkg::package::build::build_package` directly, so a `.mpkg` built
here is byte-for-byte what running `mitos-pkg build` by hand on the same
staged source would produce: a gzip'd tar containing `manifest.json`
(the payload's sha256, dependencies, provides/conflicts, and signer,
copied from the recipe) and a `payload/` tree. See `mitos-pkg`'s own
`docs/INTEGRATION.md` (`## Package archive format (.mpkg)`) for the
byte-level detail — repeating it here would just be a second copy that
could drift out of sync with the code that actually produces it.

## From recipe to `pkg.json`

`Recipe::to_package_spec()` (`toolkit/src/recipe.rs`) is the entire
conversion: `name`, `version`, `description`, `dependencies`,
`provides`, `conflicts`, and `signer` copy across verbatim into a
`mitos_pkg::package::spec::PackageSpec`, which `mitos-build-package`
writes to `pkg.json` before calling `build_package`. Nothing about
`source`, `category`, `channel`, `inherits`, or `build` survives into
the built package — those are purely this repository's concern, not
`mitos-pkg`'s.
