# syntax=docker/dockerfile:1

FROM node:25-bookworm-slim AS web
WORKDIR /src/web
RUN corepack enable
COPY web/package.json web/pnpm-lock.yaml ./
RUN --mount=type=cache,target=/root/.local/share/pnpm/store pnpm install --frozen-lockfile
COPY openapi.json /src/openapi.json
COPY web/ ./
RUN pnpm build

FROM rust:1.99.0-slim-bookworm AS backend
ARG FERRODOCK_RELEASE_PUBLIC_KEY=""
WORKDIR /src
ENV SQLX_OFFLINE=true \
    FERRODOCK_RELEASE_PUBLIC_KEY=${FERRODOCK_RELEASE_PUBLIC_KEY}
COPY Cargo.toml Cargo.lock rust-toolchain.toml clippy.toml ./
COPY .sqlx ./.sqlx
COPY crates ./crates
COPY xtask ./xtask
COPY --from=web /src/web/dist ./web/dist
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p ferrodock \
    && cp target/release/ferrodock /usr/local/bin/ferrodock

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --home-dir /var/lib/ferrodock ferrodock \
    && mkdir -p /var/lib/ferrodock /etc/ferrodock \
    && chown ferrodock:ferrodock /var/lib/ferrodock /etc/ferrodock
COPY --from=backend /usr/local/bin/ferrodock /usr/local/bin/ferrodock
USER ferrodock
ENV FERRODOCK_BIND_ADDR=0.0.0.0:3000
EXPOSE 3000
HEALTHCHECK --interval=15s --timeout=5s --start-period=20s --retries=3 \
    CMD ["ferrodock", "healthcheck"]
ENTRYPOINT ["ferrodock"]
CMD ["serve"]
