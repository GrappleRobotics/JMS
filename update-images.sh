#!/bin/bash

set -e

scp docker_images/* fta@10.0.100.10:/home/fta/docker_images
ssh fta@10.0.100.10 -t 'sudo sh -c "/var/lib/rancher/rke2/bin/ctr --address /run/k3s/containerd/containerd.sock --namespace k8s.io image import /home/fta/docker_images/jms.tar; /var/lib/rancher/rke2/bin/ctr --address /run/k3s/containerd/containerd.sock --namespace k8s.io image import /home/fta/docker_images/jms-ui.tar"'