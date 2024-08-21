#!/bin/sh

set -e

OUR_ADMIN_IP=`ip addr | grep -Eo "10.0.100.[0-9]+/[0-9]+" | cut -f1 -d "/"`

if [ ! -z "$OUR_ADMIN_IP" ]; then
  # Fix the route
  ip route del default
  ip route add default via 10.0.100.1
fi

exec "$@"