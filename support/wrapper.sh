#!/usr/bin/env bash

base_dir=$(dirname "$0")
cmd_name=$(basename "$0")

exec "$base_dir/depot_tools_rs" "$cmd_name" "$@"
