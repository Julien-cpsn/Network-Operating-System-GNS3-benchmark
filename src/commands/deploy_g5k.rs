use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, bail, Context};
use ini::Ini;
use serde_json::{json, Value};
use ssh2::{CheckResult, KnownHostFileKind, Session};
use tokio::time::sleep;
use tracing::{debug, info, warn};

use crate::args::deploy_g5k::DeployG5kCommand;
use crate::utils::files::network_stacks::parse_network_stack_list_file;
use crate::utils::files::oses::parse_os_list_file;
use crate::utils::files::routing_stacks::parse_routing_stack_list_file;

const TARGET: &str = "deploy_g5k";

const G5K_ACCESS_HOST: &str = "access.grid5000.fr";
const G5K_ACCESS_PORT: u16 = 22;
const G5K_API_URL: &str = "https://api.grid5000.fr/stable";

const SSH_TIMEOUT: Duration = Duration::from_secs(30);
const JOB_POLL_INTERVAL: Duration = Duration::from_secs(5);
const MAX_CONSECUTIVE_POLL_FAILURES: u32 = 6;
const UPLOAD_CHUNK_SIZE: usize = 256 * 1024;

pub async fn deploy_g5k(command: DeployG5kCommand) -> anyhow::Result<()> {
    // Everything below ends up in a remote shell command or in a path, so only accept safe identifiers
    ensure_identifier("username", &command.user, true)?;
    ensure_identifier("site", &command.site, false)?;
    ensure_identifier("cluster", &command.cluster, false)?;
    ensure_identifier("queue", &command.queue, false)?;
    ensure_identifier("operating system name", &command.os, true)?;

    if let Some(walltime) = &command.walltime {
        if walltime.is_empty() || !walltime.chars().all(|c| c.is_ascii_digit() || c == ':') {
            bail!("Invalid walltime \"{walltime}\", expected something like \"3\" or \"3:30\"");
        }
    }

    // Find the images to upload
    let network_stack_list = parse_network_stack_list_file()?;
    let routing_stack_list = parse_routing_stack_list_file()?;
    let os_list = parse_os_list_file(network_stack_list.keys().collect(), routing_stack_list.keys().collect())?;

    let os = os_list.get(&command.os).ok_or_else(|| {
        let available = os_list.keys().cloned().collect::<Vec<_>>().join(", ");
        anyhow!("Operating system \"{}\" is not listed in the operating systems file. Available: {available}", command.os)
    })?;

    if os.images_path.is_empty() {
        bail!("Operating system \"{}\" has no image to upload", command.os);
    }

    for image_path in &os.images_path {
        if !image_path.is_file() {
            bail!("Image \"{}\" does not exist or is not a file", image_path.display());
        }
    }

    let image_paths = os.images_path.clone();
    // Every image of the operating systems file, since the previous experiment may have used another OS
    let all_image_paths: Vec<PathBuf> = os_list.values().flat_map(|os| os.images_path.iter().cloned()).collect();

    let password = tokio::task::spawn_blocking(|| rpassword::prompt_password("Grid5000 password: ")).await??;
    println!();

    let remote_vm_dir = format!("{}/survey/vm", command.site);
    let remote_os_dir = format!("{remote_vm_dir}/{}", command.os);

    {
        let user = command.user.clone();
        let remote_vm_dir = remote_vm_dir.clone();
        let remote_os_dir = remote_os_dir.clone();

        let site = command.site.clone();

        tokio::task::spawn_blocking(move || upload_images(&user, &site, &remote_vm_dir, &remote_os_dir, &image_paths, &all_image_paths)).await??;
    }

    let job_id = submit_job(&command, &password).await?;

    if command.no_wait {
        info!(target: TARGET, "Job {job_id} submitted, not waiting for it (--no-wait)");
        return Ok(());
    }

    wait_for_job(&command, &password, job_id).await?;

    info!(target: TARGET, "Job {job_id} terminated. Its output is in \"~/{}/benchmark-nos-{}.txt\" on Grid'5000", command.site, command.os);

    Ok(())
}

fn ensure_identifier(what: &str, value: &str, allow_upper: bool) -> anyhow::Result<()> {
    let valid = !value.is_empty()
        && value.chars().all(|c| {
            c.is_ascii_lowercase()
                || c.is_ascii_digit()
                || matches!(c, '-' | '_' | '.')
                || (allow_upper && c.is_ascii_uppercase())
        });

    if !valid {
        bail!("Invalid {what} \"{value}\": only letters, digits, '-', '_' and '.' are allowed");
    }

    Ok(())
}

// ---------------------------------------------------------------- SSH

fn upload_images(user: &str, site: &str, remote_vm_dir: &str, remote_os_dir: &str, image_paths: &[PathBuf], all_image_paths: &[PathBuf]) -> anyhow::Result<()> {
    info!(target: TARGET, "Connecting to Grid'5000 SSH ({G5K_ACCESS_HOST})");
    let session = connect_to_ssh(user)?;
    info!(target: TARGET, "SSH connected successfully");

    // Clear the previous experiment. `~` is expanded by the remote shell, the arguments were validated beforehand
    let command = format!("rm -rf -- ~/{remote_vm_dir}");
    info!(target: TARGET, "> {command}");
    ssh_exec(&session, &command)?;

    let command = format!("mkdir -p -- ~/{remote_os_dir}");
    info!(target: TARGET, "> {command}");
    ssh_exec(&session, &command)?;

    // Remove the images of the previous experiment from the remote GNS3, before uploading the new ones
    let deleted = delete_remote_gns3_images(&session, user, site, all_image_paths)?;
    info!(target: TARGET, "Deleted {deleted} image(s) from the remote GNS3");

    for image_path in image_paths {
        let file_name = image_path.file_name().ok_or_else(|| anyhow!("Invalid image path \"{}\"", image_path.display()))?;
        // Relative to the remote home directory, which is where SCP starts
        let remote_path = format!("{remote_os_dir}/{}", file_name.to_string_lossy());

        info!(target: TARGET, "Uploading \"{}\" to \"~/{remote_path}\"", image_path.display());
        scp_upload(&session, image_path, &remote_path, 0o644)?;
    }

    info!(target: TARGET, "All images uploaded");

    session.disconnect(None, "done", None)?;

    Ok(())
}

fn connect_to_ssh(user: &str) -> anyhow::Result<Session> {
    let address = (G5K_ACCESS_HOST, G5K_ACCESS_PORT)
        .to_socket_addrs()
        .with_context(|| format!("Could not resolve {G5K_ACCESS_HOST}"))?
        .next()
        .ok_or_else(|| anyhow!("Could not resolve {G5K_ACCESS_HOST}"))?;

    let tcp = TcpStream::connect_timeout(&address, SSH_TIMEOUT)
        .with_context(|| format!("Could not connect to {G5K_ACCESS_HOST}"))?;

    let mut session = Session::new()?;
    session.set_tcp_stream(tcp);
    session.set_timeout(SSH_TIMEOUT.as_millis() as u32);
    session.handshake().context("SSH handshake failed")?;

    check_host_key(&session)?;
    authenticate(&session, user)?;

    Ok(session)
}

fn home_dir() -> anyhow::Result<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("Could not find your home directory"))
}

/// Verifies the server against `~/.ssh/known_hosts`, like OpenSSH does. Unknown hosts are rejected.
fn check_host_key(session: &Session) -> anyhow::Result<()> {
    let known_hosts_path = home_dir()?.join(".ssh").join("known_hosts");

    let mut known_hosts = session.known_hosts()?;

    if known_hosts_path.exists() {
        known_hosts
            .read_file(&known_hosts_path, KnownHostFileKind::OpenSSH)
            .with_context(|| format!("Could not read \"{}\"", known_hosts_path.display()))?;
    }

    let (key, _) = session.host_key().ok_or_else(|| anyhow!("The server did not present a host key"))?;

    match known_hosts.check_port(G5K_ACCESS_HOST, G5K_ACCESS_PORT, key) {
        CheckResult::Match => Ok(()),
        CheckResult::Mismatch => bail!("The host key of {G5K_ACCESS_HOST} does not match \"{}\". Refusing to connect", known_hosts_path.display()),
        CheckResult::NotFound => bail!("{G5K_ACCESS_HOST} is not in \"{}\". Run `ssh <user>@{G5K_ACCESS_HOST}` once to verify and add its host key", known_hosts_path.display()),
        CheckResult::Failure => bail!("Could not verify the host key of {G5K_ACCESS_HOST}"),
    }
}

/// Tries the SSH agent first, then the default private keys
fn authenticate(session: &Session, user: &str) -> anyhow::Result<()> {
    match session.userauth_agent(user) {
        Ok(()) if session.authenticated() => return Ok(()),
        Ok(()) => {},
        Err(e) => debug!(target: TARGET, "SSH agent authentication failed: {e}"),
    }

    let ssh_dir = home_dir()?.join(".ssh");

    for key_name in ["id_ed25519", "id_ecdsa", "id_rsa"] {
        let key_path = ssh_dir.join(key_name);

        if !key_path.exists() {
            continue;
        }

        match session.userauth_pubkey_file(user, None, &key_path, None) {
            Ok(()) if session.authenticated() => return Ok(()),
            Ok(()) => {},
            Err(e) => debug!(target: TARGET, "Authentication with \"{}\" failed: {e}", key_path.display()),
        }
    }

    bail!("SSH authentication failed for \"{user}\" on {G5K_ACCESS_HOST}. Make sure your key is registered in your Grid'5000 account and loaded in your SSH agent (passphrase-protected keys need the agent)")
}

/// Runs a command and waits for it to complete, so the following steps do not race with it
fn ssh_exec(session: &Session, command: &str) -> anyhow::Result<String> {
    let mut channel = session.channel_session()?;
    channel.exec(command)?;

    let mut stdout = String::new();
    channel.read_to_string(&mut stdout)?;

    let mut stderr = String::new();
    channel.stderr().read_to_string(&mut stderr)?;

    channel.wait_close()?;
    let status = channel.exit_status()?;

    if status != 0 {
        bail!("Remote command \"{command}\" failed with status {status}: {}", stderr.trim());
    }

    Ok(stdout)
}

/// Remote GNS3 configuration, relative to the site home directory
const REMOTE_GNS3_CONFIG: &str = ".config/GNS3/2.2/gns3_server.conf";
/// Remote GNS3 images directory when the configuration does not set one, relative to the site home directory
const REMOTE_GNS3_DEFAULT_IMAGES_DIR: &str = "GNS3/images";

/// Single-quotes a string for a POSIX shell
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// Finds the remote GNS3 images directory, relative to the site home directory (`~/<site>` from the access machine)
fn remote_gns3_images_dir(session: &Session, user: &str, site: &str) -> anyhow::Result<String> {
    let config = ssh_exec(session, &format!("cat \"$HOME\"/{site}/{REMOTE_GNS3_CONFIG} 2>/dev/null || true"))?;

    let configured = Ini::load_from_str(&config)
        .ok()
        .and_then(|ini| ini.section(Some("Server")).and_then(|server| server.get("images_path").map(str::to_string)));

    let Some(images_path) = configured.filter(|path| !path.trim().is_empty()) else {
        return Ok(REMOTE_GNS3_DEFAULT_IMAGES_DIR.to_string());
    };

    // The configuration holds a path as seen from the site frontend, whose home is `~/<site>` from the access machine
    let relative = images_path
        .trim()
        .strip_prefix(&format!("/home/{user}/"))
        .map(|relative| relative.trim_end_matches('/').to_string())
        .ok_or_else(|| anyhow!("The remote GNS3 images path \"{images_path}\" is not inside /home/{user}, cannot reach it from {G5K_ACCESS_HOST}"))?;

    if relative.is_empty() || relative.split('/').any(|part| part == "..") {
        bail!("Unsupported remote GNS3 images path \"{images_path}\"");
    }

    Ok(relative)
}

/// Removes the given images from the GNS3 server of the Grid'5000 site, since GNS3 v2.2 has no API to delete an image.
///
/// The files are removed from `<images_path>/QEMU/<image name>`, along with the `<image name>.md5sum` file GNS3
/// creates next to them. Images that are not in GNS3 are skipped. Returns the number of removed images.
fn delete_remote_gns3_images(session: &Session, user: &str, site: &str, image_paths: &[PathBuf]) -> anyhow::Result<usize> {
    // Only file names are used, so nothing outside of the GNS3 QEMU directory can be touched.
    // Several OSes can share the same image (ex: BSDRP-BIRD2 and BSDRP-FRR).
    let mut seen = HashSet::new();
    let mut image_names = Vec::new();

    for image_path in image_paths {
        let name = image_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("Invalid image path \"{}\"", image_path.display()))?;

        if seen.insert(name) {
            image_names.push(shell_quote(name));
        }
    }

    if image_names.is_empty() {
        return Ok(0);
    }

    let qemu_dir = format!("{}/QEMU", remote_gns3_images_dir(session, user, site)?);
    debug!(target: TARGET, "Remote GNS3 QEMU images directory: ~/{site}/{qemu_dir}");

    // Prints the name of every image it removed
    let command = format!(
        "cd \"$HOME\"/{site}/{} 2>/dev/null || exit 0; for f in {}; do if [ -e \"$f\" ]; then rm -f -- \"$f\" \"$f.md5sum\" && echo \"$f\"; else rm -f -- \"$f.md5sum\"; fi; done",
        shell_quote(&qemu_dir),
        image_names.join(" ")
    );

    info!(target: TARGET, "> Removing the previous images from ~/{site}/{qemu_dir}");
    let output = ssh_exec(session, &command)?;

    for name in output.lines() {
        info!(target: TARGET, "Deleted image \"{name}\" from the remote GNS3");
    }

    Ok(output.lines().count())
}

/// Uploads a file over SCP. `remote_path` is relative to the remote home directory.
///
/// The path is sent quoted to the remote `scp`, so the remote shell never expands `~`: a leading `~/` would be taken
/// as a directory literally named `~` and the transfer rejected with `[Session(-28)] failed to send file`.
/// It is stripped here so that `~/nancy/file.sh` and `nancy/file.sh` both work.
fn scp_upload(session: &Session, local_path: &Path, remote_path: &str, mode: i32) -> anyhow::Result<()> {
    let remote_path = remote_path.strip_prefix("~/").unwrap_or(remote_path);

    let mut file = File::open(local_path).with_context(|| format!("Could not open \"{}\"", local_path.display()))?;
    let size = file.metadata()?.len();

    let mut channel = session
        .scp_send(Path::new(remote_path), mode, size, None)
        .with_context(|| format!("Could not upload \"{}\" to \"~/{remote_path}\" (does the remote directory exist?)", local_path.display()))?;

    let mut buffer = vec![0u8; UPLOAD_CHUNK_SIZE];
    let mut sent: u64 = 0;
    let mut last_logged_percent = 0;

    loop {
        let read = file.read(&mut buffer)?;

        if read == 0 {
            break;
        }

        channel.write_all(&buffer[..read])?;
        sent += read as u64;

        let percent = if size == 0 { 100 } else { sent * 100 / size };

        if percent >= last_logged_percent + 10 {
            last_logged_percent = percent / 10 * 10;
            info!(target: TARGET, "  {last_logged_percent}% ({} / {} MiB)", sent / (1024 * 1024), size / (1024 * 1024));
        }
    }

    // Required for the remote end to acknowledge and finish writing the file
    channel.send_eof()?;
    channel.wait_eof()?;
    channel.close()?;
    channel.wait_close()?;

    Ok(())
}

// ---------------------------------------------------------------- Grid'5000 API

async fn submit_job(command: &DeployG5kCommand, password: &str) -> anyhow::Result<u64> {
    let client = reqwest::Client::new();
    let jobs_url = format!("{G5K_API_URL}/sites/{}/jobs", command.site);

    let resources = match &command.walltime {
        Some(walltime) => format!("nodes=1,walltime={walltime}"),
        None => "nodes=1".to_string(),
    };

    let script = command.script.clone().unwrap_or_else(|| format!("/home/{}/run_benchmark.sh", command.user));

    let payload = json!({
        "resources": resources,
        "command": format!("{script} \"{}\" \"{}\"", command.user, command.os),
        "stdout": format!("benchmark-nos-{}.txt", command.os),
        "queue": command.queue,
        "properties": format!("cluster='{}'", command.cluster),
        "name": format!("stdout-benchmark-nos-{}", command.os),
    });

    info!(target: TARGET, "Reserving a node on cluster \"{}\" (queue \"{}\") at {}", command.cluster, command.queue, command.site);

    let response = client
        .post(&jobs_url)
        .basic_auth(&command.user, Some(password))
        .json(&payload)
        .send()
        .await
        .context("Could not reach the Grid'5000 API")?;

    let status = response.status();

    if status.as_u16() != 201 {
        let body = response.text().await.unwrap_or_default();
        bail!("Job submission failed ({status}): {body}");
    }

    let job: Value = response.json().await?;
    let job_id = job["uid"].as_u64().ok_or_else(|| anyhow!("The Grid'5000 API response has no job uid: {job}"))?;

    info!(target: TARGET, "Job submitted with ID: {job_id}");

    Ok(job_id)
}

async fn wait_for_job(command: &DeployG5kCommand, password: &str, job_id: u64) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let job_url = format!("{G5K_API_URL}/sites/{}/jobs/{job_id}", command.site);

    let mut last_state = String::new();
    let mut failures = 0;

    loop {
        sleep(JOB_POLL_INTERVAL).await;

        let state = match fetch_job_state(&client, &job_url, &command.user, password).await {
            Ok(state) => {
                failures = 0;
                state
            },
            Err(e) => {
                failures += 1;
                warn!(target: TARGET, "Could not fetch the job state ({failures}/{MAX_CONSECUTIVE_POLL_FAILURES}): {e}");

                if failures >= MAX_CONSECUTIVE_POLL_FAILURES {
                    bail!("Giving up on following job {job_id}, it may still be running: {e}");
                }

                continue;
            }
        };

        if state != last_state {
            info!(target: TARGET, "Job {job_id} state: {state}");
            last_state = state.clone();
        }

        match state.as_str() {
            "terminated" => return Ok(()),
            "error" => bail!("Job {job_id} ended in error"),
            _ => {}
        }
    }
}

async fn fetch_job_state(client: &reqwest::Client, job_url: &str, user: &str, password: &str) -> anyhow::Result<String> {
    let response = client.get(job_url).basic_auth(user, Some(password)).send().await?;

    if !response.status().is_success() {
        bail!("HTTP {}", response.status());
    }

    let job: Value = response.json().await?;

    job["state"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| anyhow!("no \"state\" field in {job}"))
}
