//! Campaign driver: `inventory`, `run` and `replay` over `campaign/members.json`.
//!
//! `campaign inventory` checks the member list against the Makefile and fuzz
//! targets. `campaign run [--out DIR] [MEMBER...]` executes members and writes
//! `DIR/report.json`. `campaign replay REPORT [--out DIR]` reruns a retained
//! report against the exact source it names and writes a new attempt.

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use tl_mltl::campaign::{
    check_inventory, discover_fuzz_targets, parse_members, replay, run_campaign, Report,
};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let repo = PathBuf::from(".");
    let result = match args.first().map(String::as_str) {
        Some("inventory") => inventory(&repo),
        Some("run") => run(&repo, &args[1..]),
        Some("replay") => replay_cmd(&repo, &args[1..]),
        _ => Err(
            "usage: campaign <inventory | run [--out DIR] [MEMBER...] | replay REPORT [--out DIR]>"
                .to_owned(),
        ),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("campaign: {e}");
            ExitCode::FAILURE
        }
    }
}

fn split_out(args: &[String], default: &str) -> (PathBuf, Vec<String>) {
    let mut out = PathBuf::from(default);
    let mut rest = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == "--out" {
            if let Some(v) = it.next() {
                out = PathBuf::from(v);
            }
        } else {
            rest.push(a.clone());
        }
    }
    (out, rest)
}

fn inventory(repo: &Path) -> Result<(), String> {
    let text = fs::read_to_string(repo.join("campaign/members.json")).map_err(|e| e.to_string())?;
    let file = parse_members(&text)?;
    let makefile = fs::read_to_string(repo.join("Makefile")).map_err(|e| e.to_string())?;
    let findings = check_inventory(&file, &makefile, &discover_fuzz_targets(repo));
    if findings.is_empty() {
        println!(
            "campaign inventory: {} members, complete",
            file.members.len()
        );
        Ok(())
    } else {
        Err(findings.join("\n  "))
    }
}

fn write_report(report: &Report, out: &Path, name: &str) -> Result<(), String> {
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(report).map_err(|e| e.to_string())?;
    fs::write(out.join(name), bytes).map_err(|e| e.to_string())?;
    println!(
        "{} aggregate={} sha256={}",
        out.join(name).display(),
        report.aggregate,
        report.report_sha256
    );
    for m in &report.members {
        println!(
            "  {:<28} {:?}{}",
            m.id,
            m.status,
            m.detail
                .as_deref()
                .map(|d| format!(" ({d})"))
                .unwrap_or_default()
        );
    }
    Ok(())
}

fn run(repo: &Path, args: &[String]) -> Result<(), String> {
    let (out, only) = split_out(args, "target/campaign");
    let report = run_campaign(repo, &out, &only)?;
    write_report(&report, &out, "report.json")?;
    if report.aggregate == "reject" {
        return Err("aggregate reject".to_owned());
    }
    Ok(())
}

fn replay_cmd(repo: &Path, args: &[String]) -> Result<(), String> {
    let (out, rest) = split_out(args, "target/campaign-replay");
    let path = rest.first().ok_or("replay needs a report path")?;
    let report: Report = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let (attempt, diffs) = replay(&report, repo, &out)?;
    write_report(&attempt, &out, "replay-report.json")?;
    if diffs.is_empty() {
        println!("replay reproduces the retained report");
        Ok(())
    } else {
        Err(format!("replay differs:\n  {}", diffs.join("\n  ")))
    }
}
