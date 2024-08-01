#!/bin/sh
set -e

docker build -t jms-provision-builder:latest -f Dockerfile.provision .
mkdir -p output
docker run -v ./provision:/work/provision:ro -v ./k8s:/work/k8s:ro -v ./docker_images:/work/docker_images:ro -v ./output:/work/output -v ./cache:/work/cache jms-provision-builder:latest ./provision/build.sh