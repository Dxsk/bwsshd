//! bwsshd keeps an ssh_config in sync with the SSH keys held by the Bitwarden
//! desktop SSH agent. It pins exactly one key per host via `IdentitiesOnly` so
//! SSH offers the right key first and does not burn through `MaxAuthTries`
//! trying every key in the agent.
//!
//! The transport differs per OS (a unix socket on Linux/macOS, a named pipe on
//! Windows); that difference is isolated in [`platform`].

#[macro_use]
pub mod log;

pub mod agent;
pub mod cli;
pub mod config;
pub mod platform;
pub mod sync;
pub mod target;
pub mod tray;
