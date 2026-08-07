#!/usr/bin/bash
SERVER_URL="http://127.0.0.1:7667"
curl -sL -d '{"job":1, "system":"pluto", "account":"cscs", "user":"vagrant"}' \
-H 'Content-Type: application/json' \
${SERVER_URL}/get_config | \
jq .
