//! TL-432: C2PO import (FR-086). Nothing here executes C2PO or R2U2; the
//! retained corpus bytes are read from disk.
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).

use std::path::PathBuf;

use tl_mltl::interop::c2po::{
    export_c2po, import_c2po, import_c2po_requirements, C2poImportError, UnsupportedConstruct,
};
use tl_mltl::interop::{LossKind, ReaderLimits};
use tl_syntax::SemanticProfile;

fn corpus(path: &str) -> Vec<u8> {
    std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

fn identity_round_trip(bytes: &[u8], profile: SemanticProfile) -> usize {
    let first = import_c2po(bytes, Some(profile), ReaderLimits::default()).unwrap();
    let exported = export_c2po(&first, 1_000_000).unwrap();
    let second = import_c2po(exported.as_bytes(), Some(profile), ReaderLimits::default()).unwrap();
    assert_eq!(second.signals, first.signals, "signal order");
    assert_eq!(second.proposition_map(), first.proposition_map());
    assert_eq!(second.requirements.len(), first.requirements.len());
    let mut distinct: Vec<(Vec<tl_syntax::Node>, tl_syntax::NodeId)> = Vec::new();
    for (left, right) in first.requirements.iter().zip(&second.requirements) {
        assert_eq!(left.nodes, right.nodes);
        assert_eq!(left.root, right.root);
        let graph = (left.nodes.clone(), left.root);
        if !distinct.contains(&graph) {
            distinct.push(graph);
        }
    }
    distinct.len()
}

// Trace: TC-310, FR-086-AC-1
#[test]
fn export_then_import_is_the_identity_on_the_retained_formulas() {
    let future = corpus("corpus/r2u2-v4.2/formulas.c2po");
    let past = corpus("corpus/past-c2po-v1/target-4.2/past.c2po");
    let future_import =
        import_c2po(&future, Some(SemanticProfile::OnlinePrefixV1), ReaderLimits::default()).unwrap();
    assert_eq!(future_import.requirements.len(), 6);
    let past_import = import_c2po(
        &past,
        Some(SemanticProfile::OriginCompleteHistoryV1),
        ReaderLimits::default(),
    )
    .unwrap();
    // `O[1,1](p);` appears twice: two requirements, one distinct graph.
    assert_eq!(past_import.requirements.len(), 6);
    let distinct_future = identity_round_trip(&future, SemanticProfile::OnlinePrefixV1);
    let distinct_past = identity_round_trip(&past, SemanticProfile::OriginCompleteHistoryV1);
    assert_eq!(distinct_future, 6);
    assert_eq!(distinct_past, 5);
    assert_eq!(distinct_future + distinct_past, 11);
}

fn strict(text: &str) -> Result<(), C2poImportError> {
    import_c2po(
        text.as_bytes(),
        Some(SemanticProfile::OnlinePrefixV1),
        ReaderLimits::default(),
    )
    .map(|_| ())
}

// Trace: TC-311, FR-086-AC-2
#[test]
fn each_construct_outside_the_subset_is_refused_by_name_with_a_span() {
    use UnsupportedConstruct as C;
    let header = "INPUT\n  a, b: bool;\nFTSPEC\n  ";
    let cases: Vec<(String, C, &str)> = vec![
        ("STRUCT\n  s: { x: bool; };\n".to_owned(), C::StructDeclaration, "STRUCT"),
        ("DEFINE\n  x := a;\n".to_owned(), C::DefineMacro, "DEFINE"),
        (format!("{header}a + b;\n"), C::ArithmeticOperator, "+"),
        (format!("{header}a == b;\n"), C::RelationalOperator, "=="),
        (format!("{header}abs(a);\n"), C::FunctionCall, "abs"),
        (format!("{header}foreach(x: s)(a);\n"), C::SetAggregation, "foreach"),
        ("INPUT\n  a: int;\nFTSPEC\n  a;\n".to_owned(), C::NonBooleanInputType, "int"),
        (format!("{header}a;\nCONTRACT\n  c: a => b;\n"), C::ContractSection, "CONTRACT"),
        (format!("{header}a;\nORDER\n  a;\n"), C::OrderSection, "ORDER"),
        (format!("{header}a # b;\n"), C::UnrecognizedToken, "#"),
    ];
    assert_eq!(cases.len(), UnsupportedConstruct::ALL.len());
    for (text, expected, spelled) in cases {
        match strict(&text) {
            Err(C2poImportError::UnsupportedConstruct { construct, span }) => {
                assert_eq!(construct, expected, "{text}");
                assert_eq!(&text[span.start..span.end], spelled, "{text}");
            }
            other => panic!("{text}: expected {expected:?}, got {other:?}"),
        }
    }
}

// Trace: TC-312, FR-086-AC-3
#[test]
fn profile_is_explicit_and_never_inferred() {
    let future = corpus("corpus/r2u2-v4.2/formulas.c2po");
    let past = corpus("corpus/past-c2po-v1/target-4.2/past.c2po");
    assert_eq!(
        import_c2po(&future, None, ReaderLimits::default()),
        Err(C2poImportError::ProfileAbsent)
    );
    assert!(matches!(
        import_c2po(
            &future,
            Some(SemanticProfile::OriginCompleteHistoryV1),
            ReaderLimits::default()
        ),
        Err(C2poImportError::ProfileMismatch { section: "FTSPEC", .. })
    ));
    assert!(matches!(
        import_c2po(&past, Some(SemanticProfile::OnlinePrefixV1), ReaderLimits::default()),
        Err(C2poImportError::ProfileMismatch { section: "PTSPEC", .. })
    ));
    assert!(matches!(
        import_c2po(&future, Some(SemanticProfile::ClosedTraceV1), ReaderLimits::default()),
        Err(C2poImportError::ProfileMismatch { .. })
    ));
}

// Trace: TC-332, FR-086-AC-4
#[test]
fn per_requirement_entry_conserves_requirements_and_aborts_on_file_level_constructs() {
    let text = "INPUT\n  a, b: bool;\nFTSPEC\n  F[0,1] a;\n  a + b;\n  (a U[0,2] b);\n";
    let result = import_c2po_requirements(
        text.as_bytes(),
        Some(SemanticProfile::OnlinePrefixV1),
        ReaderLimits::default(),
    )
    .unwrap();
    assert_eq!(result.declared, 3);
    assert_eq!(result.imported.len(), 2);
    assert_eq!(result.unimported.len(), 1);
    assert_eq!(result.imported.len() + result.unimported.len(), result.declared);
    let loss = &result.unimported[0];
    assert_eq!(loss.kind, LossKind::UnsupportedOperator);
    assert_eq!(loss.requirement, Some(1));
    assert_eq!(loss.construct, "arithmetic-operator");
    assert!(loss.span.is_some());

    let file_level = "DEFINE\n  x := a;\nINPUT\n  a: bool;\nFTSPEC\n  a;\n";
    assert!(matches!(
        import_c2po_requirements(
            file_level.as_bytes(),
            Some(SemanticProfile::OnlinePrefixV1),
            ReaderLimits::default()
        ),
        Err(C2poImportError::UnsupportedConstruct {
            construct: UnsupportedConstruct::DefineMacro,
            ..
        })
    ));
}
