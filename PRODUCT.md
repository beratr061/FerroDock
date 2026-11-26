# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

- Solo developers and indie makers who run a handful of applications and databases on their own VPS and want to leave hosted PaaS bills behind.
- Small teams and agencies managing company or client projects across several servers, who need roles and permission boundaries.
- Corporate platform and infrastructure teams using it as an internal PaaS, who expect audit trails, SSO and compliance-friendly defaults.
- People migrating from Dokploy or Coolify who want the same workflows on far fewer resources.

The shared job: get code from Git (or an image) to a running, HTTPS-served service with live logs, and keep it healthy, without becoming a full-time operator.

## Product Purpose

FerroDock is a self-hosted PaaS control plane. It deploys applications, Docker Compose stacks and managed databases on servers the user owns, routes them through Traefik with automatic certificates, and shows live status, logs and metrics. Success means a user installs it with one command, deploys their first service in minutes, and forgets the panel is consuming anything.

## Positioning

- **Minimum footprint:** a single Rust binary with an embedded web UI; the panel idles around 20 MiB of memory, so the server's resources go to the user's workloads.
- **Secure by default:** no shell string interpolation, encrypted secrets at rest, one-time setup token, scoped API keys, rate limiting, audit log and signed, health-gated self-updates out of the box.
- **Genuinely open source:** 100% Apache-2.0 with no paid tiers, license keys or gated features.

## Operating Context

- Installed on a fresh Linux server with `curl ... | sudo sh`; runs as Docker Compose (panel, PostgreSQL, Traefik), no Docker Swarm.
- Used from a desktop browser during setup and debugging, and from a phone to check a deploy, read logs or restart a service.
- Integrates with Git providers (GitHub, GitLab, Gitea), container registries, S3-compatible backup destinations and notification channels as phases land.
- Remote servers are managed agentlessly over SSH.

## Capabilities and Constraints

- Built so far (phases 0 to 2, release 0.3):
  - Foundation: setup token, sign-in, lockout, scoped API keys, four roles, audit log, REST API with OpenAPI contract.
  - Applications: image or Git sources (Dockerfile or Compose), blue-green and recreate deploys, cancel, rollback and image pruning. Compose stacks are audited before every deploy unless the owner marks them as trusted.
  - Domains: Traefik routing with Let's Encrypt.
  - Configuration: encrypted variables, ports and volumes.
  - Databases: six managed engines.
  - Backups: scheduled database and volume backups to local disk or S3, plus the panel's own backups.
  - Terminal: a full-featured web terminal.
  - Live view and updates: live status, logs and stats; signed, health-gated self-update with rollback; a command palette.
- Next (phase 3): Git push webhooks, team member invitations, remote servers over SSH, metrics charts and alerts, notifications, preview deployments, scheduled tasks.
- Later (phase 4): TOTP, an audit log screen, Nixpacks and static builds, terminal recording, SQLite, Dokploy import.
- Hierarchy and terminology: organization, project, environment, service (application, compose stack, database), deployment, domain, server.
- Languages: English and Turkish from day one; every user-facing string is translatable and a new language is added by dropping a locale folder.
- Accessibility and reach: fully responsive down to phone width.
- No billing, subscriptions or license keys.
- Undecided: pricing of any hosted offering (none planned), public domain for the installer (currently GitHub releases), whether a GitHub organization will own the project.

## Brand Commitments

- Name: FerroDock. License: Apache-2.0. Repository: github.com/beratr061/FerroDock.
- Interface must stay simpler than Dokploy: few top-level areas, a handful of tabs per service, advanced settings tucked away.
- Real-time information updates live; motion is smooth and never janky.

## Evidence on Hand

- Measured: panel container idles at about 18 MiB RSS with a read-only root filesystem and all capabilities dropped; initial JavaScript is about 118 KB gzipped, against a 120 KB budget enforced in CI.
- No users, testimonials, customer logos, benchmarks against competitors or press exist yet; do not fabricate them.

## Product Principles

1. The panel gets out of the way: every feature is weighed against memory, CPU and attention it costs the user.
2. Safe defaults beat configurable safety: secure behavior ships on, and loosening it is an explicit choice.
3. Show what is actually happening: status, logs and deploy progress reflect real system state, never decorative placeholders.
4. One obvious path per task, with depth available but out of sight.
5. Open by construction: nothing a user depends on is behind a paywall or a proprietary component.

## Accessibility & Inclusion

WCAG 2.2 AA is the floor for every surface, with AAA contrast for primary and secondary text; full keyboard operation, visible focus and reduced-motion support are required.
