# SETUP

```shell
mkdir /home/${USER}/survey
cd /home/${USER}/survey
git clone https://github.com/Julien-cpsn/Network-Operating-System-GNS3-benchmark.git
cd Network-Operating-System-GNS3-benchmark

#>>>> Edit the experimentation files and the .env file

cargo run -- generate files --override
```

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

# Check and download results in
https://api.grid5000.fr/stable/sites/SITE/public/USERNAME/