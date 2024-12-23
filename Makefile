include .env
export $(shell sed 's/=.*//' .env)

default: build

build:
	docker compose up -d postgres
	cargo leptos build
	docker compose down

dev:
	# install dependencies
	sudo apt install gcc pkg-config libssl-dev
	cargo install cargo-leptos
	cargo install sqlx-cli

reset: recreate-docker recreate-user recreate-db

recreate-docker:
	docker stop postgres
	docker rm postgres
	docker run -d --name $(POSTGRES_CONTAINER) -p $(POSTGRES_PORT):5432 --env-file .env postgres
	timeout 10s bash -c "until docker exec $(POSTGRES_CONTAINER) pg_isready ; do sleep .5 ; done"

recreate-user:
	psql -U postgres -d postgres -h localhost -p $(POSTGRES_PORT) -w -c "DROP USER IF EXISTS $(POSTGRES_USERNAME)"
	psql -U postgres -d postgres -h localhost -p $(POSTGRES_PORT) -w -c "CREATE USER $(POSTGRES_USERNAME) PASSWORD '$(POSTGRES_PASSWORD)' CREATEDB"

recreate-db:
	sh ./scripts/recreate-db.sh $(POSTGRES_PORT) $(PGDATABASE) $(POSTGRES_USERNAME)

dump-db:
	mkdir -p db-backup
	docker exec -t $(POSTGRES_CONTAINER) pg_dump --data-only -U p3rtang -d tally_web > "db-backup/dbdump.sql"

watch-style:
	stylance -w ./frontend/ --output-file ./style/bundle.scss

test: recreate-db check-fmt
	docker compose up -d postgres
	sleep 1
	cargo leptos test
	cargo leptos end-to-end -r
	docker compose down

setup-pgadmin:
	docker stop pgadmin
	docker container rm pgadmin
	docker run --name pgadmin --env-file .env --restart always --network host -d dpage/pgadmin4

fmt:
	cargo fmt -q --all
	leptosfmt -q components
	leptosfmt -q frontend

check:
	cargo fmt -q --all --check
	leptosfmt -q --check .
	cargo clippy

check-fmt:
	docker compose up -d postgres
	sleep 1
	cargo fmt -q --check --all
	leptosfmt -q --check *src/*
	cargo clippy -- -D warnings
	docker compose down

serve:
	bash -c " \
		trap 'docker compose down' SIGINT; \
		docker compose up -d postgres; \
		cargo leptos serve \
	"

watch:
	bash -c " \
		trap 'docker compose down' SIGINT; \
		docker compose up -d postgres; \
		cargo leptos watch \
	"

start:
	bash -c " \
		trap 'docker compose down' SIGINT; \
		docker compose up -d postgres; \
		cargo leptos serve \
	"
