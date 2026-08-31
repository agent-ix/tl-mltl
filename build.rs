use std::{env, process::Command};

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8(output.stdout).ok()?.trim().to_owned())
}

fn main() {
    println!("cargo:rerun-if-env-changed=TL_MLTL_SOURCE_REVISION");
    println!("cargo:rerun-if-env-changed=TL_MLTL_SOURCE_STATE");
    println!("cargo:rerun-if-changed=.");
    if let Some(git_dir) = git(&["rev-parse", "--absolute-git-dir"]) {
        println!("cargo:rerun-if-changed={git_dir}/HEAD");
        println!("cargo:rerun-if-changed={git_dir}/packed-refs");
        if let Some(reference) = git(&["symbolic-ref", "--quiet", "HEAD"]) {
            println!("cargo:rerun-if-changed={git_dir}/{reference}");
        }
    }
    let revision = env::var("TL_MLTL_SOURCE_REVISION")
        .ok()
        .or_else(|| git(&["rev-parse", "HEAD"]))
        .expect("TL_MLTL_SOURCE_REVISION must be set when building outside a git source checkout");
    let source_state = env::var("TL_MLTL_SOURCE_STATE")
        .ok()
        .or_else(|| {
            git(&["status", "--porcelain", "--untracked-files=no"]).map(|status| {
                if status.is_empty() {
                    "clean"
                } else {
                    "modified"
                }
                .to_owned()
            })
        })
        .expect("TL_MLTL_SOURCE_STATE must be set when building outside a git source checkout");
    println!("cargo:rustc-env=TL_MLTL_SOURCE_REVISION={revision}");
    println!("cargo:rustc-env=TL_MLTL_SOURCE_STATE={source_state}");
}
