# Contributing

Bug reports and focused pull requests are welcome. Please discuss large changes in an issue before starting them.

## Reporting a bug

Include the launcher version, Windows version, Minecraft version and loader, steps to reproduce, expected behavior and the actual result. If the problem involves Ely.by, distinguish game authentication from skin-site authentication.

Attach only the relevant diagnostic material. Never post passwords, session cookies, access tokens or the launcher database. Review exported ZIP files before sharing them.

## Development

Follow [the build instructions](docs/BUILDING.md). Use tabs for indentation and keep changes focused. Preserve upstream license notices and use types from `packages/api-client` when available.

The frontend is in `apps/app-frontend`, Tauri commands in `apps/app`, and launcher logic in `packages/app-lib`. Keep account/session handling in the existing authentication modules rather than adding a second implementation.

Run relevant Rust tests and verify changed behavior in the final executable. Before a frontend pull request, use the repository's `pnpm prepr:frontend:app` command and the library checks when shared packages change. Record what was tested and any remaining gaps.

## Releases

Historical release tags are preserved snapshots. New releases use independent NyanRinth versions, with upstream versions recorded separately in `UPSTREAM.md`. Do not attach binaries built from another source revision to an existing tag.
