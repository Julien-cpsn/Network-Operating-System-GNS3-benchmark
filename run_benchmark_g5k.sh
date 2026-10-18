#!/usr/bin/env bash
USER="$1"

set -e

sudo-g5k

sudo apt install -y \
    python3 \
    python3-pip \
    pipx \
    python3-pyqt6 \
    python3-pyqt6.qtwebsockets \
    python3-pyqt6.qtsvg \
    qemu-kvm \
    qemu-utils \
    libvirt-clients \
    libvirt-daemon-system \
    virtinst \
    ca-certificates \
    curl \
    gnupg2 \
    libpcap-dev

pipx install gns3-server==2.2.56.1 --force

if [ ! -f /usr/local/bin/ubridge ]; then
        if [ ! -d "/home/${USER}/ubridge" ]; then
                git clone https://github.com/GNS3/ubridge.git
        fi
        cd ubridge
        make
        sudo make install
fi

cd "/home/${USER}/survey/Network-Operating-System-GNS3-benchmark"

"/home/${USER}/.local/bin/gns3server" --local > gns3_server.log 2>&1 &

sleep 2

rm -rf experimentation/results/*

chmod +x run_experiment.sh
./run_experiment.sh

if [ ! -d "/home/${USER}/public/results" ]; then
        mkdir "/home/${USER}/public/results"
fi

cp -r "experimentation/results" "/home/${USER}/public/"

echo DONE