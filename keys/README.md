# keys/

This directory documents MITOS's package-signing key model. It does
**not** — and must never — contain a private key or seed. Nothing under
`keys/` is read by any tool in this repository; it exists for
maintainers and auditors, not for `mitos-generate-index`/`mitos-sign-package`
to load from.

## The model

Signing here is the same local allowlist `mitos-pkg` itself implements
(`mitos_pkg::security::keys::KeyStore` — see that project's
`docs/security.md`): a package is trusted if, and only if, it's signed
by a key whose **public** half is present in the *installing* system's
`trusted_keys_dir`. There's no CA, no web of trust, no key server to
compromise — just a short, auditable list of `<signer>.pub` files.

- **Seeds** (the private half — 32 random bytes, hex-encoded, e.g. via
`openssl rand -hex 32 > seed.hex`) live only in whoever's signing
things' local environment or CI secrets store (see
`ci/security.yml`) — generated once per signer, never committed, never
logged, never pasted into a PR.
- **Public keys** (`<signer>.pub`, the hex-encoded 32-byte Ed25519 public
key derived from a seed) are what actually ship — both in this
repository (for maintainers to cross-check what a given signer name
*should* resolve to) and in MITOS's own `trusted_keys_dir` on installed
systems.

## Signer names used across this repository

`recipe.toml`'s `signer` field and `Manifest::signer` both name a signer
by a short identifier, not a key fingerprint directly — see
`docs/security.md` for the full verification chain. As of this writing:

| Signer name | Scope | Status |
|---|---|---|
| `mitos-release` | Official `stable`/`testing` channel packages | Key not yet generated — every `recipe.toml`'s `signer =` line is commented out pending this. |
| *(community signers)* | Individual `packages/applications/*` maintainers, one signer name per maintainer or project | None registered yet — see `docs/maintainer-guide.md` for how a community maintainer requests one. |

## Rotating or revoking a key

Removing a `<signer>.pub` from a system's `trusted_keys_dir` immediately
stops that signer's packages from installing or upgrading on that
system — there's no revocation list beyond "the key is no longer
present." Rotating a signer means: generate a new seed, publish the new
`.pub`, re-sign affected packages (`mitos-sign-package` with the new
seed), and — critically — get the new `.pub` distributed to already-
installed systems' `trusted_keys_dir` *before* publishing anything
signed only with the new key, or those systems can't verify the update
that would otherwise tell them about it. See `docs/security.md` for the
full writeup.
