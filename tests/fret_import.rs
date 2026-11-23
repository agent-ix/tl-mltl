//! TL-431: FRET MLTL and bundle import, attribution and loss records
//! (FR-087, FR-088, FR-089, FR-091).
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).
//! TC-313 and TC-315 (published FRET examples) are not written: no FRET export
//! is retained and retrieval is DEFERRED; the spellings exercised here are the
//! provisional ones documented in `src/interop/fret.rs` (UNGROUNDED).

use tl_mltl::interop::c2po::import_c2po_requirements;
use tl_mltl::interop::fret::{import_fret_bundle, import_fret_mltl, FretImportError, FretProfile};
use tl_mltl::interop::{
    Attribution, AttributionError, ForeignFormat, LossKind, LossRecord, ReaderLimits,
    REVISION_UNKNOWN,
};
use tl_mltl::wire::OwnerLimits;
use tl_syntax::{
    NodeKind, OwnedSignalDeclaration, PropositionBinding, PropositionId, SemanticProfile,
    SignalCatalogDocument, SignalDomain, SignalId,
};

fn fret_attribution() -> Attribution {
    Attribution {
        repository: Some("https://github.com/NASA-SW-VnV/fret".to_owned()),
        path: Some("examples/test".to_owned()),
        revision: Some("0123456789abcdef0123456789abcdef01234567".to_owned()),
        licence: Some("Apache-2.0".to_owned()),
    }
}

fn fret(text: &str) -> tl_mltl::interop::fret::FretMltlImport {
    import_fret_mltl(
        text.as_bytes(),
        FretProfile::ClosedTrace,
        &fret_attribution(),
        ReaderLimits::default(),
    )
    .unwrap()
}

/// Signals: 1 `a` Boolean (proposition 0), 2 `b` Boolean (proposition 1),
/// 3 `n` Integer, 4 `c` Boolean with no proposition binding.
fn catalog() -> SignalCatalogDocument {
    SignalCatalogDocument::new(
        vec![
            OwnedSignalDeclaration::new(SignalId(1), "a".to_owned(), SignalDomain::Boolean),
            OwnedSignalDeclaration::new(SignalId(2), "b".to_owned(), SignalDomain::Boolean),
            OwnedSignalDeclaration::new(
                SignalId(3),
                "n".to_owned(),
                SignalDomain::Integer(tl_syntax::IntegerSignalDomain::new(0, 9).unwrap()),
            ),
            OwnedSignalDeclaration::new(SignalId(4), "c".to_owned(), SignalDomain::Boolean),
        ],
        vec![
            PropositionBinding::new(PropositionId(0), SignalId(1)),
            PropositionBinding::new(PropositionId(1), SignalId(2)),
        ],
    )
    .unwrap()
}

const BUNDLE_C2PO: &str = "INPUT\n  x, y: bool;\nFTSPEC\n  G[0,3] x;\n  (x U[0,2] y);\n";

fn bundle(map: &str, header: &str) -> tl_mltl::interop::fret::FretBundleImport {
    import_fret_bundle(
        BUNDLE_C2PO.as_bytes(),
        map.as_bytes(),
        header.as_bytes(),
        &catalog(),
        SemanticProfile::OnlinePrefixV1,
        &fret_attribution(),
        ReaderLimits::default(),
    )
    .unwrap()
}

// Trace: TC-314, FR-087-AC-2
#[test]
fn bare_next_is_ambiguous_and_never_lowered() {
    let result = fret("r1: G[0,2] (p -> X q)\nr2: F[0,1] p\n");
    assert_eq!(result.declared, 2);
    assert_eq!(result.imported.len(), 1);
    assert_eq!(result.imported[0].id.as_deref(), Some("r2"));
    let loss = &result.unimported[0];
    assert_eq!(loss.kind, LossKind::AmbiguousNext);
    assert_eq!(loss.construct, "X");
    assert_eq!(loss.requirement, Some(0));
    // Nothing was imported as a strong or weak next.
    for requirement in &result.imported {
        assert!(requirement
            .nodes
            .iter()
            .all(|node| !matches!(
                node.kind,
                NodeKind::StrongNext { .. } | NodeKind::WeakNext { .. }
            )));
    }
}

// Trace: TC-316, FR-088-AC-2
#[test]
fn unmapped_duplicate_non_boolean_and_unknown_header_fields_yield_loss_records() {
    // Clean bundle first: both variables bound, header kept.
    let clean = bundle("x:1\ny:2\n", "# x,y\n# mission-time: 5\n");
    assert_eq!(clean.bindings.len(), 2);
    assert_eq!(clean.header.mission_time, Some(5));
    assert_eq!(clean.header.signals, Some(vec!["x".to_owned(), "y".to_owned()]));
    assert!(clean.bundle_losses.is_empty());
    assert_eq!(clean.imported.len(), 2);

    // y unmapped: requirement 1 is unimported, nothing binds y.
    let unmapped = bundle("x:1\n", "# x,y\n");
    assert!(unmapped
        .bundle_losses
        .iter()
        .any(|loss| loss.kind == LossKind::UnmappedVariable && loss.construct == "y"));
    assert_eq!(unmapped.imported.len(), 1);
    assert_eq!(unmapped.unimported[0].kind, LossKind::UnmappedVariable);

    // Two entries for x: neither binds.
    let duplicate = bundle("x:1\nx:2\ny:4\n", "");
    assert!(duplicate
        .bundle_losses
        .iter()
        .any(|loss| loss.kind == LossKind::DuplicateVariable && loss.construct == "x"));
    assert!(duplicate.bindings.iter().all(|binding| binding.variable != "x"));

    // y to an Integer signal.
    let numeric = bundle("x:1\ny:3\n", "");
    assert!(numeric
        .bundle_losses
        .iter()
        .any(|loss| loss.kind == LossKind::UnsupportedSignalType && loss.construct == "y"));
    assert!(numeric.bindings.iter().all(|binding| binding.variable != "y"));

    // Unknown header field is recorded and does not abort.
    let header = bundle("x:1\ny:2\n", "# color: red\n");
    assert!(header
        .bundle_losses
        .iter()
        .any(|loss| loss.kind == LossKind::UnknownHeaderField && loss.construct == "color"));
    assert_eq!(header.imported.len(), 2);
}

// Trace: TC-317, FR-089-AC-1, FR-089-AC-3
#[test]
fn attribution_is_carried_and_incomplete_attribution_is_refused() {
    let result = fret("F[0,1] p\n");
    assert_eq!(result.attribution, fret_attribution());

    let missing = [
        (
            Attribution {
                repository: None,
                ..fret_attribution()
            },
            AttributionError::MissingRepository,
        ),
        (
            Attribution {
                path: None,
                ..fret_attribution()
            },
            AttributionError::MissingPath,
        ),
        (
            Attribution {
                revision: None,
                ..fret_attribution()
            },
            AttributionError::MissingRevision,
        ),
        (
            Attribution {
                licence: None,
                ..fret_attribution()
            },
            AttributionError::MissingLicence,
        ),
    ];
    for (attribution, expected) in missing {
        assert_eq!(
            import_fret_mltl(
                b"F[0,1] p\n",
                FretProfile::ClosedTrace,
                &attribution,
                ReaderLimits::default()
            ),
            Err(FretImportError::Attribution(expected))
        );
    }
    // FRET needs a real revision; the UNKNOWN marker does not satisfy it.
    let unknown = Attribution {
        revision: Some(REVISION_UNKNOWN.to_owned()),
        ..fret_attribution()
    };
    assert_eq!(unknown.validate_fret(), Err(AttributionError::MissingRevision));

    // WEST: the marker is accepted, an absent revision is refused, nothing is synthesized.
    let west = Attribution {
        repository: Some("https://github.com/zwang271/WEST".to_owned()),
        path: Some("experiments/verification/verify_formulas/formulas_d1.txt".to_owned()),
        revision: Some(REVISION_UNKNOWN.to_owned()),
        licence: Some("MIT".to_owned()),
    };
    assert_eq!(west.validate_west(), Ok(()));
    assert_eq!(west.revision.as_deref(), Some("UNKNOWN"));
    for broken in [
        Attribution {
            revision: None,
            ..west.clone()
        },
        Attribution {
            repository: None,
            ..west.clone()
        },
        Attribution {
            path: None,
            ..west.clone()
        },
        Attribution {
            licence: None,
            ..west.clone()
        },
    ] {
        assert!(broken.validate_west().is_err());
    }
}

/// One fixture per loss kind.
fn fixture(kind: LossKind) -> LossRecord {
    let c2po = |text: &str, limits: ReaderLimits| {
        import_c2po_requirements(text.as_bytes(), Some(SemanticProfile::OnlinePrefixV1), limits)
            .unwrap()
            .unimported
            .remove(0)
    };
    let header = "INPUT\n  a, b: bool;\nFTSPEC\n  ";
    match kind {
        LossKind::UnsupportedOperator => {
            c2po(&format!("{header}a + b;\n"), ReaderLimits::default())
        }
        LossKind::AmbiguousNext => fret("X p\n").unimported.remove(0),
        LossKind::UnsupportedSignalType => bundle("x:1\ny:3\n", "")
            .bundle_losses
            .into_iter()
            .find(|loss| loss.kind == LossKind::UnsupportedSignalType)
            .unwrap(),
        LossKind::UnsupportedInterval => {
            c2po(&format!("{header}F[2,1] a;\n"), ReaderLimits::default())
        }
        LossKind::UnmappedVariable => {
            c2po(&format!("{header}F[0,1] c;\n"), ReaderLimits::default())
        }
        LossKind::DuplicateVariable => bundle("x:1\nx:2\n", "")
            .bundle_losses
            .into_iter()
            .find(|loss| loss.kind == LossKind::DuplicateVariable)
            .unwrap(),
        LossKind::UnknownHeaderField => bundle("x:1\ny:2\n", "# color: red\n")
            .bundle_losses
            .remove(0),
        LossKind::MissionClosureRequired => fret("G p\n").unimported.remove(0),
        LossKind::ForeignConstruct => fret("the system shall always hold p\n").unimported.remove(0),
        LossKind::ResourceLimit => c2po(
            &format!("{header}((((a))));\n"),
            ReaderLimits {
                max_nesting: 3,
                ..ReaderLimits::default()
            },
        ),
    }
}

// Trace: TC-320, FR-091-AC-1, StR-044-VC-2
#[test]
fn each_loss_kind_is_produced_by_its_fixture_and_round_trips() {
    for kind in LossKind::ALL {
        let record = fixture(kind);
        assert_eq!(record.kind, kind);
        assert!(!record.construct.is_empty());
        assert!(record.span.is_some() || record.no_span_reason.is_some());
        let format = match kind {
            LossKind::UnsupportedOperator
            | LossKind::UnsupportedInterval
            | LossKind::UnmappedVariable
            | LossKind::ResourceLimit => ForeignFormat::C2po,
            LossKind::AmbiguousNext
            | LossKind::MissionClosureRequired
            | LossKind::ForeignConstruct => {
                ForeignFormat::FretMltl
            }
            LossKind::UnsupportedSignalType
            | LossKind::DuplicateVariable
            | LossKind::UnknownHeaderField => ForeignFormat::FretBundle,
        };
        assert_eq!(record.format, format, "{kind:?}");
        let bytes = record.to_json_bytes();
        assert_eq!(
            LossRecord::from_json_bytes(&bytes, OwnerLimits::default()).unwrap(),
            record
        );
    }
}

// Trace: TC-321, FR-091-AC-2, StR-044-VC-2
#[test]
fn imported_plus_unimported_equals_declared_on_every_fixture() {
    let fret_files = [
        "r1: F[0,1] p\nr2: X p\nr3: G p\n# comment\nr4: (p & q) -> F[1,2] q\n",
        "the system shall respond\nG[0,3] (p | ~q)\n",
        "F[0,1] p\n\n\nG[1,1] q\n",
    ];
    for text in fret_files {
        let result = fret(text);
        assert_eq!(result.imported.len() + result.unimported.len(), result.declared, "{text}");
        let mut indices: Vec<usize> = result.imported.iter().map(|r| r.index).collect();
        indices.extend(result.unimported.iter().filter_map(|loss| loss.requirement));
        indices.sort_unstable();
        assert_eq!(indices, (0..result.declared).collect::<Vec<_>>(), "no requirement vanishes");
    }
    let c2po = "INPUT\n  a, b: bool;\nFTSPEC\n  a;\n  a == b;\n  abs(a);\n  G[0,1] b;\n";
    let result = import_c2po_requirements(
        c2po.as_bytes(),
        Some(SemanticProfile::OnlinePrefixV1),
        ReaderLimits::default(),
    )
    .unwrap();
    assert_eq!(result.declared, 4);
    assert_eq!(result.imported.len() + result.unimported.len(), 4);
    let bundled = bundle("x:1\n", "");
    assert_eq!(bundled.imported.len() + bundled.unimported.len(), bundled.declared);
}
