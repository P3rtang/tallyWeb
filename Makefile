include .env
export $(shell sed 's/=.*//' .env)

default:
	cargo leptos build

dev:
	# install dependencies
	sudo apt install gcc pkg-config libssl-dev
	cargo install cargo-leptos
	cargo install sqlx-cli

reset: recreate-docker recreate-user recreate-db

recreate-docker:
	docker stop tallyweb-postgres
	docker rm tallyweb-postgres
	docker run -d --name tallyweb-postgres -p $(POSTGRES_PORT):5432 --env-file .env postgres
	timeout 10s bash -c "until docker exec $(POSTGRES_CONTAINER) pg_isready ; do sleep .5 ; done"

recreate-user:
	psql -U postgres -d postgres -h localhost -p $(POSTGRES_PORT) -w -c "DROP USER IF EXISTS $(POSTGRES_USERNAME)"
	psql -U postgres -d postgres -h localhost -p $(POSTGRES_PORT) -w -c "CREATE USER $(POSTGRES_USERNAME) PASSWORD '$(POSTGRES_PASSWORD)' CREATEDB"

recreate-db:
	psql -U postgres -d postgres -h localhost -p $(POSTGRES_PORT) -w -c " \
		select pg_terminate_backend(pid) from pg_stat_activity where datname='$(PGDATABASE)'; \
	"
	psql -U postgres -d postgres -h localhost -p $(POSTGRES_PORT) -w -c "DROP DATABASE IF EXISTS $(PGDATABASE)"
	psql -U postgres -d postgres -h localhost -p $(POSTGRES_PORT) -w -c "CREATE DATABASE $(PGDATABASE) OWNER $(POSTGRES_USERNAME)"

	# setup sqlx migrations
	sqlx database create
	sqlx migrate run

	psql -U p3rtang -d $(PGDATABASE) -h localhost -p $(POSTGRES_PORT) -w -f ".github/postgres_setup/setup-test.sql"

dump-db:
	mkdir -p db-backup
	docker exec -t $(POSTGRES_CONTAINER) pg_dump --data-only -U p3rtang -d tally_web > "db-backup/dbdump.sql"

watch-style:
	stylance -w ./frontend/ --output-file ./style/bundle.scss

test: recreate-db check-fmt
	# run program tests
	cargo leptos test
	cargo leptos end-to-end -r

setup-pgadmin:
	docker stop pgadmin
	docker container rm pgadmin
	docker run --name pgadmin --env-file .env --restart always --network host -d dpage/pgadmin4

fmt:
	cargo fmt -q --all
	leptosfmt -q components
	leptosfmt -q frontend

check-fmt:
	cargo fmt -q --check --all
	leptosfmt -q --check *src/*
	cargo clippy -- -D warnings
