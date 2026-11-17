# Changelog

Selected releases are listed here. Dates for archived versions refer to the preserved source archives, not their later publication on GitHub. See [UPSTREAM.md](UPSTREAM.md) for the import details.

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
- Excludes local environment overrides, development reports, agent/editor settings and temporary build output.
- Replaces the inherited API-documentation sample token with a placeholder and adds a secret-scanning configuration.

### Known issues

- Real PNG uploads to Ely.by remain an outstanding issue.
- The complete real-account login/2FA/refresh and Minecraft-loader matrix, primary-profile migration and installation over an earlier version need further testing.
- Automatic updates remain disabled; Windows x64 releases are installed manually.

## 1.1.0 Alpha — archived 2026-11-02

- Adds local synchronization of supported game options, with installation/option exclusions and backups. Files belonging to a running game are not overwritten.
- Ports the game-options catalog and conversions from Modrinth v0.22.1 while retaining the AstralRinth launcher platform.
- Expands imports to Prism, PineconeMC, Modrinth and AstralRinth, including relocated installation directories.
- Adds a dedicated account page, offline-account guidance, a first-launch sign-in prompt and launcher information.
- Connects skin management during Ely.by sign-in, separates uploaded and saved collections, and improves account forms, accent colors and Russian interface text.

This is the final selected archive on the older platform and a migration step for earlier installations. It was labeled Alpha in its original distribution.

## 1.0.4 — archived 2026-11-02

- Introduces native Ely.by skin browsing, tag filtering, PNG uploads, Steve/Alex previews and skin application, replacing the embedded website flow.
- Stores skin-site sessions using Windows DPAPI, separately from game-authentication tokens.
- Adds personal skin collections and refreshes account avatars after skin changes.
- Improves compact-window scrolling and removes unused upstream navigation from the primary launcher interface.
- Corrects release-build settings after a skin-page runtime failure.

This release includes changes from the smaller 1.0.1–1.0.3 updates.

## 1.0.0 — archived 2026-11-02

- Establishes NyanRinth on the AstralRinth beta platform.
- Adds direct Ely.by game authentication with 2FA and token refresh.
- Bundles authlib-injector 1.2.8 for Ely.by launches.
- Adds NyanRinth branding, configurable accent colors, memory-first settings and a separate application-data identifier.
- Provides the initial Ely.by skin-site flow and disables upstream binary updates and analytics.
