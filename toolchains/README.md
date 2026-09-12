# toolchains/

Per-architecture cross-compilation settings for `mitos-build-package`
(via each package's `recipe.toml` `build.steps`, which can reference
`$MITOS_TARGET` once a step opts into cross-compiling — see
`docs/reproducible-builds.md`).

**Status:** MITOS doesn't have its own custom Rust target spec yet —
`mitos-init` and everything downstream of it currently build against the
standard `*-unknown-linux-gnu` triples. Each `toolchain.toml` below notes
this explicitly. Once MITOS's kernel/libc ABI is stable enough to ship a
custom `<arch>-unknown-mitos.json` target spec, these files are where
that lands, and every recipe's default (`x86_64-unknown-linux-gnu`
today) moves with it — recipes themselves shouldn't need to change,
since they reference `$MITOS_TARGET` rather than hardcoding a triple.

| Architecture | Status |
|---|---|
| `x86_64/` | Primary target — what CI actually builds and tests today. |
| `aarch64/` | Defined, not yet in CI (`ci/package-build.yml` builds `x86_64` only so far). |
| `riscv64/` | Defined, aspirational — no known working boot path yet. |
