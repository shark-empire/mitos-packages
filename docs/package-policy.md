# Package policy (full)

`POLICY.md` at the repository root is the summary of this document —
read that first. This is the detail behind each of its rules, and the
place to look if a PR was rejected against one of them and the reason
isn't obvious.

## Why three channels, not one

A single channel means every change — a brand-new, barely-tested
package and a security fix to something already on every installed
system — carries the same risk to whoever's pointed at it. Splitting
`testing` out means a new package can exist and be usable (opt-in, by
anyone willing to point `mitos-pkg` at the `testing` repository
directly) without that risk applying to `stable`'s default install base.
`community` exists for a related but different reason: it's not that
community packages are necessarily lower-quality, it's that this
project's maintainers can't vouch for arbitrary third-party source the
way they can for MITOS's own components — separating trust roots (see
`keys/README.md`) makes that distinction structural rather than a note
in a README nobody reads.

## Promotion criteria, expanded

The four criteria in `POLICY.md` are AND'd, not OR'd — all four, every
time, no exceptions for "this one's obviously fine." The 7-day soak
(criterion 2) specifically exists to catch problems that only show up
under real usage patterns CI doesn't exercise — shortening it for a
package that "seems fine" defeats the entire point of having it.

**On criterion 3 (signing):** an unsigned package can still be genuinely
useful in `testing` (a maintainer's own systems, explicitly opted in,
accepting the risk) — the line is drawn at promotion to `stable`
specifically because that's what `mitos-pkg`'s default configuration
installs from, i.e. what a user who never thought about signing at all
is trusting by default.

## What "an accurate dependencies list" actually protects against

Two failure modes, both real:

1. **Under-declaration**: package A's build happens to succeed because
   package B is present on the build machine for unrelated reasons (a
   shared build image, a previous build's leftover state), but A's
   `recipe.toml` never declares B as a dependency. Someone installs A
   fresh, without B, and it's broken — not because A is broken, because
   its dependency list lied. `docs/reproducible-builds.md` covers the
   build-isolation side of catching this.
2. **Over-declaration**: A declares a dependency it doesn't actually
   need anymore (left over from a previous version), forcing that
   dependency onto every system that installs A for no real reason —
   makes `mitos-pkg autoremove` and general system hygiene worse for
   everyone.

## Naming, `provides`, and `conflicts` in practice

`provides` is how `packages/applications/` can host a community
reimplementation of something without a name collision: a hypothetical
`mitos-terminal-alt` package can declare `provides = ["mitos-terminal"]`
so anything depending on "a terminal" is satisfied by either, while
`mitos-terminal-alt` and the real `mitos-terminal` still coexist as
distinct installable names. `conflicts` is the opposite tool — for two
packages that provide genuinely incompatible implementations of the
same thing (competing display managers, say) where installing both
would actively break the system rather than merely being redundant.

## Deprecation, in practice

"Failed CI for 30 consecutive days" means 30 days of `.github/workflows/package-build.yml`
actually running against that package and failing — not 30 days since
anyone looked at it. A package nobody has touched but that keeps
building fine isn't deprecated by neglect; a package that's actively
broken and nobody's fixing it is. The distinction matters because the
whole point is removing genuinely broken content, not punishing quiet
stability.
