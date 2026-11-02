<p align="center"><img src="apps/app/icons/128x128.png" width="96" alt="NyanRinth"></p>

# NyanRinth 1.0.3

Minecraft launcher for Windows with Ely.by accounts and skins, based on AstralRinth.

This is an archived release. See the [project homepage](https://github.com/iriskaaf/nyan-rinth) for the current version and [Releases](https://github.com/iriskaaf/nyan-rinth/tree/v1.0.3) for this preserved source snapshot.

## Build

Windows x64, Node.js 24.15+, pnpm 10.33.2, Rust 1.95.0 MSVC, Visual Studio C++ Build Tools with Windows SDK, and JDK 17+ are required.

```powershell
Copy-Item packages/app-lib/.env.prod packages/app-lib/.env
pnpm install --frozen-lockfile
$env:SQLX_OFFLINE = 'true'
pnpm --filter @modrinth/app build --bundles nsis
```

## Credits and licenses

See [UPSTREAM.md](UPSTREAM.md), [COPYING.md](COPYING.md), and the license files in each package. NyanRinth is an independent project and is not an official product of Modrinth, Ely.by, Mojang or Microsoft.
