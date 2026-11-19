# Changelog

Selected releases are listed here. Dates for archived versions refer to the preserved source archives, not their later publication on GitHub. See [UPSTREAM.md](UPSTREAM.md) for the import details.

## 0.2.0-beta.3 - 2026-11-19

- Shows the NyanRinth logo on the welcome screen for an empty library.

## 0.2.0-beta.2 - 2026-11-17

- Uses the GitHub asset download Accept header when checking the update manifest.
- Bounds each update check to 30 seconds so network stalls can be retried.
- Supersedes the withdrawn first Beta packaging attempt; native GitHub feed checks now pass.

## 0.2.0-beta.1 — 2026-11-17

- Promotes the Modrinth v0.23.1 based NyanRinth series to Beta.
- Checks GitHub Releases for updates at startup and every five minutes, including after a failed network request.
- Downloads Windows x64 updates in the background on unmetered connections and offers an install/restart action.
- Verifies every update using a dedicated signing key embedded in the launcher.
- Adds a signed release manifest, installer signatures and SHA-256 checksums to releases.
- Opens the NyanRinth GitHub changelog from update notifications.
- Adds a release preparation command that verifies signatures before publication.

Upgrade from earlier Alpha builds by installing this Beta once. Those builds have automatic updates disabled. The first signed Beta enables subsequent OTA updates. Updates download automatically and install when the launcher closes or the user requests a restart.

GitHub marks this Beta as Latest. Its release metadata uses the normal-release flag so GitHub can serve the latest update manifest. The application version retains the Beta identifier.

Known limitations remain in real-account Ely.by PNG upload behavior and the complete login/2FA/loader matrix. Updater signatures authenticate update packages. Windows Authenticode signing is separate and is not included in this release.

## 0.2.0-alpha.9 — source update, 2026-11-17

- Restores the six intermediate source archives: 1.0.1, 1.0.2, 1.0.3, 1.0.5, 1.0.6 and 1.0.7.
- Preserves the first saved Modrinth v0.22.1 port as the 1.2.0-alpha.1 source checkpoint.
- Connects the archived source sequence to the existing published release commits and current development.
- Adds a source-history guide and descriptions of the intermediate changes.
- Keeps machine-specific tool exclusions in local Git configuration.

This update maintains the source repository. The current downloadable Windows preview is [0.2.0-alpha.8](https://github.com/iriskaaf/nyan-rinth/releases/tag/v0.2.0-alpha.8).

## 0.2.0-alpha.8 — 2026-11-17

### Platform

- Uses the official Modrinth v0.23.1 baseline, replacing the older AstralRinth platform.
- Retains the upstream installation flow, content store, runtime cache and game-settings catalog.
- Converts compatible NyanRinth 1.1.0 data into a separate database while retaining the source database and snapshot.
- Keeps the independent NyanRinth version series introduced with `0.2.0-alpha.1`.

### Launcher

- Carries over Ely.by game authentication, native skin management, offline accounts and the bundled authlib-injector.
- Provides skin collections, tag search, Steve/Alex previews and saved-skin state handling.
- Defers shared `options.txt` writes until Minecraft exits, preserving changes made by the running game.
- Includes the NyanRinth icon set and an interactive 3D model in the About window.
- Exports logs and a filtered diagnostic summary through the About window without including the raw account database.

### Distribution

- Adds the public repository, English and Russian READMEs, build instructions, migration notes and contribution guidance.
- Imports selected historical source snapshots and publishes matching archived installers.
- Excludes local environment overrides, development reports, machine-specific settings and temporary build output.
- Replaces the inherited API-documentation sample token with a placeholder and adds a secret-scanning configuration.

### Known issues

- Real PNG uploads to Ely.by remain an outstanding issue.
- The complete real-account login/2FA/refresh and Minecraft-loader matrix, primary-profile migration and installation over an earlier version need further testing.
- Automatic updates remain disabled; Windows x64 releases are installed manually.

## 1.2.0-alpha.1 — preserved 2026-11-03

- Ports NyanRinth to the official Modrinth v0.22.1 platform.
- Reuses the upstream content store, installation jobs, runtime cache and synchronized options.
- Carries over Ely.by authentication, native skins, DPAPI sessions and authlib-injector.
- Adds a converter for the older database schema that retains the source database and snapshot.
- Defers shared settings writes while Minecraft is running.

This source checkpoint was preserved as a directory and has not been rebuilt for publication. Its tag is available for comparison; no additional binary release is attached.

## 1.1.0 Alpha — archived 2026-11-02

- Adds local synchronization of supported game options, with installation/option exclusions and backups. Files belonging to a running game are not overwritten.
- Ports the game-options catalog and conversions from Modrinth v0.22.1 while retaining the AstralRinth launcher platform.
- Expands imports to Prism, PineconeMC, Modrinth and AstralRinth, including relocated installation directories.
- Adds a dedicated account page, offline-account guidance, a first-launch sign-in prompt and launcher information.
- Connects skin management during Ely.by sign-in, separates uploaded and saved collections, and improves account forms, accent colors and Russian interface text.

This is the final selected archive on the older platform and a migration step for earlier installations. It was labeled Alpha in its original distribution.

## 1.0.7 — archived 2026-11-02

- Adds pink, blue and green accent palettes and consistent focus/overlay colors.
- Submits sign-in forms with Enter through their normal submit path.
- Uses shared account selection on the skin page and hides Ely.by collections for other account types.
- Expands Russian interface messages across installation management, settings, invitations and error dialogs.

## 1.0.6 — archived 2026-11-02

- Releases modal locks before closing successful Ely.by sign-in and skin-session forms.
- Keeps preview controls visible at compact window sizes.
- Separates catalog scrolling from the skin preview and improves the narrow-screen layout.

## 1.0.5 — archived 2026-11-02

- Opens uploaded skins by default and validates ownership in personal collections.
- Refreshes the uploaded collection after PNG submission.
- Connects skin management during game sign-in while credentials are still available in memory, then clears them.
- Preserves successful game sign-in if connecting to the skin website fails.

## 1.0.4 — archived 2026-11-02

- Introduces native Ely.by skin browsing, tag filtering, PNG uploads, Steve/Alex previews and skin application, replacing the embedded website flow.
- Stores skin-site sessions using Windows DPAPI, separately from game-authentication tokens.
- Adds personal skin collections and refreshes account avatars after skin changes.
- Improves compact-window scrolling and removes unused upstream navigation from the primary launcher interface.
- Corrects release-build settings after a skin-page runtime failure.

This release includes changes from the smaller 1.0.1–1.0.3 updates.

## 1.0.3 — archived 2026-11-02

- Separates personal skin collections and the global catalog.
- Refreshes account heads after skin application.
- Improves preview sizing and scrolling in compact windows.

## 1.0.2 — archived 2026-11-02

- Replaces the embedded skin website with native catalog browsing, tag search, uploads, previews and skin application.
- Keeps the skin-site session separate from the game token and protects it with Windows DPAPI.
- Adds local session disconnection without removing the game account.

## 1.0.1 — archived 2026-11-02

- Corrects slim Alex previews and account-card presentation.
- Reuses the Ely.by window during the initial website-based skin flow.
- Simplifies primary navigation and fixes clipped settings headings.

## 1.0.0 — archived 2026-11-02

- Establishes NyanRinth on the AstralRinth beta platform.
- Adds direct Ely.by game authentication with 2FA and token refresh.
- Bundles authlib-injector 1.2.8 for Ely.by launches.
- Adds NyanRinth branding, configurable accent colors, memory-first settings and a separate application-data identifier.
- Provides the initial Ely.by skin-site flow and disables upstream binary updates and analytics.
