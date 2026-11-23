//! FR-113: tl-mltl stays independent of tl-solve and of every SAT solver crate.
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).
//! TC-257 reads the resolved lock files instead of running `cargo tree`: a
//! `Cargo.lock` lists every package reachable under any feature set and dev
//! dependencies, so absence there implies absence from each `cargo tree` view.

use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN_PACKAGES: [&str; 5] = ["tl-solve", "rustsat", "rustsat-batsat", "batsat", "varisat"];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn locked_package_names(lock: &str) -> Vec<String> {
    lock.lines()
        .filter_map(|line| line.strip_prefix("name = \""))
        .filter_map(|rest| rest.strip_suffix('"'))
        .map(str::to_owned)
        .collect()
}

fn forbidden_in(lock: &str) -> Vec<String> {
    locked_package_names(lock)
        .into_iter()
        .filter(|name| FORBIDDEN_PACKAGES.contains(&name.as_str()))
        .collect()
}

// Trace: TC-257, FR-113-AC-1
#[test]
fn resolved_dependency_graph_has_no_solver_package() {
    for lock in ["Cargo.lock", "fuzz/Cargo.lock"] {
        let text = fs::read_to_string(root().join(lock)).expect("lock file is present");
        assert_eq!(forbidden_in(&text), Vec::<String>::new(), "{lock}");
    }
    // The check fails when one is added.
    let planted = "[[package]]\nname = \"varisat\"\nversion = \"0.2.2\"\n";
    assert_eq!(forbidden_in(planted), vec!["varisat".to_owned()]);
}

fn rust_files(directory: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "target") {
                continue;
            }
            rust_files(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

// Trace: TC-258, FR-113-AC-2
#[test]
fn no_source_imports_tl_solve_and_deny_names_no_tl_solve_repository() {
    let mut files = Vec::new();
    for directory in ["src", "tests", "fuzz", "tools"] {
        rust_files(&root().join(directory), &mut files);
    }
    assert!(!files.is_empty());
    // Assemble the needle so this file does not match itself.
    let needle = format!("{}_{}", "tl", "solve");
    for file in &files {
        let text = fs::read_to_string(file).unwrap();
        assert!(
            !text.lines().any(|line| {
                let line = line.trim_start();
                (line.starts_with("use ") || line.starts_with("extern crate"))
                    && line.contains(&needle)
                    || line.contains(&format!("{needle}::"))
            }),
            "{} imports {needle}",
            file.display()
        );
    }
    let deny = fs::read_to_string(root().join("deny.toml")).unwrap();
    let allow_git = deny
        .split("allow-git")
        .nth(1)
        .and_then(|rest| rest.split(']').next())
        .unwrap_or("");
    assert!(!allow_git.contains("tl-solve"));
}
