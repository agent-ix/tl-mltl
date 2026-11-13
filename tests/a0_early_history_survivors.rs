//! TL-371: behavior tests for the position-history owner reader's usage accounting and ceilings,
//! which the first full cargo-mutants run showed were not asserted at their boundaries.

use tl_mltl::past::history;
use tl_mltl::wire::{OwnerLimits, OwnerReadErrorCode};
use tl_mltl::{ClockBinding, PositionHistoryDocument, PositionObservation};
use tl_syntax::PropositionId;

fn document() -> PositionHistoryDocument {
    PositionHistoryDocument::new(
        "h",
        1,
        0,
        2,
        Some(ClockBinding::EventPosition),
        (0..3)
            .map(|p| PositionObservation::new(p, vec![PropositionId(0)], None))
            .collect(),
    )
    .unwrap()
}

// Trace: TC-090
#[test]
fn history_usage_counts_positions_propositions_and_span() {
    let owner = history::derive(&document(), OwnerLimits::default()).unwrap();
    let usage = owner.usage();
    assert_eq!(
        (usage.positions, usage.propositions, usage.history_span),
        (3, 3, 2)
    );
    let read = history::read(owner.bytes(), &document(), OwnerLimits::default()).unwrap();
    assert_eq!(
        (
            read.usage().positions,
            read.usage().propositions,
            read.usage().history_span
        ),
        (3, 3, 2)
    );
}

// Trace: TC-090
#[test]
fn each_history_ceiling_is_inclusive_and_refuses_one_below() {
    let default = OwnerLimits::default();
    let refused = |limits: OwnerLimits| {
        let error = history::derive(&document(), limits).err().unwrap();
        (error.code(), error.field())
    };
    let resource = OwnerReadErrorCode::ResourceIncomplete;
    assert!(history::derive(
        &document(),
        OwnerLimits {
            max_positions: 3,
            ..default
        }
    )
    .is_ok());
    assert_eq!(
        refused(OwnerLimits {
            max_positions: 2,
            ..default
        }),
        (resource, "positions")
    );
    assert!(history::derive(
        &document(),
        OwnerLimits {
            max_propositions: 3,
            ..default
        }
    )
    .is_ok());
    assert_eq!(
        refused(OwnerLimits {
            max_propositions: 2,
            ..default
        }),
        (resource, "propositions")
    );
    assert!(history::derive(
        &document(),
        OwnerLimits {
            max_history_span: 2,
            ..default
        }
    )
    .is_ok());
    assert_eq!(
        refused(OwnerLimits {
            max_history_span: 1,
            ..default
        }),
        (resource, "historySpan")
    );
}
