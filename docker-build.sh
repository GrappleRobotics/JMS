#!/bin/bash

docker build -t jaci/jms:$(cat VERSION) -t jaci/jms:latest -f Dockerfile .
docker build -t jaci/jms-ui:$(cat VERSION) -t jaci/jms-ui:latest -f Dockerfile.ui .

mkdir -p docker_images

docker image save jaci/jms:latest -o docker_images/jms.tar
docker image save jaci/jms-ui:latest -o docker_images/jms-ui.tar
