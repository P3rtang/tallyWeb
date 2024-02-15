default:
	cargo leptos build

dev:
	# install dependencies
	sudo apt install gcc pkg-config libssl-dev
	cargo install cargo-leptos
	cargo install sqlx-cli

	# setup the dev database
	docker container stop postgres_tallyweb
	docker rm postgres_tallyweb
	docker run -dit --name postgres_tallyweb -p 5432:5432 --env-file .env postgres
	docker cp postgres/00-recreate-db.sql postgres_tallyweb:/
	sleep 5
	docker exec -d postgres_tallyweb psql -U postgres -f /00-recreate-db.sql

	sleep 4
	# setup sqlx migrations
	sqlx database create
	sqlx migrate run

dump-db:
	mkdir -p db-backup
	docker exec -t postgres_tallyweb pg_dump -U p3rtang -d tally_web > "db-backup/dbdump.sql"

reset-db:
	# reset the database
	sqlx database reset -f -y

	# populate the database with a user
	docker exec postgres_tallyweb mkdir -p /postgres
	docker cp .github/postgres_setup/test_user.sql postgres_tallyweb:/postgres/test_user.sql
	docker exec postgres_tallyweb psql -U p3rtang -d tally_web -f /postgres/test_user.sql


test: reset-db
	# run the tests
	cargo leptos test
	cargo leptos end-to-end

setup-pgadmin:
	docker stop pgadmin
	docker container rm pgadmin
	docker run --name pgadmin --env-file .env --restart always --network host -d dpage/pgadmin4
