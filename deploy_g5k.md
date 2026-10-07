# SETUP
## VM
```shell
mkdir ~/survey
mkdir ~/survey/vm
mkdir ~/survey/vm/Debian
mkdir ~/survey/vm/VyOS
```

```shell
scp ~/Programmation/vm/Debian/debian-trixie.qcow2 nancy.g5k:/home/jcaposie/vm/Debian
scp ~/Programmation/vm/VyOS/vyos-2025.04.04-0018-rolling-generic-amd64.iso nancy.g5k:/home/jcaposie/vm/VyOS
```

## GIT
```shell
cd ~/survey
git clone https://github.com/Julien-cpsn/Network-Operating-System-GNS3-benchmark.git
cd Network-Operating-System-GNS3-benchmark
#>>>> Edit the experimentation files and the .env file
cargo run -- generate files --override
```

## NIX
```shell
curl -L -O https://raw.githubusercontent.com/oar-team/nix-user-chroot-companion/master/nix-user-chroot.sh
chmod +x nix-user-chroot.sh
./nix-user-chroot.sh 
echo "experimental-features = nix-command flakes" > ~/.config/nix/nix.conf
rm ~/.nix-channels
echo <<EOF > ~/.nix-channels
https://github.com/nix-community/home-manager/archive/release-26.05.tar.gz home-manager
https://nixos.org/channels/nixos-unstable nixos-unstable
https://nixos.org/channels/nixos-26.05 nixpkgs
EOF
nix-channel --update
nix-shell '<home-manager>' -A install
. "$HOME/.nix-profile/etc/profile.d/hm-session-vars.sh"
home-manager build
home-manager switch
zsh
```

# XP
Add the following script to your G5K frontend
```shell
#!/bin/bash
OS="$1"

gns3server --local > gns3_server.log &

cd ~/survey/Network-Operating-System-GNS3-benchmark
cargo run -- generate commands --os "$OS" -q > run_experiment.sh
chmod +x run_experiment.sh
./run_experiment.sh

find experimentation/results -type d -name "*$OS*" -exec cp -r "{}" ~/public/results \;
```

Run experiments
```shell
# Read needed time for your OS using:
cargo run -- generate commands --os FreeBSD
# Then change the time accordingly:
oarsub -q production -p grappe -l host=1,walltime=3 'sh "./nix-user-chroot.sh" && ./run_experiment.sh FreeBSD'
```

# Check and download results in
https://api.grid5000.fr/stable/sites/nancy/public/jcaposie/