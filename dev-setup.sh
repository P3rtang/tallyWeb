#!/bin/bash

# install dependencies
sudo apt install gcc pkg-config libssl-dev
cargo install cargo-leptos
cargo install sqlx-cli

# setup the dev database
docker rm postgres_tallyweb
docker run -d --name postgres_tallyweb -p 5432:5432 --env-file .env postgres
docker cp postgres/00-recreate-db.sql postgres_tallyweb:/
docker exec postgres_tallyweb psql -U postgres -f /00-recreate-db.sql

# setup sqlx migrations
sqlx database create
sqlx migrate run
