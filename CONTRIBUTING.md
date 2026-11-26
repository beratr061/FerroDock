# Contributing to FerroDock

## Ground rules

- No code comments. Names carry the meaning; the reasons behind a decision go into the commit message, the pull request or an ADR in `docs/adr/`. Functional directives (`#!`, `# syntax=`) are the only exception.
- Every source file stays at or below 250 lines and every Rust function at or below 40 lines. `cargo xtask check-rules` and clippy enforce this.
- Styles live only in CSS files and Tailwind utilities that reference design tokens. No inline `style`, no hex colors outside `web/src/styles/tokens.css`.
- Features never import each other. Cross-feature effects go through domain events (`EventBus`) on the backend and through the `app` layer on the frontend. ESLint `boundaries/dependencies` enforces the frontend rule.
- Tests must reflect real behavior. Fix the code when the code is wrong; fix the test only when the expectation is wrong.

## Adding a feature

1. Run `cargo xtask new-feature <snake_case_name>`. It creates the domain module, API route, migration, frontend feature folder and locale files.
2. Follow the printed next steps to register the module, route, OpenAPI path and frontend route.
3. Write queries in `crates/db/src/pg/<feature>.rs` behind a repository trait in `crates/db/src/repos.rs`, then run `cargo sqlx prepare --workspace -- --all-targets`.
4. Regenerate the contract with `cargo run -p ferrodock -- openapi > openapi.json`; the frontend types follow from it.

## Adding a language

Copy `web/locales/en` to `web/locales/<code>`, translate the values and set the display name in `meta.json`. The language appears in the selector automatically. Backend errors are returned as codes with parameters, so no backend change is needed.

## Adding a builder, database engine or notifier

Implement the corresponding trait in its own file and register it in the registry next to the trait. Existing implementations stay untouched.

## End-to-end tests

`web/e2e` holds Playwright tests that drive a real panel: setup, a project, an
image deployment with its logs and a shell, a managed database, and a check that
no page is wider than a phone or desktop screen. CI runs them against a fresh
install of the image it just built.

To run them yourself, install a panel (for example with `deploy/install.sh`),
then from `web/`:

```sh
pnpm exec playwright install chromium
FD_E2E_URL=http://127.0.0.1:3000 FD_SETUP_TOKEN="$(sudo cat /etc/ferrodock/setup-token)" pnpm e2e
```

Without `FD_SETUP_TOKEN` the setup step is skipped, so the suite can be rerun
against a panel that is already set up; names are unique per run.

## Pull requests

Each pull request runs formatting, clippy, code rules, tests, sqlx and OpenAPI drift checks, cargo-deny, frontend lint, build and bundle size budget. Security-sensitive changes get a security review before merge.
