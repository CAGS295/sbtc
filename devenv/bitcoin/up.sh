#!/bin/bash

./build.sh
docker compose -f docker-compose-bitcoin.yml up -d
