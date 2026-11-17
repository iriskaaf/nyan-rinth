# Source history

NyanRinth has two launcher generations: the original AstralRinth-based releases and the later Modrinth-based series. This repository preserves the available source checkpoints for both.

## Browse a version

| Version | Main changes | Source |
| --- | --- | --- |
| 1.0.0 | Ely.by game authentication, bundled authlib-injector, branding and memory defaults | [Snapshot](https://github.com/iriskaaf/nyan-rinth/tree/v1.0.0) |
| 1.0.1 | Alex previews, account cards and navigation | [Snapshot](https://github.com/iriskaaf/nyan-rinth/tree/v1.0.1) |
| 1.0.2 | Native skin catalog, uploads and DPAPI-protected skin sessions | [Snapshot](https://github.com/iriskaaf/nyan-rinth/tree/v1.0.2) |
| 1.0.3 | Personal skin collections, avatars and compact-window layout | [Snapshot](https://github.com/iriskaaf/nyan-rinth/tree/v1.0.3) |
| 1.0.4 | Consistent release/LTO settings for the skin-page crash | [Snapshot](https://github.com/iriskaaf/nyan-rinth/tree/v1.0.4) |
| 1.0.5 | Shared sign-in flow, ownership checks and collection refresh | [Snapshot](https://github.com/iriskaaf/nyan-rinth/tree/v1.0.5) |
| 1.0.6 | Sign-in modal closing and skin-page scrolling | [Snapshot](https://github.com/iriskaaf/nyan-rinth/tree/v1.0.6) |
| 1.0.7 | Accent palettes, forms, account selection and Russian interface text | [Snapshot](https://github.com/iriskaaf/nyan-rinth/tree/v1.0.7) |
| 1.1.0 Alpha | Installation imports and local game-options synchronization | [Snapshot](https://github.com/iriskaaf/nyan-rinth/tree/v1.1.0) |
| 1.2.0-alpha.1 | First preserved port to Modrinth v0.22.1 | [Snapshot](https://github.com/iriskaaf/nyan-rinth/tree/v1.2.0-alpha.1) |
| 0.2.0-alpha.8 | Modrinth v0.23.1 platform and first public Windows release on that platform | [Snapshot](https://github.com/iriskaaf/nyan-rinth/tree/v0.2.0-alpha.8) |

The `0.2.0-alpha.*` series follows `1.2.*-alpha`: the project adopted independent version numbers when updating to Modrinth v0.23.1. See [UPSTREAM.md](../UPSTREAM.md) for the exact base revisions.

Source tags are available for smaller updates even when no separate downloadable binary release is published. Selected installers are collected on the [Releases page](https://github.com/iriskaaf/nyan-rinth/releases).

## How the history is organized

- `main` contains current development and the merged source history.
- `legacy/astralrinth` points to the published 1.1.0 checkpoint.
- `history/archived-releases` connects the preserved intermediate source versions in their release order, ending at the first saved Modrinth port.

The original published release commits and tags retain their identifiers. Checkpoint merges connect the intermediate archives to those existing commits. The final history merge keeps the current Modrinth source tree while making the older checkpoints reachable from `main`.

For imported archives, author dates follow the preserved ZIP timestamps. The 1.2.0-alpha.1 checkpoint uses the timestamp of its version manifest because it was preserved as a directory. Commit dates record when these snapshots were imported. These are saved source states; the original individual editing sessions were not recorded in Git.

## Compare changes

```bash
git log --graph --oneline --all
git diff v1.0.1 v1.0.2 -- apps/app-frontend packages/app-lib
git diff v1.1.0 v1.2.0-alpha.1 -- packages/app-lib
git show v1.0.5:apps/app-frontend/package.json
```

Historical checkpoints retain the code and package versions from their sources. Local credentials, temporary reports and machine-specific configuration are excluded. They are reference snapshots and have not all been rebuilt against today's services.
