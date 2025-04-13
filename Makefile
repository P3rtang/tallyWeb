include .env
export $(shell sed 's/=.*//' .env)

default: build

build:
	podman-compose up -d postgres
	cargo leptos build
	podman-compose down

dev:
	# install dependencies
	sudo apt install gcc pkg-config libssl-dev
	cargo install cargo-leptos
	cargo install sqlx-cli

reset: recreate-docker recreate-user recreate-db

recreate-docker:
	podman-compose down
	podman-compose up -d postgres
	psql -U postgres -d postgres -h localhost -p $(POSTGRES_PORT) -w -c "DROP DATABASE IF EXISTS tally_web"
	timeout 10s bash -c "until podman exec $(POSTGRES_CONTAINER) pg_isready ; do sleep .5 ; done"

recreate-user:
	psql -U postgres -d postgres -h localhost -p $(POSTGRES_PORT) -w -c "DROP USER IF EXISTS $(POSTGRES_USERNAME)"
	psql -U postgres -d postgres -h localhost -p $(POSTGRES_PORT) -w -c "CREATE USER $(POSTGRES_USERNAME) PASSWORD '$(POSTGRES_PASSWORD)' CREATEDB"

recreate-db:
	sh ./scripts/recreate-db.sh $(POSTGRES_PORT) $(PGDATABASE) $(POSTGRES_USERNAME)

dump-db:
	mkdir -p db-backup
	podman exec -t $(POSTGRES_CONTAINER) pg_dump --data-only -U p3rtang -d tally_web > "db-backup/dbdump.sql"

watch-style:
	stylance -w ./frontend --output-file ./style/bundle.scss

test: recreate-db check-fmt
	podman-compose up -d postgres
	sleep 1
	cargo leptos test
	cargo leptos end-to-end -r
	podman-compose down

setup-pgadmin:
	podman stop pgadmin
	podman container rm pgadmin
	podman run --name pgadmin --env-file .env --restart always --network host -d dpage/pgadmin4

fmt:
	cargo fmt -q --all
	leptosfmt -q components
	leptosfmt -q frontend

check:
	cargo fmt -q --all --check
	leptosfmt -q --check .
	cargo clippy

check-fmt:
	podman-compose up -d postgres
	sleep 1
	cargo fmt -q --check --all
	leptosfmt -q --check *src/*
	cargo clippy -- -D warnings
	podman-compose down

serve:
	bash -c " \
		trap 'podman-compose down' SIGINT; \
		podman-compose up -d postgres; \
		cargo leptos serve \
	"

watch:
	bash -c " \
		trap 'podman-compose down' SIGINT; \
		podman-compose up -d postgres; \
		cargo leptos watch --hot-reload \
	"

start:
	bash -c " \
		trap 'podman-compose down' SIGINT; \
		podman-compose up -d postgres; \
		cargo leptos serve \
	"
