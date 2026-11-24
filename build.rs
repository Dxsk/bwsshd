// Stamps the commit hash and date shown in the tray panel. Builds outside a
// git checkout (crates.io) leave both empty.
use std::process::Command;

fn git(args: &[&str]) -> String {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
    println!(
        "cargo:rustc-env=BWSSHD_COMMIT={}",
        git(&["rev-parse", "--short=10", "HEAD"])
    );
    println!(
        "cargo:rustc-env=BWSSHD_COMMIT_DATE={}",
        git(&["log", "-1", "--format=%cs"])
    );
}
