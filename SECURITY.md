# Security Policy

## Reporting a vulnerability

**Do not open a public issue for a security vulnerability.** This
includes anything that could let an unsigned or tampered package pass
verification, a way to have a build step exfiltrate a signing key or
CI secret, or a flaw in `toolkit/`'s checksum/signature handling.

Instead, email **security@mitos-os.org** (or open a [GitHub private
security advisory](https://github.com/shark-empire/mitos-packages/security/advisories)
if you have access) with:

- What's affected — a specific tool (`mitos-verify-package`,
  `mitos-generate-index`, ...), a specific package's recipe, or CI
  itself.
- Reproduction steps, or a proof of concept if you have one.
- Your assessment of impact — does it let an untrusted package appear
  trusted, leak a key, or something else.

You'll get an acknowledgment within 5 business days. We don't currently
run a paid bug bounty; credit in the fix's changelog/release notes is
what's on offer, and we're glad to give it.

## Scope

In scope: `toolkit/`, `scripts/*`, `ci/*`, and any `recipe.toml`'s
`build.steps` being a vector for something worse than "this specific
package builds wrong" (e.g. a step that could affect *other* packages'
builds, or CI's own environment).

Mostly out of scope, but still worth a normal issue: a vulnerability in
an upstream package's own source code that has nothing to do with how
it's packaged here — report that to the upstream project. The exception
is if this repository's packaging of it introduces or worsens the issue
(e.g. an unnecessarily permissive file mode in `payload/`) — that part
is in scope.

## What "secure" means here, briefly

See [`docs/security.md`](docs/security.md) for the full verification
chain (checksum → signature → trusted-keys allowlist) and
[`keys/README.md`](keys/README.md) for the signing-key model. The short
version: a package is only as trustworthy as (a) its `recipe.toml`'s
`source.git`/`source.rev` actually pointing at what it claims to, (b)
whoever ran `mitos-sign-package` actually being who the `signer` name
says, and (c) the installing system's `trusted_keys_dir` only containing
public keys for signers it actually trusts. This repository's tooling
enforces (c) is checkable and (a)+(b) are auditable — it can't make
either of the first two true on its own, that's a process/PR-review
guarantee, not a cryptographic one.
