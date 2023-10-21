while true
do
    file_name="$(date +"%F_%T")"
    docker exec -t postgres_tallyweb pg_dump -U p3rtang -d tally_web > "db-backup/dbdump_${file_name}"
    echo "backup done stored as dbdump_${file_name}"
    sleep 3600
done
