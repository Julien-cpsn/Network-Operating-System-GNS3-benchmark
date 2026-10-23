#!/usr/bin/env bash
USER="$1"

set -e

sudo-g5k

sudo apt install -y \
    python3 \
    qemu-kvm \
    qemu-utils \
    libvirt-clients \
    libvirt-daemon-system \
    virtinst \
    ca-certificates \
    curl \
    gnupg2 \
    telnet

# The VMs are started by the benchmark user and need KVM
if [ ! -w /dev/kvm ]; then
        sudo chmod o+rw /dev/kvm
fi

cd "/home/${USER}/survey/Network-Operating-System-GNS3-benchmark"

rm -rf experimentation/results/*

chmod +x run_experiment.sh
./run_experiment.sh

if [ ! -d "/home/${USER}/public/results" ]; then
        mkdir "/home/${USER}/public/results"
fi

cp -r "experimentation/results" "/home/${USER}/public/"

echo DONE