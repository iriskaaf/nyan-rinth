<p align="center">
	<img src="apps/app/icons/128x128@2x.png" width="112" alt="NyanRinth logo">
</p>

<h1 align="center">NyanRinth</h1>

<p align="center">A Minecraft launcher with Ely.by accounts, native skin management and a personal touch.</p>

<p align="center">
	<a href="https://github.com/iriskaaf/nyan-rinth/releases"><img alt="Download" src="https://img.shields.io/badge/download-Windows_x64-f4a7c5?style=flat-square"></a>
	<img alt="Development status" src="https://img.shields.io/badge/status-beta-f472b6?style=flat-square">
	<img alt="Built with" src="https://img.shields.io/badge/built_with-Rust_%2B_Vue-9dc7bd?style=flat-square">
	<a href="COPYING.md"><img alt="Licenses" src="https://img.shields.io/badge/license-see_COPYING.md-b7a7e8?style=flat-square"></a>
</p>

<p align="center">
	<a href="https://github.com/iriskaaf/nyan-rinth/releases">Downloads</a> ·
	<a href="CHANGELOG.md">Changelog</a> ·
	<a href="docs/BUILDING.md">Build from source</a> ·
	<a href="https://github.com/iriskaaf/nyan-rinth/issues">Report a bug</a> ·
	<a href="README.ru.md">Русский</a>
</p>

NyanRinth builds on the Modrinth launcher and adds Ely.by authentication, skin collections, local game-settings synchronization and configurable accent colors. Earlier releases used AstralRinth; the current Beta series uses the official Modrinth codebase directly.

**Current Beta: [1.2.0-beta](https://github.com/iriskaaf/nyan-rinth/releases/latest)** · [All releases and archives](https://github.com/iriskaaf/nyan-rinth/releases)

## Features

| | What you can do |
| --- | --- |
| **Ely.by accounts** | Sign in with two-factor authentication, refresh sessions and launch with the bundled authlib-injector. Offline accounts are also supported. |
| **Skins in the launcher** | Browse the Ely.by catalog, search tags, preview Steve and Alex models, apply skins and manage saved collections. |
| **Mods and modpacks** | Browse Modrinth content and manage separate Minecraft installations with their own loaders, mods and saves. |
| **Bring your installations** | Import `.mrpack` files and installations from supported launchers, including Prism, PineconeMC, Modrinth and AstralRinth. |
| **Shared game settings** | Choose which supported settings are synchronized locally between installations. Changes to `options.txt` are deferred while Minecraft is running. |
| **Make it yours** | Choose an accent color, adjust memory defaults and manage accounts from a dedicated page. |
| **Diagnostics** | Export logs and a diagnostic summary. The exporter filters known secrets and excludes the raw account database. |

## Download and install

Open **[Releases](https://github.com/iriskaaf/nyan-rinth/releases)** and choose a release. Current Beta releases use the `X.X.X-beta` format; `1.1.0` is the last archived release on the older AstralRinth platform and is also marked Alpha.

| File | Purpose |
| --- | --- |
| `NyanRinth_<version>_x64-setup.exe` | Windows installer |
| `NyanRinth_<version>_x64.exe` | Standalone executable; uses the same application-data folder as an installed copy |
| `SHA256SUMS.txt` | SHA-256 checksums for the downloadable binaries |
| `Source code` | Source files matching the release tag |

Windows x64 and Microsoft Edge WebView2 Runtime are required. Release binaries are not commercially code-signed. From 0.2.0-beta.2, signed Windows updates download from GitHub Releases automatically. Earlier Alpha builds require one manual upgrade to Beta.

1. Install NyanRinth and add an Ely.by or offline account.
2. Create an installation, import an existing one or download a modpack.
3. Select your account and launch Minecraft.
4. Open **Skins** to connect skin management and browse your collections.

Ely.by authentication does not grant access to servers that require a Microsoft account. Other players need compatible client or server support to see Ely.by skins. Skin changes may require restarting Minecraft and waiting for the service cache.

## Updating from an older version

Application data lives in `%APPDATA%\dev.nyanrinth.launcher`. Back up that folder and any custom installation directories before switching platforms. If you use a release earlier than `1.1.0`, update to `1.1.0` first.

The Modrinth-based series converts the older database into a separate database and retains the original. Both launcher generations can still refer to the same game files: reopening an older executable does not undo changes to those files. See [the migration notes](docs/MIGRATION.md).

## Current status

The launcher is under active development. Windows x64 is the distribution target; macOS and Linux builds are not provided here.

- PNG uploads to Ely.by remain experimental and have an outstanding real-account issue.
- The complete login/2FA/refresh and Minecraft-loader matrix has not been retested on the new platform.
- Migration has been exercised with synthetic profiles; upgrades of a real primary profile and installation over a previous release still need broader testing.
- Ely.by skin management uses the website API, which can change independently of NyanRinth.

If something fails, include your launcher version, Minecraft version, loader and steps to reproduce in an [issue](https://github.com/iriskaaf/nyan-rinth/issues/new/choose). Review exported diagnostics before sharing them publicly; logs can contain personal server addresses and paths.

## Development

The repository retains the upstream monorepo layout:

| Directory | Responsibility |
| --- | --- |
| `apps/app-frontend` | Vue interface |
| `apps/app` | Tauri desktop shell |
| `packages/app-lib` | Rust launcher core and NyanRinth integrations |
| `packages/ui`, `packages/assets` | Shared components and assets |
| `packages/api-client` | API clients |

Start with **[Building NyanRinth](docs/BUILDING.md)**. Contribution guidelines are in [CONTRIBUTING.md](CONTRIBUTING.md), and security-reporting guidance is in [SECURITY.md](SECURITY.md).

The [source history guide](docs/HISTORY.md) links each preserved version and explains the AstralRinth-to-Modrinth transition. Smaller updates have source tags; selected versions also have downloadable releases.

## Credits and licenses

NyanRinth is maintained by [iriskaaf](https://github.com/iriskaaf). It builds on [Modrinth](https://github.com/modrinth/code), with earlier releases based on [AstralRinth](https://github.com/42WASD/AstralRinth). The current upstream baseline is **Modrinth v0.23.1**. [UPSTREAM.md](UPSTREAM.md) records the exact base commit and the imported release history.

The bundled [authlib-injector 1.2.8](https://github.com/yushijinhun/authlib-injector/tree/v1.2.8) is unmodified and licensed under AGPL-3.0. Launcher packages retain their original license and copyright notices. See [COPYING.md](COPYING.md) and the license files in each package.

NyanRinth is an independent project, not an official product of Modrinth, Ely.by, Mojang or Microsoft.
