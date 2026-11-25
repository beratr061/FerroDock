#!/bin/sh
set -eu

FERRODOCK_VERSION="${FERRODOCK_VERSION:-0.1.1}"
FERRODOCK_IMAGE="${FERRODOCK_IMAGE:-ghcr.io/beratr061/ferrodock:${FERRODOCK_VERSION}}"
FERRODOCK_ASSETS_URL="${FERRODOCK_ASSETS_URL:-https://github.com/beratr061/FerroDock/releases/download/v${FERRODOCK_VERSION}}"
PANEL_DOMAIN="${FERRODOCK_DOMAIN:-}"
ACME_EMAIL="${FERRODOCK_ACME_EMAIL:-}"
PANEL_PORT="${FERRODOCK_PANEL_PORT:-3000}"
PROXY_MODE="${FERRODOCK_PROXY:-traefik}"
CONFIG_DIR=/etc/ferrodock
LOCAL_ASSETS="${FERRODOCK_LOCAL_ASSETS:-}"

say() { printf '\033[1m==>\033[0m %s\n' "$1"; }
fail() { printf '\033[31merror:\033[0m %s\n' "$1" >&2; exit 1; }

require_root() {
    [ "$(id -u)" -eq 0 ] || fail "run as root, for example: curl -fsSL <url> | sudo sh"
}

check_platform() {
    [ "$(uname -s)" = "Linux" ] || fail "FerroDock supports Linux servers only"
    case "$(uname -m)" in
        x86_64 | amd64 | aarch64 | arm64) ;;
        *) fail "unsupported CPU architecture: $(uname -m)" ;;
    esac
}

port_in_use() {
    ss -ltnH "sport = :$1" 2>/dev/null | grep -q . && ! docker ps --format '{{.Names}}' 2>/dev/null | grep -q '^ferrodock-'
}

check_options() {
    case "$PROXY_MODE" in
        traefik | none) ;;
        *) fail "FERRODOCK_PROXY must be 'traefik' or 'none'" ;;
    esac
    if [ "$PROXY_MODE" = none ] && [ -n "$PANEL_DOMAIN" ]; then
        fail "FERRODOCK_DOMAIN needs the bundled proxy; unset it or drop FERRODOCK_PROXY=none"
    fi
}

required_ports() {
    if [ "$PROXY_MODE" = traefik ]; then
        echo 80 443 "$PANEL_PORT"
    else
        echo "$PANEL_PORT"
    fi
}

check_ports() {
    command -v ss >/dev/null 2>&1 || return 0
    for port in $(required_ports); do
        if port_in_use "$port"; then
            fail "port $port is already in use; stop the service using it and retry"
        fi
    done
}

install_docker() {
    if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
        say "Docker $(docker version --format '{{.Server.Version}}') found"
        return
    fi
    say "Installing Docker Engine from get.docker.com"
    curl -fsSL https://get.docker.com | sh
    systemctl enable --now docker >/dev/null 2>&1 || true
}

random_secret() {
    head -c 48 /dev/urandom | base64 | tr -dc 'A-Za-z0-9' | head -c 40
}

fetch_asset() {
    target="$2"
    if [ -n "$LOCAL_ASSETS" ] && [ -f "$LOCAL_ASSETS/$1" ]; then
        cp "$LOCAL_ASSETS/$1" "$target"
    else
        curl -fsSL "$FERRODOCK_ASSETS_URL/$(echo "$1" | tr '/' '-')" -o "$target"
    fi
}

prepare_directories() {
    mkdir -p "$CONFIG_DIR/secrets" "$CONFIG_DIR/panel" "$CONFIG_DIR/traefik/dynamic" "$CONFIG_DIR/traefik/acme"
    chmod 700 "$CONFIG_DIR/secrets" "$CONFIG_DIR/traefik/acme"
    chown 10001:10001 "$CONFIG_DIR/panel"
    chmod 700 "$CONFIG_DIR/panel"
    fetch_asset docker-compose.yml "$CONFIG_DIR/docker-compose.yml"
}

postgres_uid() {
    image=$(awk '$1 == "image:" && $2 ~ /^postgres:/ { print $2; exit }' "$CONFIG_DIR/docker-compose.yml")
    [ -n "$image" ] || fail "no postgres image found in $CONFIG_DIR/docker-compose.yml"
    docker run --rm --entrypoint id "$image" -u postgres
}

write_secrets() {
    if [ ! -s "$CONFIG_DIR/secrets/postgres_password" ]; then
        umask 077
        random_secret > "$CONFIG_DIR/secrets/postgres_password"
    fi
    uid=$(postgres_uid)
    chown "$uid:$uid" "$CONFIG_DIR/secrets/postgres_password"
    chmod 400 "$CONFIG_DIR/secrets/postgres_password"
    if [ ! -s "$CONFIG_DIR/setup-token" ]; then
        umask 077
        random_secret > "$CONFIG_DIR/setup-token"
    fi
}

write_panel_env() {
    password=$(cat "$CONFIG_DIR/secrets/postgres_password")
    secure=false
    trust_forwarded=false
    if [ -n "$PANEL_DOMAIN" ]; then
        secure=true
        trust_forwarded=true
    fi
    umask 077
    {
        echo "FERRODOCK_DATABASE_URL=postgres://ferrodock:${password}@postgres:5432/ferrodock"
        echo "FERRODOCK_SECURE_COOKIES=${secure}"
        echo "FERRODOCK_TRUST_FORWARDED_FOR=${trust_forwarded}"
        echo "FERRODOCK_SETUP_TOKEN=$(cat "$CONFIG_DIR/setup-token")"
        echo "FERRODOCK_LOG_JSON=true"
        if [ -n "$PANEL_DOMAIN" ]; then
            echo "FERRODOCK_PUBLIC_URL=https://${PANEL_DOMAIN}"
        fi
    } > "$CONFIG_DIR/ferrodock.env"
}

write_traefik_config() {
    [ "$PROXY_MODE" = traefik ] || return 0
    fetch_asset traefik/traefik.yml "$CONFIG_DIR/traefik/traefik.yml"
    sed -i "s|__ACME_EMAIL__|${ACME_EMAIL:-admin@${PANEL_DOMAIN:-localhost}}|" "$CONFIG_DIR/traefik/traefik.yml"
    touch "$CONFIG_DIR/traefik/acme/acme.json"
    chmod 600 "$CONFIG_DIR/traefik/acme/acme.json"
    if [ -n "$PANEL_DOMAIN" ]; then
        fetch_asset traefik/dynamic/panel.yml "$CONFIG_DIR/traefik/dynamic/panel.yml"
        sed -i "s|__PANEL_DOMAIN__|${PANEL_DOMAIN}|g" "$CONFIG_DIR/traefik/dynamic/panel.yml"
    fi
}

docker_socket_gid() {
    [ -S /var/run/docker.sock ] || fail "/var/run/docker.sock not found; FerroDock manages containers through it"
    stat -c %g /var/run/docker.sock
}

write_compose_env() {
    panel_bind=0.0.0.0
    if [ -n "$PANEL_DOMAIN" ]; then
        panel_bind=127.0.0.1
    fi
    profiles=""
    if [ "$PROXY_MODE" = traefik ]; then
        profiles=proxy
    fi
    umask 077
    {
        echo "FERRODOCK_IMAGE=${FERRODOCK_IMAGE}"
        echo "FERRODOCK_PANEL_PORT=${PANEL_PORT}"
        echo "FERRODOCK_PANEL_BIND=${panel_bind}"
        echo "FERRODOCK_DOCKER_GID=$(docker_socket_gid)"
        echo "COMPOSE_PROFILES=${profiles}"
    } > "$CONFIG_DIR/.env"
}

start_stack() {
    docker network inspect ferrodock-network >/dev/null 2>&1 || docker network create ferrodock-network >/dev/null
    say "Starting FerroDock ${FERRODOCK_VERSION}"
    docker compose -f "$CONFIG_DIR/docker-compose.yml" up -d --pull missing
}

wait_until_healthy() {
    attempt=0
    until docker compose -f "$CONFIG_DIR/docker-compose.yml" exec -T ferrodock ferrodock healthcheck >/dev/null 2>&1; do
        attempt=$((attempt + 1))
        [ "$attempt" -lt 60 ] || fail "FerroDock did not become healthy; inspect: docker compose -f $CONFIG_DIR/docker-compose.yml logs"
        sleep 2
    done
}

server_address() {
    ip route get 1.1.1.1 2>/dev/null | awk '{for (i = 1; i <= NF; i++) if ($i == "src") { print $(i + 1); exit }}'
}

print_summary() {
    if [ -n "$PANEL_DOMAIN" ]; then
        url="https://$PANEL_DOMAIN"
    else
        url="http://$(server_address):$PANEL_PORT"
    fi
    say "FerroDock is ready: $url"
    echo "    Open it and create the owner account with this setup token:"
    echo "    $(cat "$CONFIG_DIR/setup-token")"
    echo "    Configuration lives in $CONFIG_DIR; keep $CONFIG_DIR/panel/master.key backed up."
}

main() {
    require_root
    check_platform
    check_options
    install_docker
    check_ports
    prepare_directories
    write_secrets
    write_panel_env
    write_traefik_config
    write_compose_env
    start_stack
    wait_until_healthy
    print_summary
}

main "$@"
