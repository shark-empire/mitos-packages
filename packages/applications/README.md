# packages/applications/

Third-party and community application packages — the one category not
pre-populated with MITOS's own components (those live under `core/`,
`system/`, `desktop/`, and `development/`). Empty for now; this is where
a package like a browser, an office suite, or a game gets a
`packages/applications/<name>/recipe.toml` once someone packages it.

Anything here is expected to publish to the `community` channel (see
`POLICY.md`) unless a maintainer specifically takes it under the
project's own signing key. See `docs/maintainer-guide.md` for how to add
a new package, and `recipes/applications/cargo.toml` for the default
build template most Rust applications can just `inherits = "cargo"`
from.
