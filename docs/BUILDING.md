# Building NyanRinth

These instructions target the current Modrinth-based Windows x64 release. For historical releases, check out their tag and use the README at that tag.

## Requirements

- Windows x64
- Node.js 24.15 or newer
- pnpm 12.11.2, as pinned in `package.json`
- Rust 1.95.0 with the `x86_64-pc-windows-msvc` toolchain
- Visual Studio C++ Build Tools and Windows SDK
- JDK 17 or newer; the Java helper uses Gradle toolchains
- Microsoft Edge WebView2 Runtime

The Windows Cargo configuration uses `rust-lld`; ensure that the Rust toolchain's linker is available in your build environment.

## Prepare

```powershell
git clone https://github.com/iriskaaf/nyan-rinth.git
cd nyan-rinth
rustup toolchain install 1.95.0-x86_64-pc-windows-msvc
rustup override set 1.95.0-x86_64-pc-windows-msvc
Copy-Item packages/app-lib/.env.prod packages/app-lib/.env
pnpm install --frozen-lockfile
$env:SQLX_OFFLINE = 'true'
pnpm -w i18n:coverage
cargo run -p theseus --bin export_app_events --features export-ts --locked
```

The production environment template contains public service URLs, not private credentials. Keep local overrides in the ignored `.env` file. With `SQLX_OFFLINE=true`, compilation uses the checked-in SQLx query metadata.

## Build the installer

```powershell
pnpm --filter @modrinth/app build --bundles nsis
```

The standalone executable is written to `target/release/NyanRinth.exe`, and the installer to `target/release/bundle/nsis/`. The Tauri build runs the frontend production build before compiling the desktop shell.

Use the checked-in release/LTO profile consistently. Do not turn off LTO for only the executable: mixed build settings have caused runtime failures in the skin interface.

## Run and check

```powershell
pnpm app:dev
cargo test -p theseus --lib --locked
cargo clippy -p theseus -p theseus_gui --all-targets --locked
```

Development builds still use launcher data. Use a separate test profile when exercising migrations, imports or game-settings synchronization.

## Releases

Synchronize the versions in `apps/app-frontend/package.json`, `apps/app/Cargo.toml`, `packages/app-lib/Cargo.toml` and the corresponding `Cargo.lock` entries. Build from the release commit, check the executable version and exercise the changed flows. Record checks that remain outstanding.

Run `gitleaks git --redact` before pushing. Create the version tag and publish matching binaries, SHA-256 checksums, changes and known issues.

## Signed Windows releases

Releases are published to `iriskaaf/nyan-rinth`. The NyanRinth version and the Modrinth base version are tracked separately. The current NyanRinth series is `0.2.0-beta.*` on Modrinth v0.23.1.

The Tauri updater feature is enabled by default. `tauri.conf.json` contains the public signing key and the GitHub latest-release manifest URL. Keep the matching private signing key outside the source tree. The local release key is encrypted and its password is protected with Windows DPAPI in `%LOCALAPPDATA%/NyanRinth/ReleaseSigning`. Preserve this directory and its Windows user credentials when backing up the release machine.

Set `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` in the build process environment before bundling. These values must never enter Git. Tauri produces the NSIS installer and its `.sig` file when `createUpdaterArtifacts` is enabled.

After building from the committed version, run the release command to verify the installer signature and prepare the update feed and checksums. Set `CARGO_TARGET_DIR` if the build uses a shared target directory.

```powershell
node scripts/prepare-nyanrinth-release.mjs
git tag v0.2.0-beta.2
git push origin main v0.2.0-beta.2
node scripts/prepare-nyanrinth-release.mjs --publish
```

Replace the example tag with the actual package version. The publish command requires a clean checkout and matching tag. It uploads all assets as a draft, then publishes the complete release. Beta releases use GitHub's normal-release flag to permit the Latest designation and `/releases/latest/download/latest.json`; the application version keeps its Beta identifier. Alpha tags remain GitHub prereleases.

Clients check at startup and every five minutes. Network failures retain the retry timer. Downloads use the configured public key to verify the installer before it is queued. On metered connections, the existing download prompt is used. Installation occurs on launcher exit or an explicit restart request. Earlier builds with the updater disabled need one manual installation of this first Beta.
