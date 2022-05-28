#!/bin/sh
podman run -it --rm --name db -e POSTGRES_PASSWORD=mysecretpassword -e POSTGRES_DB=track-wear -v "$PWD/runtime/db:/var/lib/postgresql/data" -p 5432:5432 docker.io/postgres