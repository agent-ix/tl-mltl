#!/usr/bin/env python3
"""Run the same checked V4 campaign contract for closed-trace evaluation."""

import run_v4_campaign

run_v4_campaign.TARGETS["tl-mltl"] = "closed_eval"

if __name__ == "__main__":
    run_v4_campaign.main()
