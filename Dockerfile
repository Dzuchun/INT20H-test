FROM rust:latest as builder

WORKDIR /app

COPY . .

WORKDIR /app/backend

RUN cargo build --release

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

COPY --from=builder /app/target/release/backend /usr/local/bin/backend_server

COPY backend/config.toml config.toml
COPY backend/migrations/1_everything/up.sql /docker-entrypoint-initdb.d/up.sql

EXPOSE 80

CMD service postgresql start && \
    su - postgres -c "psql -d mydb -f /docker-entrypoint-initdb.d/up.sql" && \
    /usr/local/bin/backend_server
