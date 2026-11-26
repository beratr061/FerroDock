# Security policy

## Reporting a vulnerability

Please do not open a public issue for a security problem.

Report it privately through GitHub: open the repository's **Security** tab and choose **Report a vulnerability**. Only the maintainers can see the report.

Please include:

- the FerroDock version (Settings → Updates, or `ferrodock --version`);
- what an attacker needs first (no account, a viewer or developer account, an API key, control of a Git repository that gets deployed);
- steps to reproduce, and what they can read or change as a result.

You will get an answer within 7 days. Once a fix is released, the advisory is published with credit to you unless you prefer to stay anonymous.

## Supported versions

Only the latest release gets security fixes. The panel can update itself from Settings → Updates.

| Version | Supported |
| --- | --- |
| 0.3.x | Yes |
| older | No |

## Scope

In scope: the panel (API, web interface, WebSocket and terminal), the installer and update process, release signing, and the way FerroDock runs the applications, Compose stacks, databases and backups it manages.

Especially relevant:

- getting past role checks, for example a developer or a developer API key reaching another organization's resources or the host;
- Compose files or Git repositories that get past the compose security checks;
- reading the master key, encrypted variables, database passwords or backup contents;
- getting the updater to accept a release that was not signed.

Out of scope:

- anything that needs owner rights, or a stack the owner marked as trusted, since those can control the server by design;
- vulnerabilities in the applications you deploy, or in Docker, Traefik or PostgreSQL themselves (report those upstream, though we want to hear about insecure defaults we ship);
- missing rate limits on endpoints that do not need authentication, unless they cause real harm;
- denial of service through resource exhaustion by a user who is allowed to deploy.

## Testing

Test only against your own installation. Do not access other people's data, and stop as soon as you have shown the problem.
