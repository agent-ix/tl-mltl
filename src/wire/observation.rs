//! QObs C00 compatibility dispatch for the temporal owner boundary.
//!
//! Temporal handoff delegates to the strict bounded request adapter. QObs-owned
//! repair and population-query semantics remain outside this crate and are
//! represented by explicit compatibility dispositions rather than projections.

use super::{request, OwnerLimits, OwnerReadError};
use crate::QUIRE_OBSERVATION_REVISION;

/// Exact QObs C00 repair-plan contract not consumed by tl-mltl.
pub const REPAIR_PLAN_CONTRACT: &str = "quire.observation.repair-plan/v1";
/// Exact QObs C00 closed-population-query contract not consumed by tl-mltl.
pub const CLOSED_POPULATION_QUERY_CONTRACT: &str = "quire.observation.closed-population-query/v1";

/// Closed QObs C00 contract families presented to the TL consumer boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Contract {
    /// Existing temporal owner views consumed through the TL request contract.
    TemporalAssessment,
    /// QObs-owned repair-plan artifact.
    RepairPlan,
    /// QObs-owned closed-population-query evaluation.
    ClosedPopulationQuery,
}

impl Contract {
    /// Returns the exact wire contract associated with this compatibility row.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::TemporalAssessment => request::CONTRACT,
            Self::RepairPlan => REPAIR_PLAN_CONTRACT,
            Self::ClosedPopulationQuery => CLOSED_POPULATION_QUERY_CONTRACT,
        }
    }
}

/// Machine-matchable refusal to consume one QObs-owned semantic contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Unsupported {
    contract: Contract,
}

impl Unsupported {
    /// Returns the exact unsupported contract family.
    #[must_use]
    pub const fn contract(self) -> Contract {
        self.contract
    }

    /// Returns the exact unsupported wire-contract label.
    #[must_use]
    pub const fn contract_label(self) -> &'static str {
        self.contract.label()
    }

    /// Returns the exact compiled QObs revision at which support was assessed.
    #[must_use]
    pub const fn observation_revision(self) -> &'static str {
        QUIRE_OBSERVATION_REVISION
    }
}

/// Machine-matchable declaration of one QObs contract consumed by TL.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Supported {
    contract: Contract,
}

impl Supported {
    /// Returns the exact supported contract family.
    #[must_use]
    pub const fn contract(self) -> Contract {
        self.contract
    }

    /// Returns the exact supported wire-contract label.
    #[must_use]
    pub const fn contract_label(self) -> &'static str {
        self.contract.label()
    }

    /// Returns the exact compiled QObs revision at which support was assessed.
    #[must_use]
    pub const fn observation_revision(self) -> &'static str {
        QUIRE_OBSERVATION_REVISION
    }
}

/// Compatibility of one QObs C00 contract at the TL consumer boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Compatibility {
    /// The existing strict bounded temporal adapter consumes this contract.
    Supported(Supported),
    /// The contract is owned by QObs and has no TL semantic consumer.
    Unsupported(Unsupported),
}

/// Reports support without accepting or inspecting an unsupported artifact.
#[must_use]
pub const fn compatibility(contract: Contract) -> Compatibility {
    match contract {
        Contract::TemporalAssessment => Compatibility::Supported(Supported { contract }),
        Contract::RepairPlan | Contract::ClosedPopulationQuery => {
            Compatibility::Unsupported(Unsupported { contract })
        }
    }
}

/// Delegates a temporal handoff to the existing strict bounded request adapter.
///
/// No observation, repair, query, or temporal semantic is duplicated here.
pub fn consume_temporal(
    input: request::RequestInput<'_>,
    limits: OwnerLimits,
) -> Result<request::TemporalRequestDocument, OwnerReadError> {
    request::derive(input, limits)
}
