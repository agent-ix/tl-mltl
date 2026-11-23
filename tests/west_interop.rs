//! TL-433: WEST import and export (FR-092), the retained WEST corpus
//! (FR-089-AC-2), the WEST importer's closure refusal (FR-094) and the hostile
//! cases of all three foreign readers (NFR-047).
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).
//! TC-331 (three reader fuzz targets) is not written: adding fuzz targets
//! needs campaign members and a build; see the PLAN deferred table.

use std::path::{Path, PathBuf};

use tl_mltl::interop::c2po::{import_c2po, C2poImportError};
use tl_mltl::interop::fret::{import_fret_mltl, FretImportError, FretProfile};
use tl_mltl::interop::west::{
    read_west, read_west_file, write_west, write_west_bounded, WestError,
};
use tl_mltl::interop::{Attribution, LossKind, ReaderError, ReaderLimits, REVISION_UNKNOWN};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus/west-v1")
}

fn corpus_lines() -> Vec<String> {
    std::fs::read_to_string(corpus_dir().join("formulas_d1.txt"))
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect()
}

#[derive(Clone)]
enum W {
    P(u32),
    T,
    Fl,
    Not(Box<W>),
    And(Box<W>, Box<W>),
    Or(Box<W>, Box<W>),
    G(u32, u32, Box<W>),
    F(u32, u32, Box<W>),
    U(u32, u32, Box<W>, Box<W>),
}

fn lower(w: &W, nodes: &mut Vec<Node>) -> NodeId {
    let kind = match w {
        W::P(n) => NodeKind::Proposition {
            proposition: PropositionId(*n),
        },
        W::T => NodeKind::True,
        W::Fl => NodeKind::False,
        W::Not(x) => NodeKind::Not {
            operand: lower(x, nodes),
        },
        W::And(x, y) => {
            let left = lower(x, nodes);
            let right = lower(y, nodes);
            NodeKind::And { left, right }
        }
        W::Or(x, y) => {
            let left = lower(x, nodes);
            let right = lower(y, nodes);
            NodeKind::Or { left, right }
        }
        W::G(a, b, x) => NodeKind::Globally {
            interval: Interval::new(*a, *b).unwrap(),
            operand: lower(x, nodes),
        },
        W::F(a, b, x) => NodeKind::Future {
            interval: Interval::new(*a, *b).unwrap(),
            operand: lower(x, nodes),
        },
        W::U(a, b, x, y) => {
            let left = lower(x, nodes);
            let right = lower(y, nodes);
            NodeKind::Until {
                interval: Interval::new(*a, *b).unwrap(),
                left,
                right,
            }
        }
    };
    nodes.push(Node::new(kind));
    NodeId((nodes.len() - 1) as u32)
}

fn b(w: W) -> Box<W> {
    Box::new(w)
}

/// The ten tl-rewrite `selectedCases`: (1-based line, hand-authored graph).
fn selected() -> Vec<(usize, W)> {
    vec![
        (6, W::And(b(W::Not(b(W::P(1)))), b(W::Not(b(W::P(1)))))),
        (9, W::Or(b(W::Not(b(W::P(1)))), b(W::T))),
        (10, W::And(b(W::Not(b(W::P(0)))), b(W::T))),
        (11, W::G(0, 2, b(W::T))),
        (12, W::F(0, 2, b(W::T))),
        (14, W::U(0, 2, b(W::T), b(W::Not(b(W::P(1)))))),
        (15, W::Or(b(W::Fl), b(W::P(0)))),
        (16, W::And(b(W::T), b(W::P(1)))),
        (19, W::Or(b(W::Fl), b(W::Fl))),
        (20, W::And(b(W::Fl), b(W::Fl))),
    ]
}

// Trace: TC-322, FR-092-AC-1
#[test]
fn selected_cases_round_trip_to_the_same_line_and_hand_authored_graph() {
    let lines = corpus_lines();
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(corpus_dir().join("manifest.json")).unwrap())
            .unwrap();
    let manifest_lines: Vec<usize> = manifest["selectedCases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| case["line"].as_u64().unwrap() as usize)
        .collect();
    let cases = selected();
    assert_eq!(cases.iter().map(|(line, _)| *line).collect::<Vec<_>>(), manifest_lines);
    for (line, tree) in cases {
        let source = &lines[line - 1];
        let read = read_west(source.as_bytes(), ReaderLimits::default()).unwrap();
        let mut nodes = Vec::new();
        let root = lower(&tree, &mut nodes);
        assert_eq!((read.nodes.clone(), read.root), (nodes, root), "line {line}");
        assert_eq!(&write_west(read.formula()).unwrap(), source, "line {line}");
    }
}

// Trace: TC-323, FR-092-AC-2
#[test]
fn all_twenty_lines_round_trip_byte_for_byte() {
    let bytes = std::fs::read(corpus_dir().join("formulas_d1.txt")).unwrap();
    let formulas = read_west_file(&bytes, ReaderLimits::default()).unwrap();
    assert_eq!(formulas.len(), 20);
    let mut written = String::new();
    for formula in &formulas {
        written.push_str(&write_west(formula.formula()).unwrap());
        written.push('\n');
    }
    assert_eq!(written.as_bytes(), &bytes[..]);
}

// Trace: TC-324, FR-092-AC-3
#[test]
fn unexportable_classes_are_refused_with_a_loss_record_and_no_text() {
    let atom = |n| {
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(n),
        })
    };
    let cases: Vec<(SemanticProfile, Vec<Node>, LossKind, &str)> = vec![
        (
            SemanticProfile::ClosedTraceV1,
            vec![
                atom(0),
                atom(1),
                Node::new(NodeKind::Implies {
                    left: NodeId(0),
                    right: NodeId(1),
                }),
            ],
            LossKind::UnsupportedOperator,
            "implies",
        ),
        (
            SemanticProfile::ClosedTraceV1,
            vec![
                atom(0),
                atom(1),
                Node::new(NodeKind::Equivalent {
                    left: NodeId(0),
                    right: NodeId(1),
                }),
            ],
            LossKind::UnsupportedOperator,
            "equivalent",
        ),
        (
            SemanticProfile::OriginCompleteHistoryV1,
            vec![
                atom(0),
                atom(1),
                Node::new(NodeKind::Once {
                    interval: Interval::new(0, 1).unwrap(),
                    operand: NodeId(1),
                }),
            ],
            LossKind::UnsupportedOperator,
            "past-operator",
        ),
        (
            SemanticProfile::LtlfFiniteTraceV1,
            vec![atom(0), atom(1), Node::new(NodeKind::LtlfGlobally { operand: NodeId(1) })],
            LossKind::UnsupportedInterval,
            "non-closed-interval",
        ),
    ];
    // SPEC-GAP: the fifth class (`W`/`M`) has no graph node and cannot reach
    // the writer; see `write_west`.
    for (profile, nodes, kind, construct) in cases {
        let formula = Formula::new(profile, NodeId(2), &nodes).unwrap();
        let record = write_west(formula).unwrap_err();
        assert_eq!(record.kind, kind);
        assert_eq!(record.construct, construct);
        assert!(record.no_span_reason.is_some());
    }
}

/// tl-rewrite's `corpus/west-v1`: `TL_REWRITE_WEST_DIR` first, then the
/// sibling layouts this repository is checked out in (a `worktrees/` directory
/// holding `TL-code-tl-rewrite`, or a plain `tl-rewrite` checkout next to the
/// repository or next to its `worktrees/` directory).
fn tl_rewrite_copy() -> Option<PathBuf> {
    if let Some(configured) = std::env::var_os("TL_REWRITE_WEST_DIR") {
        return Some(PathBuf::from(configured));
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    [
        "../TL-code-tl-rewrite/corpus/west-v1",
        "../tl-rewrite/corpus/west-v1",
        "../../tl-rewrite/corpus/west-v1",
    ]
    .iter()
    .map(|relative| root.join(relative))
    .find(|path| path.is_dir())
}

// Trace: TC-325, TC-317, FR-092-AC-4, FR-089-AC-2, FR-089-AC-3
#[test]
fn retained_corpus_is_complete_attributed_and_byte_equal_to_tl_rewrite() {
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(corpus_dir().join("manifest.json")).unwrap())
            .unwrap();
    let mut listed: Vec<String> = manifest["retainedFiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| name.as_str().unwrap().to_owned())
        .collect();
    listed.sort();
    let mut census: Vec<String> = std::fs::read_dir(corpus_dir())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    census.sort();
    assert_eq!(listed, census);
    assert!(corpus_dir().join(manifest["licenseFile"].as_str().unwrap()).is_file());

    let attribution = Attribution {
        repository: manifest["upstream"].as_str().map(str::to_owned),
        path: manifest["upstreamPath"].as_str().map(str::to_owned),
        revision: manifest["revision"].as_str().map(str::to_owned),
        licence: manifest["license"].as_str().map(str::to_owned),
    };
    assert_eq!(attribution.revision.as_deref(), Some(REVISION_UNKNOWN));
    assert_eq!(attribution.validate_west(), Ok(()));

    // Byte equality with tl-rewrite's copy while that copy exists. Absence
    // fails loudly; `TL_REWRITE_WEST_SKIP=1` is the explicit, visible opt-out
    // for a checkout without tl-rewrite (never a silent pass).
    match tl_rewrite_copy() {
        Some(owner) => {
            for file in ["formulas_d1.txt", "LICENSE"] {
                assert_eq!(
                    std::fs::read(corpus_dir().join(file)).unwrap(),
                    std::fs::read(owner.join(file))
                        .unwrap_or_else(|error| panic!("{}: {error}", owner.display())),
                    "{file}"
                );
            }
        }
        None if std::env::var_os("TL_REWRITE_WEST_SKIP").is_some_and(|value| value == "1") => {
            eprintln!("TL_REWRITE_WEST_SKIP=1: WEST byte comparison explicitly skipped");
        }
        None => panic!(
            "tl-rewrite corpus/west-v1 not found; set TL_REWRITE_WEST_DIR, or \
             TL_REWRITE_WEST_SKIP=1 to skip the byte comparison explicitly"
        ),
    }
}

// Trace: TC-327, FR-094-AC-1
#[test]
fn west_importer_refuses_an_unbounded_operator() {
    assert!(matches!(
        read_west(b"Gp0", ReaderLimits::default()),
        Err(WestError::MissionClosureRequired { offset: 0 })
    ));
    let record = read_west(b"(p0Up1)", ReaderLimits::default())
        .unwrap_err()
        .loss_record(Some(0));
    assert_eq!(record.kind, LossKind::MissionClosureRequired);
}

// ------------------------------------------------------------ NFR-047

fn label_c2po(result: Result<tl_mltl::interop::c2po::C2poImport, C2poImportError>) -> String {
    match result {
        Ok(_) => "ok".to_owned(),
        Err(C2poImportError::Reader(error)) => reader_label(&error),
        Err(other) => format!("other:{other:?}"),
    }
}

fn label_fret(result: Result<tl_mltl::interop::fret::FretMltlImport, FretImportError>) -> String {
    match result {
        Ok(import) if import.unimported.is_empty() => "ok".to_owned(),
        Ok(import) => format!("loss:{:?}", import.unimported[0].kind),
        Err(FretImportError::Reader(error)) => reader_label(&error),
        Err(other) => format!("other:{other:?}"),
    }
}

fn label_west(result: Result<tl_mltl::interop::west::WestFormula, WestError>) -> String {
    match result {
        Ok(_) => "ok".to_owned(),
        Err(WestError::Reader(error)) => reader_label(&error),
        Err(other) => format!("other:{other:?}"),
    }
}

fn reader_label(error: &ReaderError) -> String {
    match error {
        ReaderError::Empty => "empty",
        ReaderError::NonUtf8 { .. } => "non-utf8",
        ReaderError::NulByte { .. } => "nul",
        ReaderError::SourceTooLarge { .. } => "too-large",
        ReaderError::NestingTooDeep { .. } => "too-deep",
        ReaderError::TooManyTokens { .. } => "too-many-tokens",
        ReaderError::DuplicateSignal { .. } => "duplicate-signal",
    }
    .to_owned()
}

/// The hostile-case manifest: (reader, class, input, limits, expected label).
fn hostile_cases() -> Vec<(&'static str, &'static str, Vec<u8>, ReaderLimits, &'static str)> {
    let d = ReaderLimits::default();
    let sized = |text: &[u8], extra: isize| ReaderLimits {
        max_source_bytes: (text.len() as isize + extra) as usize,
        ..d
    };
    let nested = |depth: u32| ReaderLimits {
        max_nesting: depth,
        ..d
    };
    let c2po_ok = b"INPUT\n  a: bool;\nFTSPEC\n  a;\n".to_vec();
    let c2po_nest = b"INPUT\n  a: bool;\nFTSPEC\n  ((a));\n".to_vec();
    let fret_ok = b"F[0,1] p\n".to_vec();
    let fret_nest = b"((p))\n".to_vec();
    let west_ok = b"(p0&p1)".to_vec();
    let west_nest = b"~~p0".to_vec();
    vec![
        ("c2po", "empty", Vec::new(), d, "empty"),
        ("c2po", "non-utf8", b"INPUT\n  a: bool;\xff".to_vec(), d, "non-utf8"),
        ("c2po", "nul", b"INPUT\n  a: bool;\0".to_vec(), d, "nul"),
        ("c2po", "source-exact", c2po_ok.clone(), sized(&c2po_ok, 0), "ok"),
        ("c2po", "source-over", c2po_ok.clone(), sized(&c2po_ok, -1), "too-large"),
        ("c2po", "nesting-exact", c2po_nest.clone(), nested(2), "ok"),
        ("c2po", "nesting-over", c2po_nest, nested(1), "too-deep"),
        (
            "c2po",
            "duplicate-signal",
            b"INPUT\n  a, a: bool;\nFTSPEC\n  a;\n".to_vec(),
            d,
            "duplicate-signal",
        ),
        ("fret", "empty", Vec::new(), d, "empty"),
        ("fret", "non-utf8", b"F[0,1] p\xff".to_vec(), d, "non-utf8"),
        ("fret", "nul", b"F[0,1] p\0".to_vec(), d, "nul"),
        ("fret", "source-exact", fret_ok.clone(), sized(&fret_ok, 0), "ok"),
        ("fret", "source-over", fret_ok.clone(), sized(&fret_ok, -1), "too-large"),
        ("fret", "nesting-exact", fret_nest.clone(), nested(2), "ok"),
        ("fret", "nesting-over", fret_nest, nested(1), "loss:ResourceLimit"),
        ("fret", "duplicate-signal", b"(p & p)\n".to_vec(), d, "ok"),
        ("west", "empty", Vec::new(), d, "empty"),
        ("west", "non-utf8", b"p0\xff".to_vec(), d, "non-utf8"),
        ("west", "nul", b"p0\0".to_vec(), d, "nul"),
        ("west", "source-exact", west_ok.clone(), sized(&west_ok, 0), "ok"),
        ("west", "source-over", west_ok.clone(), sized(&west_ok, -1), "too-large"),
        ("west", "nesting-exact", west_nest.clone(), nested(2), "ok"),
        ("west", "nesting-over", west_nest, nested(1), "too-deep"),
        ("west", "duplicate-signal", b"(p0&p0)".to_vec(), d, "ok"),
    ]
}

// Trace: TC-330, NFR-047-AC-1
#[test]
fn hostile_cases_yield_their_manifest_outcomes() {
    let attribution = Attribution {
        repository: Some("https://github.com/NASA-SW-VnV/fret".to_owned()),
        path: Some("examples".to_owned()),
        revision: Some("0123456789abcdef".to_owned()),
        licence: Some("Apache-2.0".to_owned()),
    };
    let cases = hostile_cases();
    assert_eq!(cases.len(), 24);
    for (reader, class, input, limits, expected) in cases {
        let label = match reader {
            "c2po" => {
                let profile = Some(SemanticProfile::OnlinePrefixV1);
                label_c2po(import_c2po(&input, profile, limits))
            }
            "fret" => {
                let profile = FretProfile::ClosedTrace;
                label_fret(import_fret_mltl(&input, profile, &attribution, limits))
            }
            _ => label_west(read_west(&input, limits)),
        };
        assert_eq!(label, expected, "{reader} {class}");
    }
}

// Trace: TC-324, FR-092-AC-3
#[test]
fn writer_refuses_another_profile_and_stops_at_its_budget() {
    let nodes = vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::Not { operand: NodeId(0) }),
        Node::new(NodeKind::Not { operand: NodeId(1) }),
    ];
    let prefix = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(2), &nodes).unwrap();
    let record = write_west(prefix).unwrap_err();
    assert_eq!(record.construct, "profile");
    let closed = Formula::new(SemanticProfile::ClosedTraceV1, NodeId(2), &nodes).unwrap();
    assert_eq!(write_west_bounded(closed, 3).unwrap(), "~~p0");
    let record = write_west_bounded(closed, 2).unwrap_err();
    assert_eq!(record.kind, LossKind::ResourceLimit);
}
