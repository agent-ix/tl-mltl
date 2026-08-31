#!/usr/bin/env python3
"""Behavior tests for the qualified executable lock."""

from __future__ import annotations

import copy

import tool_identity


def expect(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def main() -> int:
    value, tools = tool_identity.load_lock()
    expect(set(tools) == set(tool_identity.REQUIRED), "mandatory tool census drifted")
    unavailable, mismatches = tool_identity.verify_live(value, tools)
    expect(not unavailable and not mismatches, "live executable identities disagree with tools.lock")
    changed = copy.deepcopy(value)
    changed["tools"]["cargo"]["sha256"] = "0" * 64
    changed_tools = tool_identity.validate_lock(changed)
    _, changed_mismatches = tool_identity.verify_live(changed, changed_tools)
    expect(any("cargo" in item for item in changed_mismatches), "digest mutation was not detected")
    print("qualified tool identity behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
