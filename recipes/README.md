# recipes/

Shared, reusable build-step templates — the `inherits` target for
`packages/<category>/<name>/recipe.toml` files (see
`docs/package-format.md`). A package recipe that sets
`inherits = "cargo"` pulls its default `build.steps` from
`recipes/<its category>/cargo.toml`, instead of every one of the twenty
or so Rust-based MITOS components repeating the same build/install lines.

This mirrors how most real distros avoid the same duplication — Gentoo's
eclasses, Void's `srcpkgs` templates — just scoped down to what this
repository actually needs: one build system (Cargo), two shapes of crate
(binary, library-only).

## What's here

Each category directory (`core/`, `system/`, `desktop/`, `development/`,
`applications/`) holds:

- **`cargo.toml`** — for crates that produce one or more binaries.
  `cargo install --path . --root "$MITOS_PAYLOAD_DIR/usr"` works
  generically for any such crate regardless of its binary name(s) —
  including multi-binary crates like `mitos-pkg` (which also builds
  `mitos-pkgd`) — because `cargo install --root` figures out what a
  crate produces and installs it under `<root>/bin/` itself.
- **`cargo-lib.toml`** — for library-only crates (`mitos-utils`,
  `mitos-sdk`). `cargo install` has nothing to install for these (no
  binaries), so this template only runs `cargo build --release` and
  leaves `$MITOS_PAYLOAD_DIR` empty. A library-only recipe that actually
  needs something in its package (headers, a `.rlib`, docs) should give
  its own `[build] steps` in full rather than relying on this template —
  see the comment in that file.

## Writing a new template

A template is just a `Template` (`toolkit::recipe::Template`):

```toml
system = "cargo"
steps = [
    "first shell command, run via sh -c, cwd = the checked-out source",
    "second shell command",
]
```

Every step runs with `$MITOS_PAYLOAD_DIR` set to the package's (already
created, initially empty) payload staging directory — a step installs
its output by writing there, the same directory that becomes `payload/`
inside the built `.mpkg`. See `docs/package-format.md` for the full
archive layout and `docs/maintainer-guide.md` for a worked example of
writing a recipe that needs a step beyond what a template provides.

## What `inherits` does and doesn't do

- A recipe's own `build.steps`, once non-empty, always wins — `inherits`
  supplies a *default*, not a forced override. A package that needs one
  extra `patch` step before the usual cargo build writes its own full
  `steps` list (which can just be the template's steps plus one more)
  instead of fighting the template.
- `inherits` only resolves within the recipe's own category
  (`packages/system/mitos-network/recipe.toml` with `inherits = "cargo"`
  resolves against `recipes/system/cargo.toml`, not
  `recipes/core/cargo.toml`) — if two categories' templates should be
  identical, that's a sign the template belongs in one shared place
  instead; this repository hasn't needed that yet, since every category
  here is Rust/Cargo end to end.
