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

The public history starts with selected preserved NyanRinth source archives. It does not reproduce individual development commits or import the complete upstream Git history. Original copyright notices and package licenses are retained, and the exact upstream commits are identified above.

| Snapshot | Preserved source archive SHA-256 | Archive timestamp (UTC+05:00) |
| --- | --- | --- |
| `1.0.0` | `6c77be1afb46a220e4f48e5bc63ce7b4785ebe3f254321bf9122ebbec4cb1681` | 2026-11-02 12:40:25 |
| `1.0.4` | `ffee55b5a0327ca3a81ce0a31449e7457d474ac3e58749f3f47c3ed972a3b838` | 2026-11-02 14:15:48 |
| `1.1.0` | `94e88f019a20ea127d99c39ece60aae2e68f78e306d9b23b1f525aa1c3928a04` | 2026-11-02 21:28:56 |

For imported snapshots, the author date is the preserved archive file timestamp and the commit date is the import date. Archive timestamps are evidence of the saved release, not a daily activity log. Local credentials, editor/agent settings and temporary reports are excluded. A sample access token in inherited API documentation is replaced with a placeholder.

Intermediate binaries without a verified matching source snapshot are not republished as releases. The current release is built from its tagged source.

## Third-party components

Bundled authlib-injector **1.2.8** is unmodified. [Source and license](https://github.com/yushijinhun/authlib-injector/tree/v1.2.8). JAR SHA-256: `9c7f4343e6c82034958ffb48c14a2cb0c85928be7283103ce17da00c6d5a7b10`.

See [COPYING.md](COPYING.md), `apps/app/LICENSE`, `apps/app-frontend/LICENSE`, `packages/app-lib/COPYING.md` and the license files in other packages.
