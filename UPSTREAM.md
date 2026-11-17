# Upstream and release history

## Current platform

- Project: [Modrinth monorepo](https://github.com/modrinth/code)
- Baseline: [`v0.23.1`](https://github.com/modrinth/code/tree/v0.23.1)
- Commit: `e949f1b2e73853e83ffdc9541bdb5aa5d5a660f1`
- NyanRinth version: `0.2.0-alpha.8`

NyanRinth retains the upstream content store, installation jobs, runtime cache, game-settings catalog and migration system. Its integrations include Ely.by authentication and skins, Windows DPAPI session storage, authlib-injector, offline accounts, launcher branding and deferred writes to game settings while Minecraft is running.

The upstream binary updater and outbound upstream error/game analytics are disabled for this distribution. The monorepo layout and package names are retained to keep integration with upstream practical.

## Earlier platform

NyanRinth `1.0.*` and `1.1.0` were based on [AstralRinth](https://github.com/42WASD/AstralRinth), beta commit `69e9ef400c5c77b0360d87bffabb952fa5aa6e76`. AstralRinth itself derives from Modrinth. The game-options catalog and conversions in NyanRinth `1.1.0` were ported from Modrinth `v0.22.1` without replacing that launcher platform.

The intermediate `1.2.*-alpha` builds ported the integrations to Modrinth `v0.22.1`. With the update to Modrinth `v0.23.1`, NyanRinth adopted independent version numbers beginning at `0.2.0-alpha.1`. The lower number represents a versioning reset, not an older codebase.

## Imported history

The source history includes every preserved 1.0.* archive, 1.1.0, the first saved Modrinth port and the current platform. See [docs/HISTORY.md](docs/HISTORY.md) for changes and source links. Original package copyright notices and licenses are retained; exact upstream commits are identified above.

| Snapshot | Preserved source archive SHA-256 | Saved timestamp (UTC+05:00) |
| --- | --- | --- |
| `1.0.0` | `6c77be1afb46a220e4f48e5bc63ce7b4785ebe3f254321bf9122ebbec4cb1681` | 2026-11-02 12:40:25 |
| `1.0.1` | `4a13ed7699ead475b82a19e1ff1e99529e8504675d1556aea5e33ee00f769de5` | 2026-11-02 13:00:55 |
| `1.0.2` | `77cf0bebeffbb86a383419ab1a2c1ef9fa6a8279be5da21d10761bdb3e380085` | 2026-11-02 13:35:16 |
| `1.0.3` | `36f155e528920f2aedd37411283cdb56a415392ebbb86aabcd7a2a20dea952be` | 2026-11-02 13:58:18 |
| `1.0.4` | `ffee55b5a0327ca3a81ce0a31449e7457d474ac3e58749f3f47c3ed972a3b838` | 2026-11-02 14:15:48 |
| `1.0.5` | `7aed225a2841857b79cfac71a583897e9d79c6eee1f62d6d4fd006933d07fb81` | 2026-11-02 14:37:53 |
| `1.0.6` | `8ed242bf53001d31ed2a999f9333a091517c4cf779e2e4afd8722cfe97ab7ee1` | 2026-11-02 14:56:27 |
| `1.0.7` | `d8ca9885cdd4b9a04931e913459c536484fe4ef5320b13fc024f60ccf0576490` | 2026-11-02 15:41:27 |
| `1.1.0` | `94e88f019a20ea127d99c39ece60aae2e68f78e306d9b23b1f525aa1c3928a04` | 2026-11-02 21:28:56 |
| `1.2.0-alpha.1` | Preserved source directory; version-manifest timestamp | 2026-11-03 13:19:36 |

Author dates follow the saved archive timestamps; the 1.2.0-alpha.1 directory checkpoint uses its version-manifest timestamp. Commit dates record the import. Local credentials, temporary reports and machine-specific configuration are excluded, and a sample token in inherited API documentation is replaced with a placeholder.

The `history/archived-releases` branch connects the intermediate snapshots to the existing release commits. Their published tags remain unchanged. Binary releases are limited to versions with verified matching artifacts; the other tags provide source checkpoints for comparison.

## Third-party components

Bundled authlib-injector **1.2.8** is unmodified. [Source and license](https://github.com/yushijinhun/authlib-injector/tree/v1.2.8). JAR SHA-256: `9c7f4343e6c82034958ffb48c14a2cb0c85928be7283103ce17da00c6d5a7b10`.

See [COPYING.md](COPYING.md), `apps/app/LICENSE`, `apps/app-frontend/LICENSE`, `packages/app-lib/COPYING.md` and the license files in other packages.
