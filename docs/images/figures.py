# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (C) 2026 Agent-IX
"""tl-mltl documentation figures, rendered by the docs-figures skill.

Regenerate from the repository root with the agent-ix/dev-tools skill:

    python3 -m pip install fonttools
    python3 <dev-tools>/skills/docs-figures/scripts/render.py docs/images/figures.py

Add --check to confirm the committed SVGs are current. Values are read from this
repository: the retained R2U2 4.2 corpus (formula, trace, C2PO input and the
differential report) and the CLI request fixtures.

The block between the two SHARED markers is identical in every TL repository
(tl-syntax, tl-parse, tl-rewrite, tl-mltl, tl-oracle, quire-mltl). Copy it
verbatim and call tl_stack(here="<crate>") and stack_pipeline(here="<crate>").
"""

import json
from pathlib import Path

from docs_figures import FONTS, SMALL, TITLE, Figure  # noqa: F401

STYLE = "ix-docs"

ROOT = Path(__file__).resolve().parents[2]

# One role per kind of thing, fixed across the TL and Quire repositories.
ROLES = [
    ("code", "authored source", "box"),
    ("logic", "processing step", "box"),
    ("model", "checked document / view", "box"),
    ("evidence", "outcome", "box"),
    ("ink", "input", "box"),
]


# ==== SHARED: copy verbatim into sibling figures.py (begin) ===================

TL_CRATES = [
    # (column, row, name, one-line role, dependency note)
    (0, 1, "tl-parse", "text to formula graph"),
    (1, 1, "tl-oracle", "independent reference"),
    (2, 1, "tl-mltl", "evaluates traces"),
    (3, 1, "quire-mltl", "the bridge to Quire"),
    (2, 2, "tl-rewrite", "equivalent rewrites"),
    (3, 2, "quire-observation", "Quire observations"),
]


def tl_stack(here="tl-syntax"):
    """The TL crate family as a dependency graph; `here` is drawn highlighted."""
    f = Figure(
        "tl-stack",
        980,
        356,
        "The TL crates. tl-syntax defines the formula graph, profiles and signal catalog, and "
        "tl-parse, tl-oracle, tl-mltl, tl-rewrite and quire-mltl all depend on it. tl-rewrite "
        "depends on tl-mltl. quire-mltl depends on tl-mltl and quire-observation, and is the "
        "single bridge to Quire. Dashed lines are development-only: tl-mltl and tl-rewrite "
        "test against tl-oracle, and tl-rewrite tests against tl-parse.",
    )
    w, h, step = 192, 56, 262
    rows = {0: 24, 1: 140, 2: 256}
    f.text(3 * step, 244, "QUIRE", "muted", "mono", SMALL)
    f.box(0, rows[0], 980, h, "logic" if here == "tl-syntax" else "ink", "tl-syntax",
          "formula graph · semantic profiles · signal catalog · strict documents")
    where = {}
    for col, row, name, sub in TL_CRATES:
        x, y = col * step, rows[row]
        where[name] = (x, y)
        role = "logic" if name == here else "ink"
        f.box(x, y, w, h, role, name, sub, dashed=(name == "quire-observation"))
    for name in ("tl-parse", "tl-oracle", "tl-mltl", "quire-mltl"):
        x, y = where[name]
        f.line([(x + w / 2, rows[0] + h), (x + w / 2, y - 3)], "model")
    f.note(where["tl-parse"][0] + w / 2 + 10, 104, "types", "model")
    mx, my = where["tl-mltl"]
    rx, ry = where["tl-rewrite"]
    f.line([(mx + w / 2, my + h), (rx + w / 2, ry - 3)], "model")
    f.note(mx + w / 2 + 10, my + h + 30, "documents", "model")
    qx, qy = where["quire-mltl"]
    f.line([(mx + w, my + h / 2), (qx - 3, qy + h / 2)], "model")
    f.note(mx + w + 8, my + h / 2 - 8, "results", "model")
    ox, oy = where["quire-observation"]
    f.line([(ox + w / 2, oy - 3), (ox + w / 2, qy + h)], "model")
    f.note(ox + w / 2 + 10, qy + h + 30, "observations", "model")
    # development-only edges: tl-oracle and tl-parse feed tests of tl-mltl and tl-rewrite
    px, py = where["tl-parse"]
    zx, zy = where["tl-oracle"]
    f.line([(zx + w, zy + h / 2), (mx - 3, my + h / 2)], "model", dashed=True)
    f.line([(zx + w / 2, zy + h), (zx + w / 2, ry + 18), (rx - 3, ry + 18)], "model", dashed=True)
    f.line([(px + w / 2, py + h), (px + w / 2, ry + 40), (rx - 3, ry + 40)], "model", dashed=True)
    f.legend(0, 350, [("logic", "this repository", "box"), ("ink", "other crate", "box"),
                      ("model", "depends on", "solid"), ("model", "development only", "dash")])
    return f


PIPELINE_STAGES = [
    ("MLTL text", "your formula", "code", None),
    ("tl-parse", "parse", "logic", "tl-parse"),
    ("tl-rewrite", "rewrite", "logic", "tl-rewrite"),
    ("tl-mltl", "evaluate", "logic", "tl-mltl"),
    ("tl-oracle", "tests only", "logic", "tl-oracle"),
    ("quire-mltl", "map to Quire", "logic", "quire-mltl"),
]


def stack_pipeline(here="tl-syntax"):
    """End-to-end flow: text, parse, rewrite, evaluate with an oracle cross-check, Quire view."""
    f = Figure(
        "pipeline",
        980,
        392,
        "A formula moves from text to a Quire outcome. tl-parse turns MLTL text into a formula "
        "document, tl-rewrite returns a rewritten document, and tl-mltl evaluates it against a "
        "trace. In tests, tl-oracle evaluates the same formula and trace independently and the "
        "verdicts are compared. quire-mltl maps the result to a Boolean Value or a typed "
        "NonValue reason. All documents use the vocabulary of tl-syntax.",
    )
    w, h = 140, 56
    step = (980 - w) / (len(PIPELINE_STAGES) - 1)
    cx = [round(i * step + w / 2) for i in range(len(PIPELINE_STAGES))]
    for i, (name, sub, role, crate) in enumerate(PIPELINE_STAGES):
        if crate is not None:
            role = "logic" if crate == here else "ink"
        f.box(cx[i] - w // 2, 0, w, h, role, name, sub)
        f.line([(cx[i], h + 2), (cx[i], 312)], "faint", dashed=True, arrow=False, weight=1.0)
    msgs = [
        (0, 1, 96, "MLTL text", "code", False),
        (1, 2, 140, "formula document", "model", False),
        (2, 3, 184, "rewritten formula", "model", False),
        (3, 4, 228, "formula + trace", "model", True),
        (4, 3, 262, "reference verdict", "evidence", True),
        (3, 5, 306, "result", "model", False),
    ]
    for a, b, y, label, role, dev in msgs:
        sign = 1 if b > a else -1
        f.line([(cx[a] + 3 * sign, y), (cx[b] - 3 * sign, y)], role, dashed=dev,
               weight=2.4 if not dev else 1.5)
        left = min(cx[a], cx[b]) + 10
        f.note(left, y - 8, label, role)
    f.box(980 - 190, 332, 190, 50, "evidence", "Value or NonValue", "true / false or reason")
    f.line([(cx[5], 312), (cx[5], 329)], "evidence")
    f.box(0, 332, 520, 50, "logic" if here == "tl-syntax" else "ink", "tl-syntax",
          "the formula, profile, signal and document types every step uses")
    return f


# ==== SHARED (end) ============================================================


# ---- repository data ---------------------------------------------------------

CASE = "r2u2-until-lower-bound-v1"


def load(path):
    return json.loads((ROOT / path).read_text(encoding="utf-8"))


def node_text(index, node):
    kind = node["kind"]
    if kind == "proposition":
        return f"n{index}  proposition {node['proposition']}"
    iv = node["interval"]
    operands = ", ".join(f"n{node[k]}" for k in ("left", "right", "operand") if k in node)
    return f"n{index}  {kind} [{iv['start']},{iv['end']}] of {operands}"


def lookahead(nodes):
    """Static lookahead of the root: interval end plus the deepest operand, as in docs/horizon.md."""
    values = []
    for node in nodes:
        kind = node["kind"]
        children = [values[node[k]] for k in ("left", "right", "operand") if k in node]
        if kind == "proposition":
            values.append(0)
        elif "interval" in node:
            values.append(node["interval"]["end"] + max(children))
        else:
            values.append(max(children, default=0))
    return values[-1]


def instants_text(trace):
    return " ".join("{" + ",".join(str(p) for p in instant) + "}" for instant in trace)


# ---- figures -----------------------------------------------------------------


def hero():
    """README hero: one formula and trace give a verdict and a horizon, and map out to C2PO."""
    corpus = load("corpus/r2u2-v4.2/manifest.json")
    report = load("corpus/r2u2-v4.2/differential-report.json")
    case = next(c for c in corpus["cases"] if c["id"] == CASE)
    outcome = next(c for c in report["cases"] if c["id"] == CASE)
    nodes = case["formula"]["nodes"]
    profile = case["formula"]["semantic_profile"]
    trace = corpus["trace"]
    c2po = [ln.strip().rstrip(";") for ln in (ROOT / "corpus/r2u2-v4.2/formulas.c2po").read_text().splitlines()
            if ln.startswith("    ") and ln.strip().endswith(";") and "bool" not in ln]
    spec = c2po[case["formulaIndex"]]
    look = lookahead(nodes)
    verdict = "true" if outcome["verdict"] else "false"
    tool = f"R2U2 {corpus['source']['tag'].split('-')[0]}"
    f = Figure(
        "hero",
        980,
        356,
        f"A formula and a trace go into tl-mltl. evaluate_closed or evaluate_prefix returns a verdict, "
        f"here {verdict} at time {outcome['verdictTime']}, and pending when the prefix does not decide it. "
        f"analyze_horizon returns a lookahead of {look} and a buffer of {look + 1} instants. "
        f"map_to_c2po returns a manifest with the C2PO expression and SHA-256 digests of its input and output. "
        f"C2PO and R2U2 run outside this crate, and compare_external classifies their verdict against the "
        f"reference verdict, here {report['agreements']} of {report['supported']} supported cases agree.",
    )
    f.box(0, 24, 240, 62, "ink", "trace", instants_text(trace), "propositions per instant")
    f.box(0, 122, 240, 118, "code", "formula", *[node_text(i, n) for i, n in enumerate(nodes)], profile)
    f.line([(240, 55), (302, 55)], "ink")
    f.note(246, 49, "trace", "ink")
    f.line([(240, 181), (272, 181)], "code", arrow=False)
    f.line([(272, 70), (272, 267)], "code", arrow=False)
    f.line([(272, 70), (302, 70)], "code")
    f.line([(272, 158), (302, 158)], "code")
    f.line([(272, 267), (302, 267)], "code")
    f.box(305, 24, 180, 62, "logic", "evaluate_prefix", "or evaluate_closed")
    f.box(305, 130, 180, 56, "logic", "analyze_horizon", "checked sums")
    f.box(305, 236, 180, 62, "logic", "map_to_c2po", "online-prefix only")
    f.box(555, 24, 180, 62, "evidence", "verdict", f"{verdict} at time {outcome['verdictTime']}", "or pending")
    f.box(555, 130, 180, 56, "evidence", "HorizonReport", f"lookahead {look} · buffer {look + 1}")
    f.box(555, 236, 180, 62, "model", "MappingManifest", f"C2PO  {spec}", "SHA-256 digests")
    f.line([(485, 55), (552, 55)], "logic", weight=2.4)
    f.line([(485, 158), (552, 158)], "logic", weight=2.4)
    f.line([(485, 267), (552, 267)], "logic", weight=2.4)
    f.box(805, 24, 165, 62, "logic", "compare_external", "value · time")
    f.box(805, 130, 165, 56, "ink", tool, "out of band")
    f.box(805, 236, 165, 62, "evidence", "agreement", f"{report['agreements']} of {report['supported']} supported")
    f.line([(735, 55), (802, 55)], "evidence")
    f.note(741, 49, "verdict", "evidence")
    f.line([(735, 267), (770, 267), (770, 158), (802, 158)], "model")
    f.line([(887, 127), (887, 89)], "ink")
    f.line([(970, 55), (976, 55), (976, 267), (973, 267)], "evidence")
    f.legend(0, 340, [ROLES[0], ROLES[1], ROLES[2], ROLES[3], ROLES[4]])
    return f


def tl_stack_here():
    return tl_stack(here="tl-mltl")


def pipeline_here():
    return stack_pipeline(here="tl-mltl")


FIGURES = [hero, tl_stack_here, pipeline_here]
