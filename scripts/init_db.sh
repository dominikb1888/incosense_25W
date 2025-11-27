#!/usr/bin/env bash
set -xeuo pipefail

# ----------------------------
# Ensure dependencies exist
# ----------------------------
if ! command -v psql >/dev/null; then
  echo >&2 "Error: psql is not installed."
  exit 1
fi

if ! command -v sqlx >/dev/null; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Install with:"
  echo >&2 "    cargo install --version '~0.8.6' sqlx-cli --no-default-features --features rustls,postgres"
  exit 1
fi

# ----------------------------
# Load configuration from environment
# ----------------------------
DB_USER="${APP__DATABASE__USERNAME:?Must be set}"
DB_PASSWORD="${APP__DATABASE__PASSWORD:?Must be set}"
DB_NAME="${APP__DATABASE__DATABASE_NAME:?Must be set}"
DB_PORT="${APP__DATABASE__PORT:?Must be set}"
DB_HOST="${APP__DATABASE__HOST:?Must be set}"

export DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

# ----------------------------
# Wait for Postgres
# ----------------------------
echo >&2 "Waiting for Postgres at ${DB_HOST}:${DB_PORT}..."
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
  echo >&2 "Postgres is still unavailable - sleeping 1s..."
  sleep 1
done

echo >&2 "Postgres is up - running migrations."

# ----------------------------
# Run migrations
# ----------------------------
sqlx database create
sqlx migrate run

echo >&2 "Database ready!"
echo >&2 "Postgres has been migrated, ready to go!"
