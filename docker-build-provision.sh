#!/bin/sh
docker build -t jms-provision-builder:latest -f Dockerfile.provision .
mkdir -p output
docker run -v ./output:/work/output -v ./cache:/work/cache jms-provision-builder:latest