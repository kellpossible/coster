#!/bin/sh

echo "this requires coster to be running on http://localhost:8000,"
echo "and have graphqurl installed via npm install -g graphqurl"

gq --introspect http://localhost:8000/api > src/graphql/schema.graphql