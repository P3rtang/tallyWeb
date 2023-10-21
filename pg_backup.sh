#!/bin/bash

backup_num=0

while true
do
    file_name=""
    if (( backup_num % 24 == 0 )); then
        file_name="$(date +"%F")_daily"
    else
        file_name="$(date +"%F_%T")"
    fi

    backup_num+=1

    docker exec -t postgres_tallyWeb pg_dump -U p3rtang -d tally_web > "db-backup/dbdump_${file_name}"
    echo "backup done stored as dbdump_${file_name}"
    sleep 3600
done
