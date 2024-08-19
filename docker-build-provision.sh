#!/bin/bash
set -e

docker build -t jms-provision-builder:latest -f Dockerfile.provision .
mkdir -p output
docker run -v `pwd`/provision:/work/provision:ro -v `pwd`/docker_images:/work/docker_images:ro -v `pwd`/output:/work/output -v `pwd`/cache:/work/cache jms-provision-builder:latest ./provision/build.sh