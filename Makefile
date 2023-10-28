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
