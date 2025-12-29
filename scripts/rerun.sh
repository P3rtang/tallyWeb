#!/usr/bin/env bash

PORT=3000

is_port_free() {
    # Returns 0 if port is available, 1 if in use
    ! lsof -iTCP:"$PORT" -sTCP:LISTEN >/dev/null 2>&1
}

podman compose up -d postgres

while true; do
    cargo leptos serve &
    pid=$!

    while true; do
        echo
        echo "Press 'r' to rerun, or Ctrl-C to exit."
        read -n 1 -s k
        if [[ "$k" == "r" ]]; then
            kill -TERM "$pid"
            killall tallyweb-frontend

            while ! is_port_free; do
                echo "Waiting for port $PORT to be free..."
                sleep 1
            done

            break
        fi
    done
done

podman compose down
