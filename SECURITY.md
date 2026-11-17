# Security

Please do not post live credentials or detailed account-compromise exploits in public issues.

If GitHub offers **Report a vulnerability** in this repository's Security tab, use it for a private report. Otherwise, open an issue asking the maintainer for a private reporting channel without including the exploit, tokens or personal account data.

Include the affected version, impact, steps to reproduce and a sanitized example. Do not attach a real launcher database, session directory or unreviewed diagnostic archive.

## Credentials and diagnostics

Local environment overrides, signing keys, session files, databases and logs are excluded from version control. The repository's production environment templates contain public service endpoints. Ely.by skin sessions are protected with Windows DPAPI and should not be transferred between Windows accounts.

Diagnostics filter known secrets and omit the raw account database, but logs can still contain personal paths and server addresses. Review an export before making it public.

Archived releases do not receive backported fixes. The current Beta series is the active development line; no fixed security-response timeline is promised.
