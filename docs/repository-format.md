# Repository format

What lives under `repositories/<channel>/` (`channel` = `stable`,
`testing`, or `community`), and how the four subdirectories relate.
Everything here is *generated* — nothing under `repositories/` is
hand-edited; see each subdirectory's own `README.md` for which tool
writes it.

```
repositories/<channel>/
├── packages/<name>-<version>.mpkg          mitos-build-package writes
├── signatures/<name>-<version>.mpkg.sig    mitos-sign-package writes (optional)
├── metadata/<name>/<version>.json          mitos-generate-metadata writes
└── index/index.json (+ index.json.sig)     mitos-generate-index / mitos-publish-repository write
```

## `packages/`

The built archives themselves, named exactly what
`mitos_pkg::package::format::package_filename` produces
(`<name>-<version>.mpkg`) — the same function every tool here and
`mitos-pkg` itself uses, so there's one naming convention, not several
that happen to usually agree.

## `signatures/`

One `<name>-<version>.mpkg.sig` per signed archive: the hex-encoded
Ed25519 signature over that archive's `payload_sha256`, as plain text.
This is a **standalone copy** for auditing/tooling convenience — the
signature that actually matters is the one embedded in that package's
`metadata/.../<version>.json` (`PackageMetadata::signature`), which is
what `mitos-pkg` actually checks. A missing file here for an otherwise-
signed package is a warning from `mitos-publish-repository`, not a
build-breaking error.

## `metadata/`

One JSON file per package version — a `mitos_pkg::repository::metadata::PackageMetadata`:
name, version, description, dependencies, provides, conflicts, the
download `url`, the whole-archive `sha256`, optional `signature`, and
`size_bytes`. This is the unit `mitos-generate-index` folds into
`index.json` — see `toolkit::metadata::build_metadata` for exactly how
one gets produced from a built archive.

**Why a whole-archive `sha256` here, separate from the manifest's
`payload_sha256` inside the archive?** They check different things.
`payload_sha256` (inside `manifest.json`, inside the archive) is what
gets *signed* — it's a checksum of the payload files only, computed
before the archive itself exists. `PackageMetadata::sha256` (out here)
is a checksum of the **whole downloaded `.mpkg` file**, checked *before
the archive is even opened* — it's what stops a corrupted or truncated
download from reaching the tar/gzip parser at all. `mitos-pkg`'s own
`package::signature::verify_package` checks both, in that order,
deliberately — see that function's doc comments in `mitos-pkg` for the
full reasoning.

## `index/`

`index.json` is every `metadata/` entry folded into one
`mitos_pkg::repository::index::RepositoryIndex` — a map from package
name to every known version's metadata. This is the one file
`mitos_pkg::service::PackageService::update` actually fetches over the
network for a configured repository; everything else under
`repositories/<channel>/` only matters once `index.json` has pointed a
client at it. If the channel requires signing (any package in it has a
`signer` set — see `POLICY.md`), `index.json.sig` sits alongside it: the
hex Ed25519 signature over the *hex SHA-256 digest* of `index.json`'s
raw bytes (not the bytes directly — see `toolkit::sign::sign_index_bytes`'s
doc comment for why, and `mitos-pkg`'s `package::signature::verify_index`
for the check this mirrors).

## Regenerating vs. trusting what's on disk

`mitos-generate-index` always rebuilds `index.json` from whatever's
currently in `metadata/` — it never reads the previous `index.json` to
decide what to include. This means `metadata/` is the actual source of
truth for "what's in this channel," and `index.json` is a derived,
disposable file safe to delete and regenerate at any time.
`mitos-publish-repository` goes one step further and also confirms every
`metadata/` entry's archive is actually present in `packages/` and
matches its declared `sha256` before it'll regenerate the index — see
that tool's own doc comment in `scripts/publish-repository/src/main.rs`.
