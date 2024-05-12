#!/bin/bash
docker save jaci/jms:latest | gzip | pv | ssh fta@jms-primary.jms.local docker load