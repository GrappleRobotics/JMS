#!/bin/sh
set -e

if [ ! -f ./cache/Rocky-9.4-x86_64-minimal.iso ]; then
  mkdir -p ./cache
  wget -O ./cache/Rocky-9.4-x86_64-minimal.iso https://download.rockylinux.org/pub/rocky/9/isos/x86_64/Rocky-9.4-x86_64-minimal.iso
fi
rm ./output/JMS-Rocky-9.4-x86_64-minimal.iso 2> /dev/null
mkdir -p ./output
mkksiso --add ./k8s --add ./docker_images --ks ./provision/kickstart/JMS-Rocky-9-Headless.ks ./cache/Rocky-9.4-x86_64-minimal.iso ./output/JMS-Rocky-9.4-x86_64-minimal.iso