use std::{env, path::Path, process::Command};

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8(output.stdout).ok()?.trim().to_owned())
}

fn watch_existing_git_path(path: &str) {
    if let Some(path) = git(&["rev-parse", "--path-format=absolute", "--git-path", path]) {
        if Path::new(&path).exists() {
            println!("cargo:rerun-if-changed={path}");
        }
    }
}

fn main() {
    println!("cargo:rerun-if-env-changed=TL_MLTL_SOURCE_REVISION");
    println!("cargo:rerun-if-env-changed=TL_MLTL_SOURCE_STATE");
    if let Some(files) = git(&["ls-files"]) {
        for file in files.lines() {
            println!("cargo:rerun-if-changed={file}");
        }
    }
    watch_existing_git_path("HEAD");
    watch_existing_git_path("packed-refs");
    if let Some(reference) = git(&["symbolic-ref", "--quiet", "HEAD"]) {
        watch_existing_git_path(&reference);
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
