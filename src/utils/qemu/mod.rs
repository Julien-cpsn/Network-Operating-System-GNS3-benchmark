//! Direct QEMU backend.
//!
//! Replaces the former GNS3 server. Each node of a topology is a `qemu-system-x86_64` process
//! started with `-daemonize`, its disks are linked clones (qcow2 overlays) of the original images,
//! its serial console is exposed as a telnet server, and each physical link is a point-to-point
//! UDP tunnel between two QEMU `socket` netdevs. Links are cut through QMP (`set_link`).

pub mod image;
pub mod ports;
pub mod process;
pub mod qmp;
pub mod vm;

#[cfg(test)]
mod tests;
