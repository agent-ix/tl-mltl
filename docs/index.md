# tl-mltl documentation

tl-mltl evaluates Mission-time Linear Temporal Logic (MLTL) formulas over finite traces, analyzes their horizon and maps them to the R2U2 monitor. This directory documents what the crate contains and how to use it. The [README](../README.md) has the overview and the smallest example.

| Page | What it covers |
| --- | --- |
| [how-it-works.md](how-it-works.md) | The evaluation pipeline, the input and result types, and where the crate sits between text and a Quire outcome. |
| [semantics.md](semantics.md) | The meaning of the future, past and infinite-trace evaluators, with their verdict values and refusals. |
| [horizon.md](horizon.md) | Lookahead, propagation delay, buffer length and required history, and how each is computed. |
| [monitors-interop.md](monitors-interop.md) | The C2PO mapping, mapping manifests, external verdicts, differential comparison and the retained R2U2 exchange. |
| [cli.md](cli.md) | The `tl-mltl` command, its request and trace documents, exit status and the conformance producers. |
| [scenario.md](scenario.md) | A worked debugging example across the TL crates on a retained R2U2 case, with a triage guide. |
| [examples.md](examples.md) | Runnable examples for evaluation, horizon, mapping, comparison, past-time evaluation and the CLI. |
| [api-reference.md](api-reference.md) | Public types and functions by module, with their error types. |
