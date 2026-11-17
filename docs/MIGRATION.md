# Moving from AstralRinth-based releases

The `0.2.0-alpha.*` series runs on the Modrinth platform. Its database schema differs from the schema used by NyanRinth `1.1.0` and earlier.

## Before updating

1. Close Minecraft and the launcher.
2. Back up `%APPDATA%\dev.nyanrinth.launcher` and any custom directories holding installations and saves.
3. If you use a release earlier than `1.1.0`, update to `1.1.0` first. Older profile schemas need that intermediate update.

## What the converter does

The converter reads the older database without modifying its migration history, creates a SQLite snapshot that includes WAL state, and transfers supported data into a separately migrated database. It checks row counts, foreign keys and database integrity before publishing the converted database.

The converted file remains named `app-modrinth-v0221.db` for compatibility with the first Modrinth-based builds. Later upstream schema migrations apply normally; the filename does not identify the current upstream version.

Game accounts, installation identifiers and supported local synchronization preferences are retained. Newly introduced synchronization categories are disabled until selected. Unknown source columns stop conversion rather than being silently discarded.

## Returning to an older executable

The old database and its snapshot are preserved. However, game files can be shared by the old and new launcher generations. Reopening the older executable does not reverse changes to mods, saves or `options.txt`, and changes written only to the new database are not copied back.

Use a complete backup if you need a full rollback. Conversion has been exercised with synthetic profiles; migration of a primary real-world profile and installation over an earlier release remain testing gaps.
