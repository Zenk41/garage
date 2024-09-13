#!/bin/sh

# Create the garage.toml configuration file
cat <<EOF > /etc/garage.toml
metadata_dir = "/var/lib/garage/meta"
data_dir = "/var/lib/garage/data"

replication_mode = "none"

rpc_bind_addr = "[::]:3901"
rpc_secret = "${GARAGE_RPC_SECRET}"

bootstrap_peers = []

[s3_api]
s3_region = "gibber"
api_bind_addr = "[::]:3900"

[s3_web]
bind_addr = "[::]:3902"
root_domain = "localhost"
EOF
