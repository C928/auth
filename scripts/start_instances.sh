#!/bin/bash

docker rm redis postgres \

docker run \
    -p 6379:6379 \
    -d \
    --name redis redis

docker run \
    -p 5432:5432 \
    -e POSTGRES_DB=test \
    -e POSTGRES_USER=postgres \
    -e POSTGRES_PASSWORD=password \
    -d \
    --name postgres postgres

sqlx migrate run
