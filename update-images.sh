#!/bin/bash

set -e

scp docker_images/* fta@10.0.100.10:/jms/docker_images
ssh fta@10.0.100.10 -t 'docker image load --input /jms/docker_images/jms.tar; docker image load --input /jms/docker_images/jms-ui.tar; docker image load --input /jms/docker_images/jms-nginx.tar'