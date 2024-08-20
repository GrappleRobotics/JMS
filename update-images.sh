#!/bin/bash

set -e

scp build/docker-images/* fta@10.0.100.10:/jms/docker-images
ssh fta@10.0.100.10 -t 'docker image load --input /jms/docker-images/jms.tar; docker image load --input /jms/docker-images/jms-ui.tar; docker image load --input /jms/docker-images/jms-nginx.tar'