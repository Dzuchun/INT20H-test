# build back
FROM rust:latest as back-builder

WORKDIR /app

COPY backend backend
COPY common common

WORKDIR /app/backend

RUN cargo build --release

# build front
FROM rust:latest as front-builder

# wasm target
RUN rustup target add wasm32-unknown-unknown
# binstall
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
# trunk
RUN cargo-binstall trunk

WORKDIR /app
COPY frontend frontend
COPY common common
COPY leptos-flavour leptos-flavour 

WORKDIR /app/frontend

# compile-front
RUN trunk build --release

FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

RUN ln -fs /usr/share/zoneinfo/Europe/Kiev /etc/localtime

RUN apt-get update && apt-get install -y \
    postgresql postgresql-contrib libpq-dev libpq5 libssl-dev && \
    rm -rf /var/lib/apt/lists/*

RUN service postgresql start && \
    su - postgres -c "psql -c \"ALTER USER postgres WITH PASSWORD 'password';\"" && \
    su - postgres -c "psql -c \"CREATE DATABASE mydb;\"" && \
    su - postgres -c "psql -c \"GRANT ALL PRIVILEGES ON DATABASE mydb TO postgres;\""

COPY --from=back-builder /app/backend/target/release/backend /usr/local/bin/backend_server
COPY --from=front-builder /app/frontend/dist /usr/serve/

COPY backend/config.toml config.toml
COPY backend/migrations/1_everything/up.sql /docker-entrypoint-initdb.d/up.sql

EXPOSE 80

CMD service postgresql start && \
    su - postgres -c "psql -d mydb -f /docker-entrypoint-initdb.d/up.sql" && \
    /usr/local/bin/backend_server
