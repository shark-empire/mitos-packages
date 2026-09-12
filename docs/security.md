# Security

The full verification chain from "recipe in this repository" to
"`mitos-pkg` on a user's system accepts this package" — see
`SECURITY.md` at the repository root to report a problem with any of
this, and `keys/README.md` for the key-management side specifically.

## The chain

```
recipe.toml (this repo, reviewed via PR)
│ git clone recipe.source.{git,rev}
▼
upstream source (a separate repo, its own history/review)
│ recipe.build.steps
▼
payload/ + pkg.json
│ mitos_pkg::package::build::build_package
▼
<name>-<version>.mpkg (contains manifest.json: payload_sha256, ...)
│ mitos-sign-package: sign(seed, payload_sha256)
▼
<name>-<version>.mpkg.sig + embedded in PackageMetadata.signature
│ mitos-generate-index
▼
index.json (+ index.json.sig if the channel requires signing)
│ network fetch
▼
mitos-pkg's PackageService::update → cached index
│ mitos-pkg install <name>
▼
PackageMetadata.sha256 checked against the downloaded file
PackageMetadata.signature checked against payload_sha256, via
trusted_keys_dir's public key for `signer`
```

Two independent checks happen on the install side, deliberately in this
order (see `mitos-pkg`'s `package::signature::verify_package`):

1. **Whole-file checksum** (`PackageMetadata.sha256`) — catches a
corrupted or truncated download before the tar/gzip parser ever sees
the bytes. This is an integrity check, not an authenticity one: it
protects against random corruption, not a deliberate attacker (who
could just recompute the checksum for a tampered file, if that were
the only check).
2. **Signature over `payload_sha256`** — the actual authenticity check.
`payload_sha256` is computed over the payload files specifically
(not the whole archive, which includes `manifest.json` itself — you
can't checksum a file that contains its own checksum), and the
signature over *that* value is what a trusted signer actually
attests to: "I built this payload."

Everything upstream of step 2 in the diagram — the recipe, the source
repo, the build steps — is what determines whether that attestation is
actually *true* (did the signer actually build what they claim to have
built), not something the signature itself can verify. That's a
process guarantee (PR review, `source.rev` pinning to a specific
commit for release builds, reproducible builds — see
`docs/reproducible-builds.md`), not a cryptographic one.

## What this repository's tooling does and doesn't protect against

**Does:** a tampered `.mpkg` in transit or on a mirror (checksum
catches it), a package claiming to be signed by someone who didn't sign
it (signature verification against `trusted_keys_dir` catches it), a
channel's `index.json` being tampered with in transit if the channel
requires index signing (`verify_index` catches it).

**Doesn't:** a compromised upstream source repository (this repository
clones whatever `source.git`/`source.rev` says — if that repository is
compromised, this pipeline faithfully builds and can sign the
compromised code, because from its perspective that *is* the source).
A compromised CI runner with access to a signing seed (see
`keys/README.md`'s rotation section for the response if this happens).
A malicious `build.steps` entry that does something other than what it
claims (this is exactly why new/changed recipes go through PR review,
not just "does it build").

## Reporting

See `SECURITY.md` at the repository root — do not open a public issue
for anything in this document's "does" list actually failing to hold.
