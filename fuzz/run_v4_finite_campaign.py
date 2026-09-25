#!/usr/bin/env python3
"""Record a bounded finite-oracle differential libFuzzer campaign."""

import run_v4_campaign

run_v4_campaign.TARGETS["tl-mltl"] = "finite_oracle_differential"
run_v4_campaign.REPORT_SCHEMA = "tl-v4.finite-oracle-fuzz-campaign/v1"
run_v4_campaign.SCOPE = (
    "finite/past production-vs-independent-oracle differential; bounded run only"
)
run_v4_campaign.TEMP_PREFIX = "tl-v4-finite-oracle-fuzz-"

if __name__ == "__main__":
    run_v4_campaign.main()
