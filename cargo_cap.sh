#!/bin/bash

set -e;

MODE="debug"
BUILD_ARGS=()
PARSING_ARGS="1"
ACTION=$1
shift

while [[ $# -gt 0 && "$PARSING_ARGS" != 0 ]]
do
case $1 in
  --)
    PARSING_ARGS=0
    shift
    ;;
  --release)
    BUILD_ARGS+=("--release")
    MODE="release"
    shift
    ;;
  *)
    BUILD_ARGS+=("$1")
    shift
    ;;
esac
done

case $ACTION in
  build)
    set -x
    cargo build "${BUILD_ARGS[@]}"
    sudo setcap cap_net_admin=eip target/$MODE/jms
    ;;
  run)
    set -x
    cargo build "${BUILD_ARGS[@]}"
    sudo setcap cap_net_admin=eip target/$MODE/jms
    target/$MODE/jms $*
    ;;
esac