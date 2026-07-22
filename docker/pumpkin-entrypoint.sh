#!/usr/bin/env sh
set -eu

# The Railway volume is mounted over /pumpkin, so image-provided plugins must
# be installed into the persistent directory when the container starts.
mkdir -p /pumpkin/plugins
cp /opt/pumpkin/plugins/libpatchbukkit.so /pumpkin/plugins/libpatchbukkit.so

exec /bin/pumpkin "$@"
