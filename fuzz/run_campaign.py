#!/usr/bin/env python3
"""STOPGAP: first-party gating logic belongs in Rust (TL-370 covers moving this
driver there); no further feature work goes into this Python script.

Run the cargo-fuzz campaign (FR-021): every target under several seeds with
a fixed budget and a trailing-window plateau stop, writing a campaign.json.

Usage: fuzz/run_campaign.py OUT_DIR [--budget 900] [--window 300] [--seeds 101,102,103]

Each run starts from the checked-in seed corpus (read-only) into its own scratch
corpus, so the seeded runs stay independent. Runs of different targets execute in
parallel; runs of one target execute in sequence. Build time is excluded from the
observed duration. A plateau means no new coverage feature over the trailing
window; a run that reaches the budget while still growing is `growth_observed`.
"""
import json, os, re, signal, subprocess, sys, threading, time, argparse, shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parent
LINE = re.compile(r"^#(\d+)\s+(\w+)\s+cov: (\d+) ft: (\d+) corp: (\d+)/")


def run_one(target, seed, args, out):
    binary = ROOT / "target" / args.triple / "release" / target
    scratch = out / "scratch" / f"{target}-{seed}"
    shutil.rmtree(scratch, ignore_errors=True)
    scratch.mkdir(parents=True)
    log = out / "logs" / f"{target}-{seed}.log"
    log.parent.mkdir(parents=True, exist_ok=True)
    seeds = ROOT / "corpus" / target
    start_files = len(list(seeds.iterdir()))
    cmd = [str(binary), str(scratch), str(seeds), f"-seed={seed}",
           f"-max_total_time={args.budget}", "-rss_limit_mb=8192",
           f"-artifact_prefix={out}/crashes/{target}-{seed}-", "-print_final_stats=1"]
    (out / "crashes").mkdir(exist_ok=True)
    t0 = time.monotonic()
    p = subprocess.Popen(cmd, stderr=subprocess.PIPE, stdout=subprocess.DEVNULL, text=True, cwd=ROOT)
    cov = ft = corp = 0
    last_growth = t0
    execs = 0
    stop = "budget"
    with open(log, "w") as lf:
        for line in p.stderr:
            lf.write(line)
            m = LINE.match(line)
            if m:
                c, f, k = int(m[3]), int(m[4]), int(m[5])
                now = time.monotonic()
                if f > ft:
                    last_growth = now
                cov, ft, corp = c, f, k
                if now - last_growth >= args.window and stop == "budget" and now - t0 > args.window:
                    stop = "plateau"
                    p.send_signal(signal.SIGINT)
            m = re.match(r"stat::number_of_executed_units:\s+(\d+)", line)
            if m:
                execs = int(m[1])
    rc = p.wait()
    secs = time.monotonic() - t0
    text = log.read_text()
    outcome = None
    if "SUMMARY: libFuzzer: timeout" in text:
        outcome = "timeout"
    elif "out-of-memory" in text:
        outcome = "out_of_memory"
    elif "SUMMARY" in text:
        outcome = "crash"
    elif rc not in (0, 130) and stop != "plateau":
        outcome = "tool_error"
    if outcome:
        stop = outcome
    elif stop == "budget":
        stop = "growth_observed" if time.monotonic() - last_growth < args.window else "budget_no_recent_growth"
    crashes = sorted(p.name for p in (out / "crashes").glob(f"{target}-{seed}-*"))
    return {
        "target": target, "seed": seed, "requested_seconds": args.budget,
        "observed_seconds": round(secs), "plateau_window_seconds": args.window,
        "starting_corpus_files": start_files, "executions": execs,
        "final_cov_edges": cov, "final_features": ft, "final_corpus_entries": corp,
        "last_new_feature_seconds": round(last_growth - t0),
        "stop_reason": stop, "exit_status": rc, "crash": crashes or None,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out", type=Path)
    ap.add_argument("--budget", type=int, default=900)
    ap.add_argument("--window", type=int, default=300)
    ap.add_argument("--seeds", default="101,102,103")
    ap.add_argument("--triple", default="aarch64-apple-darwin")
    ap.add_argument("--targets", default="")
    args = ap.parse_args()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    targets = args.targets.split(",") if args.targets else sorted(p.stem for p in (ROOT / "fuzz_targets").glob("*.rs"))
    seeds = [int(s) for s in args.seeds.split(",")]
    results = {}
    def worker(t):
        results[t] = [run_one(t, s, args, out) for s in seeds]
    threads = [threading.Thread(target=worker, args=(t,)) for t in targets]
    [t.start() for t in threads]
    [t.join() for t in threads]
    runs = [r for t in targets for r in results[t]]
    record = {"budget_seconds": args.budget, "plateau_window_seconds": args.window,
              "seeds": seeds, "runs": runs}
    (out / "runs.json").write_text(json.dumps(record, indent=1))
    print(json.dumps(record, indent=1))

if __name__ == "__main__":
    main()
