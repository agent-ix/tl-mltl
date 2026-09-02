---
type: log
title: "PLAN-002 - Update log"
description: "Chronological changes to the tl-mltl shared-assurance migration plan bundle."
---
# PLAN-002 - Update log

## History

- **2026-09-02** - Bundle opened for issue #13 on top of `fix/qualified-collector`
  rather than `main`, because that branch carries seven commits of evidence
  machinery this migration removes and the old path has to exist to be run
  against the same candidate revision as the new one. This plan supersedes
  PR #12.
