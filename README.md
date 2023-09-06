# TallyWeb

### docker setup files
`docker cp ../postgres/00-recreate-db.sql postgres_tallyWeb:/postgres/`
`docker cp ../postgres/01-create-schema.sql postgres_tallyWeb:/postgres/`
`docker exec -it postgres_tallyWeb psql -U postgres -a -f /postgres/00-recreate-db.sql`
`docker exec -it postgres_tallyWeb psql -U postgres -d tally_web -a -f /postgres/01-create-schema.sql`
