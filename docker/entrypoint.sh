#!/bin/sh
# A Fly volume arrives owned by root. Hand /data to the account that runs the
# server, then drop privileges — the platform executes code visitors submit.
set -e

mkdir -p /data
chown -R mts:mts /data

exec gosu mts "$@"
