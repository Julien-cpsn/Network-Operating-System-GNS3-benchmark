# DEPLOY (automated)
The `deploy_g5k` command uploads the images of an OS and reserves a node for its experiment.
```shell
cargo run -- deploy_g5k <USER> <SITE> <OS> [--queue abaca] [--cluster grappe] [--walltime 3:00] [--script PATH] [--no-wait]
# Example
cargo run -- deploy_g5k jcaposie nancy FreeBSD --walltime 3
```
It asks for your Grid'5000 password (only used for the Grid'5000 API), then:
1. connects over SSH to `access.grid5000.fr` (SSH agent or `~/.ssh/id_*` key, host key checked against `~/.ssh/known_hosts`)
2. clears `~/<SITE>/survey/vm` and uploads the `images_path` files of `<OS>` (from `experimentation/operating_systems.toml`) to `~/<SITE>/survey/vm/<OS>`
3. submits a job running `/home/<USER>/run_benchmark.sh "<USER>" "<OS>"` (override with `--script`) and waits for it to terminate (unless `--no-wait`)

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