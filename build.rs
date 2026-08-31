use std::{env, process::Command};

fn main() {
    println!("cargo:rerun-if-env-changed=TL_MLTL_SOURCE_REVISION");
    println!("cargo:rerun-if-changed=.git/HEAD");
    let revision = env::var("TL_MLTL_SOURCE_REVISION").ok().or_else(|| {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        Some(String::from_utf8(output.stdout).ok()?.trim().to_owned())
    });
    let revision = revision
        .expect("TL_MLTL_SOURCE_REVISION must be set when building outside a git source checkout");
    println!("cargo:rustc-env=TL_MLTL_SOURCE_REVISION={revision}");
}
