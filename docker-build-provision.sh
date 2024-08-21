#!/bin/bash
set -e

docker build -t jms-provision-builder:latest -f Dockerfile.provision .
mkdir -p build
docker run -v `pwd`/provision:/work/provision:ro -v `pwd`/build:/work/build jms-provision-builder:latest ./provision/build.sh