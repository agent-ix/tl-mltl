# TL-217 current-source past grid

`report.json` records the 2026-09-25 native R2U2 run against clean
`tl-mltl` source `29cea002008f4a6855f54b73ecd63c234499fa3c` and clean
R2U2 source `336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a`. Its 147
SHA-256 entries cover the generated C2PO and CSV inputs, compiled monitor
inputs, and compiler and monitor output streams. The independent gate checked
each retained raw file and all 1,350 reported cells before this summary was
committed. The producer stdout and replay report hashes bind the summary to
the exact local run.

To reproduce, use clean checkouts at those commits and run the command in
`campaign/TL217.md` from the measured `tl-mltl` checkout, with Rust 1.98.1,
Python 3.13 and the pinned R2U2 compiler and built monitor. Choose new report
and raw directory paths. Compare the new report's `TL217_PAST_GRID` marker
with `report.json`: source, lock, target, compiler, monitor, counts, and every
`artifacts` digest must match. The fresh gate itself independently verifies
the raw bytes and per-step verdicts.

The result covers 360 admitted target agreements. The other 990 cells have
typed `unsupported_mapping` classifications, including trace-origin cases;
they are documented limitations and receive no agreement credit. The
51-member native ARM EA/Quoin diagnostic on this same measured source graph
accepted 50 members; its optional `V10.monitor.unsafe-since` member was
inconclusive. That selected run does not qualify the full 120-member Stage 1
Campaign.
