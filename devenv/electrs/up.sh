#!/bin/bash

./build.sh
docker compose -f docker-compose-electrs.yml up -d
