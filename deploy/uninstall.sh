#!/bin/sh
set -eu

CONFIG_DIR=/etc/ferrodock

[ "$(id -u)" -eq 0 ] || { echo "run as root" >&2; exit 1; }

if [ -f "$CONFIG_DIR/docker-compose.yml" ]; then
    COMPOSE_PROFILES=proxy docker compose -f "$CONFIG_DIR/docker-compose.yml" down
fi

if [ "${1:-}" = "--purge" ]; then
    printf 'This deletes all FerroDock data, the database and %s. Type "delete" to continue: ' "$CONFIG_DIR"
    read -r answer
    [ "$answer" = "delete" ] || { echo "aborted"; exit 1; }
    COMPOSE_PROFILES=proxy docker compose -f "$CONFIG_DIR/docker-compose.yml" down --volumes 2>/dev/null || true
    docker network rm ferrodock-network 2>/dev/null || true
    rm -rf "$CONFIG_DIR" /var/lib/ferrodock
    echo "FerroDock and its data were removed."
else
    echo "FerroDock stopped. Data is kept in $CONFIG_DIR, /var/lib/ferrodock and Docker volumes; run with --purge to delete it."
fi
