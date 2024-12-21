psql -U postgres -d postgres -h localhost -p $(POSTGRES_PORT) -w -c " \
    select pg_terminate_backend(pid) from pg_stat_activity where datname='$(PGDATABASE)'; \
"
psql -U postgres -d postgres -h localhost -p $(POSTGRES_PORT) -w -c "DROP DATABASE IF EXISTS $(PGDATABASE)"
psql -U postgres -d postgres -h localhost -p $(POSTGRES_PORT) -w -c "CREATE DATABASE $(PGDATABASE) OWNER $(POSTGRES_USERNAME)"

# setup sqlx migrations
sqlx database create
sqlx migrate run

psql -U p3rtang -d $(PGDATABASE) -h localhost -p $(POSTGRES_PORT) -w -f ".github/postgres_setup/setup-test.sql"
