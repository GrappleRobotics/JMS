#!/bin/bash

set -e

docker build -t jaci/jms:$(cat VERSION) -t jaci/jms:latest -f Dockerfile .
docker build -t jaci/jms-ui:$(cat VERSION) -t jaci/jms-ui:latest -f Dockerfile.ui .
docker build -t jaci/jms-nginx:$(cat VERSION) -t jaci/jms-nginx:latest -f nginx/Dockerfile nginx

mkdir -p build/docker-images

docker image save jaci/jms:latest -o build/docker-images/jms.tar
docker image save jaci/jms-ui:latest -o build/docker-images/jms-ui.tar
docker image save jaci/jms-nginx:latest -o build/docker-images/jms-nginx.tar
cp docker-compose.yml build/docker-images