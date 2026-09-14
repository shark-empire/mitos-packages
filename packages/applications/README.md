# packages/applications/

Third-party and community application packages — the one category not
made up of MITOS's own components (those live under `core/`, `system/`,
`desktop/`, and `development/`). Anything here publishes to the
`community` channel (see `POLICY.md`) once verified — `testing` until
then — unless a maintainer specifically takes it under the project's
own signing key.

## What's here so far

| Package | Fills the gap of | Upstream | Channel |
|---|---|---|---|
| `bottom` | Task manager (process list + kill) | [ClementTsang/bottom](https://github.com/ClementTsang/bottom) | community |
| `task-manager-og` | Task manager (GUI alternative to `bottom`) | [mikedesu/evildojo-tmog](https://github.com/mikedesu/evildojo-tmog) | testing — see its `recipe.toml`, provenance not yet independently confirmed |
| `helix` | Text editor | [helix-editor/helix](https://github.com/helix-editor/helix) | community |
| `ouch` | Archive manager | [ouch-org/ouch](https://github.com/ouch-org/ouch) | community |
| `dust` | Disk usage analyzer | [bootandy/dust](https://github.com/bootandy/dust) | community |
| `gitui` | Terminal git client | [gitui-org/gitui](https://github.com/gitui-org/gitui) | community |
| `firefox` | Web browser | Mozilla's official binary — see its `recipe.toml` for a trademark/redistribution note | testing |

Two of these are `testing` rather than `community` on purpose, not by
oversight — read the note at the top of each one's `recipe.toml` before
promoting either:

- **`task-manager-og`**: the *official* "Task Manager OG"/TMOG (Dave
  Plummer) is closed-source with a paid tier, so this can only be an
  unofficial, unaffiliated rebuild — worth confirming what
  `mikedesu/evildojo-tmog` actually is before this goes further.
- **`firefox`**: redistributes Mozilla's official trademarked binary
  directly, not a build from source — worth checking Mozilla's current
  branding/distribution policy first (see Debian's old Iceweasel
  history for why this has mattered to other distros before).

`bottom` also overlaps `packages/system/mitos-system-monitor` to some
degree — see `bottom`'s own `recipe.toml`.

## Still missing, on purpose rather than by oversight

An image viewer and a PDF viewer aren't stubbed in yet — most
reasonable options need a GUI toolkit (GTK, Qt) whose support under
`mitos-gui`'s Smithay-based compositor isn't confirmed. `firefox` and
`task-manager-og` (Qt5) above are already testing that water to some
degree; once one of them is confirmed working end to end on real
`mitos-gui`, the same toolkit question is answered for the rest of this
category too.

See `docs/maintainer-guide.md` for how to add the next one.
