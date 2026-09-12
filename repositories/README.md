# repositories/

Three channels — `stable/`, `testing/`, `community/` — see
[`POLICY.md`](../POLICY.md) for what each means and how a package moves
between them. Each channel has the same four generated subdirectories;
see [`docs/repository-format.md`](../docs/repository-format.md) for the
full layout and [`scripts/`](../scripts) for what writes to each one.

Nothing under here is hand-edited. If you're trying to add or update a
package, you want [`packages/`](../packages) and
[`CONTRIBUTING.md`](../CONTRIBUTING.md), not this directory.
