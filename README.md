# FerroDock

A lightweight, self-hosted PaaS control plane. Rust backend, React SPA, one binary, Apache-2.0.

FerroDock deploys applications, Docker Compose stacks and managed databases on your own servers with Traefik routing, automatic HTTPS and live logs, while staying small enough to leave the server's resources to your workloads.

## Status

Phase 0 (foundation) is in place:

- Rust workspace with strict lints, typed errors with i18n-ready error codes, AES-256-GCM secret storage and a generated master key.
- PostgreSQL schema and migrations for organizations, users, sessions, API keys and audit logs, with compile-time checked queries (`sqlx` offline data in `.sqlx/`).
- Authentication: first-owner setup, Argon2id passwords, account lockout, HttpOnly session cookies, scoped API keys, CSRF origin checks, per-IP rate limiting and security headers.
- REST API under `/api/v1` with an OpenAPI 3.1 contract (`openapi.json`) that generates the frontend types.
- React SPA shell: setup, sign-in, projects, settings and API keys; English and Turkish; light, dark and system themes; responsive down to 375 px.
- Container image, Docker Compose stack, one-command installer and CI.

Deployments, Traefik routing for apps, managed databases and the terminal arrive in the following phases; see `docs/adr/0001-architecture.md`.

## Install on a server

```sh
curl -fsSL https://github.com/beratr061/FerroDock/releases/latest/download/install.sh | sudo sh
```

The installer prints a one-time setup token; the first owner account can only be created with it (also stored in `/etc/ferrodock/setup-token`).

Optional environment variables: `FERRODOCK_DOMAIN` (serve the panel over HTTPS on this domain), `FERRODOCK_ACME_EMAIL`, `FERRODOCK_PANEL_PORT` (default 3000) and `FERRODOCK_VERSION`. Configuration lives in `/etc/ferrodock`; back up `/etc/ferrodock/panel/master.key`, without it encrypted secrets cannot be read.

`deploy/uninstall.sh` stops the stack; `deploy/uninstall.sh --purge` also deletes all data.

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
