# FerroDock

A lightweight, self-hosted PaaS control plane. Rust backend, React SPA, one binary, Apache-2.0.

FerroDock deploys applications, Docker Compose stacks and managed databases on your own servers with Traefik routing, automatic HTTPS and live logs, while staying small enough to leave the server's resources to your workloads.

## What it does today (0.3)

- **Applications** from a Docker image or a Git repository (HTTPS or SSH), built from a Dockerfile or run as a Docker Compose stack. Deployments run by hand or on every Git push (GitHub, GitLab, Gitea and Forgejo webhooks), use blue-green (zero downtime) or recreate, stream their log live, can be cancelled and rolled back, and old images are pruned (the last five successful ones are kept).
- **Compose stacks are checked before they run.** Options that reach the host are refused, including privileged mode, the Docker socket, host namespaces, devices and files outside the repository. Compose itself runs in a short-lived helper container that cannot see the panel's keys. Only the organization owner can mark a stack as trusted.
- **Domains** routed through Traefik, with Let's Encrypt certificates, an HTTP to HTTPS redirect, basic auth and a DNS pre-check.
- **Encrypted environment variables**, published ports and volumes per application.
- **Managed databases:** PostgreSQL, MySQL, MariaDB, MongoDB, Redis and Valkey, with generated credentials, CPU and memory limits and an optional public port.
- **Backups:** scheduled database and volume backups to local disk or S3-compatible storage, streamed and encrypted, with restore. The panel also backs itself up daily.
- **Web terminal** into containers: WebGL rendering, reconnect after a network drop, a mobile key bar.
- **Live view:** container status, logs and stats over a single WebSocket.
- **Security:** first-owner setup token, Argon2id passwords, lockout, scoped API keys, roles (owner, admin, developer, viewer), audit log, rate limiting and strict security headers.
- **Self-update:** signed release manifest, a backup before every update, a health-gated switch and automatic rollback.
- **Interface:** English and Turkish, light and dark themes, works down to phone width, and a Ctrl/Cmd+K command palette.

Not yet: inviting team members, remote servers over SSH, metrics charts, notifications and preview deployments. The roadmap is in `docs/adr/0001-architecture.md`.

## Install on a server

```sh
curl -fsSL https://github.com/beratr061/FerroDock/releases/latest/download/install.sh | sudo sh
```

The installer prints a one-time setup token; the first owner account can only be created with it (also stored in `/etc/ferrodock/setup-token`).

Optional environment variables: `FERRODOCK_DOMAIN` (serve the panel over HTTPS on this domain), `FERRODOCK_ACME_EMAIL`, `FERRODOCK_PANEL_PORT` (default 3000) and `FERRODOCK_VERSION`. Configuration lives in `/etc/ferrodock`; back up `/etc/ferrodock/panel/master.key`, without it encrypted secrets cannot be read.

`deploy/uninstall.sh` stops the stack; `deploy/uninstall.sh --purge` also deletes all data.

## Backups

The panel backs itself up every day (`FERRODOCK_BACKUP_INTERVAL_HOURS`, default 24, `0` turns it off) and keeps the newest seven archives (`FERRODOCK_BACKUP_RETENTION`). An archive holds a `pg_dump` of the panel database, taken inside the Postgres container, plus the proxy route files, sealed with AES-256-GCM using the master key. The master key is not inside the archive: export it once from Settings → Backups (your password is required) and store it away from the server.

To restore on a fresh install, copy the archive to `/var/lib/ferrodock/backups/` and the key to `/var/lib/ferrodock/restore.key`, then:

```sh
docker compose -f /etc/ferrodock/docker-compose.yml stop ferrodock
docker compose -f /etc/ferrodock/docker-compose.yml run --rm ferrodock restore /var/lib/ferrodock/backups/<file>.fdbk --master-key /var/lib/ferrodock/restore.key
docker compose -f /etc/ferrodock/docker-compose.yml start ferrodock
```

Archives from a newer FerroDock release are refused; update the panel first.

## Updates

Settings → Updates checks the signed release manifest. Updating takes a backup, pulls the release by digest and starts a short-lived `ferrodock-updater` container from the new image. The updater points `FERRODOCK_IMAGE` in `/etc/ferrodock/.env` at the new image, recreates the panel with Docker Compose and waits for its health check; if the new panel is not healthy it switches back to the previous image on its own. The outcome is shown on the same page, which also offers a rollback to the previous version. Updates are refused while deployments are running.

The same switch can be run by hand: `ferrodock update apply --image <image> --config-dir /etc/ferrodock`.

## Develop

Requirements: Rust 1.99, Node 24 with pnpm, Docker.

```sh
docker run -d --name ferrodock-dev-pg -e POSTGRES_USER=ferrodock -e POSTGRES_PASSWORD=ferrodock-dev -e POSTGRES_DB=ferrodock -p 127.0.0.1:55433:5432 postgres:18.7-alpine
```

```sh
FERRODOCK_DATABASE_URL=postgres://ferrodock:ferrodock-dev@127.0.0.1:55433/ferrodock FERRODOCK_MASTER_KEY_PATH=./data/master.key FERRODOCK_SECURE_COOKIES=false FERRODOCK_LOG_JSON=false FERRODOCK_BIND_ADDR=127.0.0.1:3900 cargo run -p ferrodock -- serve
```

```sh
pnpm --dir web install && pnpm --dir web dev
```

The Vite dev server proxies `/api` to `127.0.0.1:3900`.

| Task | Command |
| --- | --- |
| Lint Rust | `cargo clippy --workspace --all-targets` |
| Test Rust (needs `DATABASE_URL`) | `cargo test --workspace` |
| Code rules (no comments, 250 lines) | `cargo xtask check-rules` |
| Scaffold a feature | `cargo xtask new-feature <snake_case_name>` |
| Refresh sqlx offline data | `cargo sqlx prepare --workspace -- --all-targets` |
| Refresh API contract | `cargo run -p ferrodock -- openapi > openapi.json` |
| Lint and build web | `pnpm --dir web lint && pnpm --dir web build` |

## Releasing

Releases are signed. Once, run `cargo xtask release-key ./release.key`, store the printed private key file content as the repository secret `FERRODOCK_RELEASE_SIGNING_KEY`, the printed public key as the repository variable `FERRODOCK_RELEASE_PUBLIC_KEY`, then delete `release.key`. Pushing a `vX.Y.Z` tag builds the multi-arch image, embeds the public key, signs `manifest.json` with the image digest and publishes the installer assets. Servers check for updates with `ferrodock update check`; a build without the public key refuses to trust any manifest.

## License

Apache-2.0.
