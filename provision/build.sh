#!/bin/sh
set -e

if [ ! -f ./build/cache/Rocky-9.4-x86_64-minimal.iso ]; then
  mkdir -p ./build/cache
  wget -O ./build/cache/Rocky-9.4-x86_64-minimal.iso https://download.rockylinux.org/pub/rocky/9/isos/x86_64/Rocky-9.4-x86_64-minimal.iso
fi
rm ./build/iso/JMS-Rocky-9.4-x86_64-minimal.iso || true 2> /dev/null
mkdir -p ./build/iso
mkksiso --add ./build/docker-images --ks ./provision/kickstart/JMS-Rocky-9-Headless.ks ./build/cache/Rocky-9.4-x86_64-minimal.iso ./build/iso/JMS-Rocky-9.4-x86_64-minimal.iso