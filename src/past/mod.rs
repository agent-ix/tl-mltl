//! Origin-complete past-time history, requirement, result, and evaluation subsystem.

pub mod evaluate;
pub mod history;
pub mod requirement;
pub mod result;

use core::fmt;

use serde::{Deserialize, Serialize};
use tl_syntax::{
    Formula, FormulaDocument, NodeId, NodeKind, PropositionId, SemanticProfile, PAST_OPERATORS_V1,
};

use crate::{context::domain_sha256, MAX_RECURSION_DEPTH, TL_SYNTAX_REVISION};

/// Position-history wire identity.
pub const POSITION_HISTORY_V1: &str = "tl-mltl.position-history/v1";
/// Required-history analysis wire identity.
pub const HISTORY_REQUIREMENT_V1: &str = "tl-mltl.history-requirement/v1";
/// Past-evaluation result wire identity.
pub const PAST_EVALUATION_V1: &str = "tl-mltl.past-evaluation/v1";
/// Past evaluator identity.
pub const PAST_EVALUATOR_V1: &str = "tl-mltl.past-evaluator/v1";

const MAX_IDENTITY_BYTES: usize = 256;
const MAX_UNIT_BYTES: usize = 128;
const MAX_HISTORY_POSITIONS: usize = 1_000_000;
const MAX_PROPOSITIONS_PER_POSITION: usize = 100_000;
const HARD_MAX_PAST_STEPS: u64 = 4_000_000;
const HARD_MAX_TEMPORAL_SPAN: u64 = 100_000;

struct BoundedStringVisitor {
    field: &'static str,
    max: usize,
}

impl serde::de::Visitor<'_> for BoundedStringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} with at most {} UTF-8 bytes",
            self.field, self.max
        )
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_string(value.to_owned())
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if value.len() > self.max {
            return Err(E::invalid_length(value.len(), &self));
        }
        Ok(value)
    }
}

fn deserialize_identity<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_string(BoundedStringVisitor {
        field: "identity",
        max: MAX_IDENTITY_BYTES,
    })
}

fn deserialize_unit<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_string(BoundedStringVisitor {
        field: "clock unit",
        max: MAX_UNIT_BYTES,
    })
}

fn deserialize_digest<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_string(BoundedStringVisitor {
        field: "SHA-256 digest",
        max: 64,
    })
}

/// Closed position-history schema identity.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PositionHistorySchemaVersion {
    /// Initial origin-complete position history.
    #[serde(rename = "tl-mltl.position-history/v1")]
    V1,
}

/// Closed required-history report identity.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum HistoryRequirementSchemaVersion {
    /// Initial checked reverse-offset analysis.
    #[serde(rename = "tl-mltl.history-requirement/v1")]
    V1,
}

/// Closed past-evaluation result identity.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PastEvaluationSchemaVersion {
    /// Initial anchored Boolean past result.
    #[serde(rename = "tl-mltl.past-evaluation/v1")]
    V1,
}

/// Closed past-evaluator identity.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PastEvaluatorIdentity {
    /// The evaluator implementing `mltl.origin-complete-history/v1`.
    #[serde(rename = "tl-mltl.past-evaluator/v1")]
    V1,
}

/// An exact normalized rational number.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", try_from = "ExactNumberWire")]
pub struct ExactNumber {
    numerator: i64,
    denominator: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExactNumberWire {
    numerator: i64,
    denominator: u64,
}

/// Exact-number construction refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExactNumberError {
    /// A rational denominator must be positive.
    ZeroDenominator,
    /// The supplied numerator and denominator are not in lowest terms.
    NotNormalized,
    /// A fixed-sample period must be strictly positive.
    NonPositivePeriod,
    /// Exact arithmetic exceeded the admitted `i64`/`u64` representation.
    ArithmeticOverflow,
}

impl fmt::Display for ExactNumberError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroDenominator => formatter.write_str("exact denominator is zero"),
            Self::NotNormalized => formatter.write_str("exact number is not normalized"),
            Self::NonPositivePeriod => formatter.write_str("fixed-sample period is not positive"),
            Self::ArithmeticOverflow => formatter.write_str("exact-number arithmetic overflowed"),
        }
    }
}

impl std::error::Error for ExactNumberError {}

impl TryFrom<ExactNumberWire> for ExactNumber {
    type Error = ExactNumberError;

    fn try_from(wire: ExactNumberWire) -> Result<Self, Self::Error> {
        Self::new(wire.numerator, wire.denominator)
    }
}

impl ExactNumber {
    /// Constructs an exact number only when its representation is normalized.
    pub fn new(numerator: i64, denominator: u64) -> Result<Self, ExactNumberError> {
        if denominator == 0 {
            return Err(ExactNumberError::ZeroDenominator);
        }
        if gcd_u64(numerator.unsigned_abs(), denominator) != 1 {
            return Err(ExactNumberError::NotNormalized);
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }

    /// Returns the signed numerator.
    pub const fn numerator(self) -> i64 {
        self.numerator
    }

    /// Returns the positive denominator.
    pub const fn denominator(self) -> u64 {
        self.denominator
    }
}

fn gcd_u64(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

fn gcd_u128(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

fn normalized_i128(numerator: i128, denominator: u128) -> Result<ExactNumber, ExactNumberError> {
    if denominator == 0 {
        return Err(ExactNumberError::ZeroDenominator);
    }
    let divisor = gcd_u128(numerator.unsigned_abs(), denominator);
    let numerator =
        numerator / i128::try_from(divisor).map_err(|_| ExactNumberError::ArithmeticOverflow)?;
    let denominator = denominator / divisor;
    ExactNumber::new(
        i64::try_from(numerator).map_err(|_| ExactNumberError::ArithmeticOverflow)?,
        u64::try_from(denominator).map_err(|_| ExactNumberError::ArithmeticOverflow)?,
    )
}

/// A runtime sample coordinate supplied for one fixed-sample position.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClockSample {
    /// Exact sample instant.
    pub instant: ExactNumber,
    /// Opaque clock unit, compared byte-for-byte.
    #[serde(deserialize_with = "deserialize_unit")]
    pub unit: String,
}

/// Unsupported clock forms retained as distinct typed refusals.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnsupportedClockKind {
    /// Dense-time observations.
    Dense,
    /// Arbitrary timestamped observations.
    Timestamped,
    /// Duration reinterpretation of formula bounds.
    Duration,
    /// Rounded sample coordinates.
    Rounded,
    /// Implicitly resampled observations.
    Resampled,
    /// Wall-clock inferred positions.
    WallClock,
}

/// Declared position clock.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClockBinding {
    /// Formula positions are explicit event positions with no timestamp field.
    EventPosition,
    /// Formula positions map exactly to `epoch + position * period` in one unit.
    FixedSample {
        /// Exact normalized epoch.
        epoch: ExactNumber,
        /// Exact normalized strictly positive period.
        period: ExactNumber,
        /// Opaque nonempty unit name.
        #[serde(deserialize_with = "deserialize_unit")]
        unit: String,
    },
    /// A deliberately unsupported clock form.
    Unsupported {
        /// Preserved owner classification.
        clock_kind: UnsupportedClockKind,
    },
}

/// Clock admission refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClockError {
    /// A fixed-sample period is zero or negative.
    NonPositivePeriod,
    /// Unit is empty.
    EmptyUnit,
    /// Unit exceeds the fixed byte limit.
    UnitTooLong { actual: usize, limit: usize },
    /// A clock form has no origin-complete position mapping.
    Unsupported { kind: UnsupportedClockKind },
    /// Event-position input carried a timestamp/sample coordinate.
    UnexpectedSample { position: u64 },
    /// Runtime sample unit differs byte-for-byte from the binding.
    SampleUnitMismatch { position: u64 },
    /// Runtime sample instant does not equal `epoch + position * period`.
    SampleInstantMismatch {
        position: u64,
        expected: ExactNumber,
        actual: ExactNumber,
    },
    /// Exact sample-position arithmetic overflowed.
    ArithmeticOverflow { position: u64 },
}

impl fmt::Display for ClockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonPositivePeriod => formatter.write_str("fixed-sample period is not positive"),
            Self::EmptyUnit => formatter.write_str("clock unit is empty"),
            Self::UnitTooLong { actual, limit } => {
                write!(formatter, "clock unit length {actual} exceeds {limit}")
            }
            Self::Unsupported { kind } => write!(formatter, "unsupported clock kind {kind:?}"),
            Self::UnexpectedSample { position } => {
                write!(
                    formatter,
                    "event position {position} carries a sample instant"
                )
            }
            Self::SampleUnitMismatch { position } => {
                write!(
                    formatter,
                    "fixed-sample position {position} has a different unit"
                )
            }
            Self::SampleInstantMismatch { position, .. } => {
                write!(
                    formatter,
                    "fixed-sample position {position} has a different instant"
                )
            }
            Self::ArithmeticOverflow { position } => {
                write!(
                    formatter,
                    "fixed-sample position {position} overflows exact arithmetic"
                )
            }
        }
    }
}

impl std::error::Error for ClockError {}

impl ClockBinding {
    fn validate_static(&self) -> Result<(), ClockError> {
        match self {
            Self::EventPosition => Ok(()),
            Self::FixedSample { period, unit, .. } => {
                validate_unit(unit)?;
                if period.numerator <= 0 {
                    return Err(ClockError::NonPositivePeriod);
                }
                Ok(())
            }
            Self::Unsupported { clock_kind } => Err(ClockError::Unsupported { kind: *clock_kind }),
        }
    }

    fn validate_sample(&self, observation: &PositionObservation) -> Result<(), HistoryError> {
        match self {
            Self::EventPosition => {
                if observation.sample.is_some() {
                    Err(ClockError::UnexpectedSample {
                        position: observation.position,
                    }
                    .into())
                } else {
                    Ok(())
                }
            }
            Self::FixedSample {
                epoch,
                period,
                unit,
            } => {
                let sample = observation
                    .sample
                    .as_ref()
                    .ok_or(HistoryError::IncompleteSample {
                        position: observation.position,
                    })?;
                validate_unit(&sample.unit)?;
                if sample.unit != *unit {
                    return Err(ClockError::SampleUnitMismatch {
                        position: observation.position,
                    }
                    .into());
                }
                let expected = fixed_sample_instant(*epoch, *period, observation.position)
                    .map_err(|_| {
                        HistoryError::Clock(ClockError::ArithmeticOverflow {
                            position: observation.position,
                        })
                    })?;
                if sample.instant != expected {
                    return Err(ClockError::SampleInstantMismatch {
                        position: observation.position,
                        expected,
                        actual: sample.instant,
                    }
                    .into());
                }
                Ok(())
            }
            Self::Unsupported { clock_kind } => {
                Err(ClockError::Unsupported { kind: *clock_kind }.into())
            }
        }
    }
}

fn validate_unit(unit: &str) -> Result<(), ClockError> {
    if unit.is_empty() {
        return Err(ClockError::EmptyUnit);
    }
    if unit.len() > MAX_UNIT_BYTES {
        return Err(ClockError::UnitTooLong {
            actual: unit.len(),
            limit: MAX_UNIT_BYTES,
        });
    }
    Ok(())
}

/// Computes one fixed-sample instant with checked exact arithmetic.
pub fn fixed_sample_instant(
    epoch: ExactNumber,
    period: ExactNumber,
    position: u64,
) -> Result<ExactNumber, ExactNumberError> {
    if period.numerator <= 0 {
        return Err(ExactNumberError::NonPositivePeriod);
    }
    let epoch_denominator = u128::from(epoch.denominator);
    let period_denominator = u128::from(period.denominator);
    let gcd = gcd_u128(epoch_denominator, period_denominator);
    let common_denominator = epoch_denominator
        .checked_div(gcd)
        .and_then(|value| value.checked_mul(period_denominator))
        .ok_or(ExactNumberError::ArithmeticOverflow)?;
    let epoch_scale = common_denominator / epoch_denominator;
    let period_scale = common_denominator / period_denominator;
    let epoch_numerator = i128::from(epoch.numerator)
        .checked_mul(i128::try_from(epoch_scale).map_err(|_| ExactNumberError::ArithmeticOverflow)?)
        .ok_or(ExactNumberError::ArithmeticOverflow)?;
    let period_numerator = i128::from(period.numerator)
        .checked_mul(
            i128::try_from(period_scale).map_err(|_| ExactNumberError::ArithmeticOverflow)?,
        )
        .and_then(|value| value.checked_mul(i128::from(position)))
        .ok_or(ExactNumberError::ArithmeticOverflow)?;
    normalized_i128(
        epoch_numerator
            .checked_add(period_numerator)
            .ok_or(ExactNumberError::ArithmeticOverflow)?,
        common_denominator,
    )
}

/// One exact position and its total Boolean valuation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", try_from = "PositionObservationWire")]
pub struct PositionObservation {
    /// Absolute discrete position.
    pub position: u64,
    /// Strictly increasing identities whose values are true at this position.
    pub true_propositions: Vec<PropositionId>,
    /// Required only for fixed-sample clocks and forbidden for event positions.
    pub sample: Option<ClockSample>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PositionObservationWire {
    position: u64,
    #[serde(deserialize_with = "deserialize_propositions")]
    true_propositions: Vec<PropositionId>,
    sample: Option<ClockSample>,
}

impl TryFrom<PositionObservationWire> for PositionObservation {
    type Error = &'static str;

    fn try_from(wire: PositionObservationWire) -> Result<Self, Self::Error> {
        Ok(Self {
            position: wire.position,
            true_propositions: wire.true_propositions,
            sample: wire.sample,
        })
    }
}

impl PositionObservation {
    /// Constructs one observation; history admission validates ordering and clock fields.
    pub fn new(
        position: u64,
        true_propositions: Vec<PropositionId>,
        sample: Option<ClockSample>,
    ) -> Self {
        Self {
            position,
            true_propositions,
            sample,
        }
    }
}

fn deserialize_propositions<'de, D>(deserializer: D) -> Result<Vec<PropositionId>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct PropositionVisitor;

    impl<'de> serde::de::Visitor<'de> for PropositionVisitor {
        type Value = Vec<PropositionId>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                formatter,
                "at most {MAX_PROPOSITIONS_PER_POSITION} proposition identities"
            )
        }

        fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut propositions = Vec::with_capacity(
                sequence
                    .size_hint()
                    .unwrap_or(0)
                    .min(MAX_PROPOSITIONS_PER_POSITION),
            );
            while let Some(proposition) = sequence.next_element()? {
                if propositions.len() == MAX_PROPOSITIONS_PER_POSITION {
                    return Err(serde::de::Error::custom(
                        "too many propositions at one position",
                    ));
                }
                propositions.push(proposition);
            }
            Ok(propositions)
        }
    }

    deserializer.deserialize_seq(PropositionVisitor)
}

/// A complete immutable position-history document.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", try_from = "PositionHistoryWire")]
pub struct PositionHistoryDocument {
    schema_version: PositionHistorySchemaVersion,
    history_id: String,
    revision: u64,
    history_sha256: String,
    origin_position: u64,
    through_position: u64,
    clock: Option<ClockBinding>,
    observations: Vec<PositionObservation>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PositionHistoryWire {
    schema_version: PositionHistorySchemaVersion,
    #[serde(deserialize_with = "deserialize_identity")]
    history_id: String,
    revision: u64,
    #[serde(deserialize_with = "deserialize_digest")]
    history_sha256: String,
    origin_position: u64,
    through_position: u64,
    clock: Option<ClockBinding>,
    #[serde(deserialize_with = "deserialize_observations")]
    observations: Vec<PositionObservation>,
}

fn deserialize_observations<'de, D>(deserializer: D) -> Result<Vec<PositionObservation>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct ObservationVisitor;

    impl<'de> serde::de::Visitor<'de> for ObservationVisitor {
        type Value = Vec<PositionObservation>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                formatter,
                "at most {MAX_HISTORY_POSITIONS} position observations"
            )
        }

        fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut observations =
                Vec::with_capacity(sequence.size_hint().unwrap_or(0).min(MAX_HISTORY_POSITIONS));
            while let Some(observation) = sequence.next_element()? {
                if observations.len() == MAX_HISTORY_POSITIONS {
                    return Err(serde::de::Error::custom(
                        "position history exceeds wire limit",
                    ));
                }
                observations.push(observation);
            }
            Ok(observations)
        }
    }

    deserializer.deserialize_seq(ObservationVisitor)
}

/// Position-history admission refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HistoryError {
    /// Stable identity is empty.
    EmptyHistoryIdentity,
    /// Stable identity exceeds its fixed byte bound.
    HistoryIdentityTooLong { actual: usize, limit: usize },
    /// Revisions begin at one.
    RevisionZero,
    /// A corrected history revision must strictly advance.
    RevisionNotAdvanced { previous: u64, proposed: u64 },
    /// Clock binding is absent.
    MissingClock,
    /// Clock binding or runtime sample is invalid.
    Clock(ClockError),
    /// A fixed-sample position has not arrived and the history is incomplete.
    IncompleteSample { position: u64 },
    /// History has no position.
    EmptyHistory,
    /// Origin is not exactly zero.
    OriginNotZero { actual: u64 },
    /// Two observations carry the same position.
    DuplicatePosition { position: u64 },
    /// Observation positions descend.
    OutOfOrder { previous: u64, current: u64 },
    /// An expected physical position is absent after origin.
    Gap { expected: u64, found: u64 },
    /// Declared through-position does not equal the final observation.
    ThroughPositionMismatch { declared: u64, observed: u64 },
    /// A valuation contains duplicate or descending proposition identities.
    PropositionsNotStrictlyOrdered {
        position: u64,
        previous: PropositionId,
        current: PropositionId,
    },
    /// One position contains more proposition identities than admitted.
    PropositionLimitExceeded {
        position: u64,
        actual: usize,
        limit: usize,
    },
    /// The document contains too many observations.
    PositionLimitExceeded { actual: usize, limit: usize },
    /// Declared digest does not match the immutable history bytes.
    StaleDigest { expected: String, actual: String },
    /// Canonical history serialization failed.
    IdentityEncoding,
}

impl fmt::Display for HistoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyHistoryIdentity => formatter.write_str("history identity is empty"),
            Self::HistoryIdentityTooLong { actual, limit } => {
                write!(
                    formatter,
                    "history identity length {actual} exceeds {limit}"
                )
            }
            Self::RevisionZero => formatter.write_str("history revision is zero"),
            Self::RevisionNotAdvanced { previous, proposed } => write!(
                formatter,
                "corrected history revision {proposed} does not advance {previous}"
            ),
            Self::MissingClock => formatter.write_str("history clock binding is absent"),
            Self::Clock(error) => error.fmt(formatter),
            Self::IncompleteSample { position } => write!(
                formatter,
                "fixed-sample history is incomplete at position {position}"
            ),
            Self::EmptyHistory => formatter.write_str("position history is empty"),
            Self::OriginNotZero { actual } => {
                write!(formatter, "history origin is {actual}, not zero")
            }
            Self::DuplicatePosition { position } => {
                write!(formatter, "duplicate position {position}")
            }
            Self::OutOfOrder { previous, current } => {
                write!(
                    formatter,
                    "position {current} follows later position {previous}"
                )
            }
            Self::Gap { expected, found } => {
                write!(
                    formatter,
                    "history gap: expected position {expected}, found {found}"
                )
            }
            Self::ThroughPositionMismatch { declared, observed } => write!(
                formatter,
                "declared through-position {declared} differs from final observation {observed}"
            ),
            Self::PropositionsNotStrictlyOrdered {
                position,
                previous,
                current,
            } => write!(
                formatter,
                "position {position} proposition {} does not follow {}",
                current.0, previous.0
            ),
            Self::PropositionLimitExceeded {
                position,
                actual,
                limit,
            } => write!(
                formatter,
                "position {position} proposition count {actual} exceeds {limit}"
            ),
            Self::PositionLimitExceeded { actual, limit } => {
                write!(formatter, "history position count {actual} exceeds {limit}")
            }
            Self::StaleDigest { .. } => formatter.write_str("history digest is stale"),
            Self::IdentityEncoding => formatter.write_str("history identity encoding failed"),
        }
    }
}

impl std::error::Error for HistoryError {}

impl From<ClockError> for HistoryError {
    fn from(error: ClockError) -> Self {
        Self::Clock(error)
    }
}

impl TryFrom<PositionHistoryWire> for PositionHistoryDocument {
    type Error = HistoryError;

    fn try_from(wire: PositionHistoryWire) -> Result<Self, Self::Error> {
        match wire.schema_version {
            PositionHistorySchemaVersion::V1 => {}
        }
        Self::from_declared(
            wire.history_id,
            wire.revision,
            wire.history_sha256,
            wire.origin_position,
            wire.through_position,
            wire.clock,
            wire.observations,
        )
    }
}

impl PositionHistoryDocument {
    /// Constructs a history and computes its immutable digest.
    pub fn new(
        history_id: impl Into<String>,
        revision: u64,
        origin_position: u64,
        through_position: u64,
        clock: Option<ClockBinding>,
        observations: Vec<PositionObservation>,
    ) -> Result<Self, HistoryError> {
        let mut document = Self {
            schema_version: PositionHistorySchemaVersion::V1,
            history_id: history_id.into(),
            revision,
            history_sha256: String::new(),
            origin_position,
            through_position,
            clock,
            observations,
        };
        document.validate_without_digest()?;
        document.history_sha256 = document.compute_sha256()?;
        Ok(document)
    }

    /// Admits a document carrying a declared digest and refuses stale bytes.
    #[allow(clippy::too_many_arguments)]
    pub fn from_declared(
        history_id: impl Into<String>,
        revision: u64,
        history_sha256: impl Into<String>,
        origin_position: u64,
        through_position: u64,
        clock: Option<ClockBinding>,
        observations: Vec<PositionObservation>,
    ) -> Result<Self, HistoryError> {
        let document = Self {
            schema_version: PositionHistorySchemaVersion::V1,
            history_id: history_id.into(),
            revision,
            history_sha256: history_sha256.into(),
            origin_position,
            through_position,
            clock,
            observations,
        };
        document.validate()?;
        Ok(document)
    }

    /// Creates a new immutable revision with replacement observations.
    pub fn corrected(
        &self,
        revision: u64,
        through_position: u64,
        observations: Vec<PositionObservation>,
    ) -> Result<Self, HistoryError> {
        if revision <= self.revision {
            return Err(HistoryError::RevisionNotAdvanced {
                previous: self.revision,
                proposed: revision,
            });
        }
        Self::new(
            self.history_id.clone(),
            revision,
            self.origin_position,
            through_position,
            self.clock.clone(),
            observations,
        )
    }

    /// Revalidates structure, clock mapping, and immutable digest.
    pub fn validate(&self) -> Result<(), HistoryError> {
        self.validate_without_digest()?;
        let expected = self.compute_sha256()?;
        if self.history_sha256 != expected {
            return Err(HistoryError::StaleDigest {
                expected,
                actual: self.history_sha256.clone(),
            });
        }
        Ok(())
    }

    fn validate_without_digest(&self) -> Result<(), HistoryError> {
        validate_identity(&self.history_id)?;
        if self.revision == 0 {
            return Err(HistoryError::RevisionZero);
        }
        if self.origin_position != 0 {
            return Err(HistoryError::OriginNotZero {
                actual: self.origin_position,
            });
        }
        if self.observations.len() > MAX_HISTORY_POSITIONS {
            return Err(HistoryError::PositionLimitExceeded {
                actual: self.observations.len(),
                limit: MAX_HISTORY_POSITIONS,
            });
        }
        let clock = self.clock.as_ref().ok_or(HistoryError::MissingClock)?;
        clock.validate_static()?;
        let first = self
            .observations
            .first()
            .ok_or(HistoryError::EmptyHistory)?;
        if first.position != self.origin_position {
            return Err(HistoryError::Gap {
                expected: self.origin_position,
                found: first.position,
            });
        }
        let mut previous_position = first.position;
        for (index, observation) in self.observations.iter().enumerate() {
            if index > 0 {
                if observation.position == previous_position {
                    return Err(HistoryError::DuplicatePosition {
                        position: observation.position,
                    });
                }
                if observation.position < previous_position {
                    return Err(HistoryError::OutOfOrder {
                        previous: previous_position,
                        current: observation.position,
                    });
                }
                let expected = previous_position.checked_add(1).ok_or(HistoryError::Gap {
                    expected: previous_position,
                    found: observation.position,
                })?;
                if observation.position != expected {
                    return Err(HistoryError::Gap {
                        expected,
                        found: observation.position,
                    });
                }
                previous_position = observation.position;
            }
            if observation.true_propositions.len() > MAX_PROPOSITIONS_PER_POSITION {
                return Err(HistoryError::PropositionLimitExceeded {
                    position: observation.position,
                    actual: observation.true_propositions.len(),
                    limit: MAX_PROPOSITIONS_PER_POSITION,
                });
            }
            for pair in observation.true_propositions.windows(2) {
                if pair[0] >= pair[1] {
                    return Err(HistoryError::PropositionsNotStrictlyOrdered {
                        position: observation.position,
                        previous: pair[0],
                        current: pair[1],
                    });
                }
            }
            clock.validate_sample(observation)?;
        }
        let observed = self
            .observations
            .last()
            .map(|item| item.position)
            .unwrap_or(0);
        if self.through_position != observed {
            return Err(HistoryError::ThroughPositionMismatch {
                declared: self.through_position,
                observed,
            });
        }
        Ok(())
    }

    fn compute_sha256(&self) -> Result<String, HistoryError> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct DigestView<'a> {
            schema_version: PositionHistorySchemaVersion,
            history_id: &'a str,
            revision: u64,
            origin_position: u64,
            through_position: u64,
            clock: &'a ClockBinding,
            observations: &'a [PositionObservation],
        }
        domain_sha256(
            "tl-mltl.position-history/v1",
            &DigestView {
                schema_version: self.schema_version,
                history_id: &self.history_id,
                revision: self.revision,
                origin_position: self.origin_position,
                through_position: self.through_position,
                clock: self.clock.as_ref().ok_or(HistoryError::MissingClock)?,
                observations: &self.observations,
            },
        )
        .map_err(|_| HistoryError::IdentityEncoding)
    }

    /// Stable history identity.
    pub fn history_id(&self) -> &str {
        &self.history_id
    }

    /// Monotonic immutable history revision.
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Digest of the complete immutable history input.
    pub fn history_sha256(&self) -> &str {
        &self.history_sha256
    }

    /// Declared final complete position.
    pub const fn through_position(&self) -> u64 {
        self.through_position
    }

    /// Exact admitted clock binding.
    pub fn clock(&self) -> Option<&ClockBinding> {
        self.clock.as_ref()
    }

    /// Gap-free observations from origin through the declared end.
    pub fn observations(&self) -> &[PositionObservation] {
        &self.observations
    }
}

fn validate_identity(value: &str) -> Result<(), HistoryError> {
    if value.is_empty() {
        return Err(HistoryError::EmptyHistoryIdentity);
    }
    if value.len() > MAX_IDENTITY_BYTES {
        return Err(HistoryError::HistoryIdentityTooLong {
            actual: value.len(),
            limit: MAX_IDENTITY_BYTES,
        });
    }
    Ok(())
}

/// Owner-produced non-values that TL preserves without Boolean coercion.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerHistoryState {
    /// Required data has not arrived.
    Incomplete,
    /// Owner cannot provide the data in this environment.
    Unavailable,
    /// Owner does not support the requested source/profile.
    Unsupported,
    /// Owner evaluation failed.
    Failed,
    /// Owner explicitly refused the input.
    Refused,
    /// Owner found conflicting values.
    Conflict,
}

/// Either a complete admitted history or an owner-produced non-value.
#[derive(Clone, Copy, Debug)]
pub enum PositionHistorySource<'a> {
    /// Complete candidate history.
    History(&'a PositionHistoryDocument),
    /// Non-value preserved exactly as supplied by the owner.
    NonValue(OwnerHistoryState),
}

impl<'a> From<&'a PositionHistoryDocument> for PositionHistorySource<'a> {
    fn from(history: &'a PositionHistoryDocument) -> Self {
        Self::History(history)
    }
}

/// Checked required-history result.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", try_from = "HistoryRequirementReportWire")]
pub struct HistoryRequirementReport {
    /// Closed report schema.
    pub schema_version: HistoryRequirementSchemaVersion,
    /// Caller-provided stable formula identity.
    pub formula_id: String,
    /// Span-independent formula digest.
    pub formula_sha256: String,
    /// Root node identity.
    pub formula_root: u32,
    /// Fixed past operator profile.
    pub operator_profile: String,
    /// Fixed semantic profile.
    pub semantic_profile: SemanticProfile,
    /// Maximum reverse offset required by the formula.
    pub required_positions: u64,
    /// Unit of `required_positions`.
    pub unit: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HistoryRequirementReportWire {
    schema_version: HistoryRequirementSchemaVersion,
    #[serde(deserialize_with = "deserialize_identity")]
    formula_id: String,
    #[serde(deserialize_with = "deserialize_digest")]
    formula_sha256: String,
    formula_root: u32,
    #[serde(deserialize_with = "deserialize_identity")]
    operator_profile: String,
    semantic_profile: SemanticProfile,
    required_positions: u64,
    #[serde(deserialize_with = "deserialize_unit")]
    unit: String,
}

/// Required-history report wire refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HistoryRequirementValidationError {
    /// A closed identity or unit is wrong.
    IdentityMismatch { field: &'static str },
    /// Formula digest is not lowercase SHA-256.
    MalformedFormulaDigest,
}

impl fmt::Display for HistoryRequirementValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "history-requirement validation failed: {self:?}")
    }
}

impl std::error::Error for HistoryRequirementValidationError {}

impl TryFrom<HistoryRequirementReportWire> for HistoryRequirementReport {
    type Error = HistoryRequirementValidationError;

    fn try_from(wire: HistoryRequirementReportWire) -> Result<Self, Self::Error> {
        let report = Self {
            schema_version: wire.schema_version,
            formula_id: wire.formula_id,
            formula_sha256: wire.formula_sha256,
            formula_root: wire.formula_root,
            operator_profile: wire.operator_profile,
            semantic_profile: wire.semantic_profile,
            required_positions: wire.required_positions,
            unit: wire.unit,
        };
        report.validate()?;
        Ok(report)
    }
}

impl HistoryRequirementReport {
    /// Revalidates the closed identities and formula digest on persisted bytes.
    pub fn validate(&self) -> Result<(), HistoryRequirementValidationError> {
        if self.formula_id.is_empty() || self.formula_id.len() > MAX_IDENTITY_BYTES {
            return Err(HistoryRequirementValidationError::IdentityMismatch { field: "formulaId" });
        }
        if !is_sha256(&self.formula_sha256) {
            return Err(HistoryRequirementValidationError::MalformedFormulaDigest);
        }
        if self.operator_profile != PAST_OPERATORS_V1 {
            return Err(HistoryRequirementValidationError::IdentityMismatch {
                field: "operatorProfile",
            });
        }
        if self.semantic_profile != SemanticProfile::OriginCompleteHistoryV1 {
            return Err(HistoryRequirementValidationError::IdentityMismatch {
                field: "semanticProfile",
            });
        }
        if self.unit != "positions" {
            return Err(HistoryRequirementValidationError::IdentityMismatch { field: "unit" });
        }
        Ok(())
    }
}

/// Required-history analysis refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HistoryRequirementError {
    /// Formula is not the origin-complete past profile.
    UnsupportedProfile {
        expected: &'static str,
        actual: &'static str,
    },
    /// A future node appeared in a past analysis.
    FutureNodeUnsupported { node: NodeId },
    /// A validated formula exposed a missing predecessor.
    InvalidNodeReference { node: NodeId },
    /// Reverse-offset arithmetic overflowed.
    ArithmeticOverflow { node: NodeId },
    /// Caller formula identity is absent or over the fixed bound.
    InvalidFormulaIdentity,
    /// Formula semantic identity could not be encoded.
    FormulaIdentityEncoding,
}

impl fmt::Display for HistoryRequirementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProfile { expected, actual } => {
                write!(
                    formatter,
                    "expected semantic profile {expected}, found {actual}"
                )
            }
            Self::FutureNodeUnsupported { node } => {
                write!(
                    formatter,
                    "future node {} is unsupported by past analysis",
                    node.0
                )
            }
            Self::InvalidNodeReference { node } => {
                write!(formatter, "invalid formula node reference {}", node.0)
            }
            Self::ArithmeticOverflow { node } => {
                write!(
                    formatter,
                    "required-history arithmetic overflowed at node {}",
                    node.0
                )
            }
            Self::InvalidFormulaIdentity => formatter.write_str("formula identity is invalid"),
            Self::FormulaIdentityEncoding => {
                formatter.write_str("formula semantic identity encoding failed")
            }
        }
    }
}

impl std::error::Error for HistoryRequirementError {}

fn formula_sha256(formula: Formula<'_>) -> Result<String, HistoryRequirementError> {
    let document = FormulaDocument::from_formula_v2(formula)
        .map_err(|_| HistoryRequirementError::FormulaIdentityEncoding)?;
    domain_sha256("tl-mltl.past-formula/v1", &document.semantic_view())
        .map_err(|_| HistoryRequirementError::FormulaIdentityEncoding)
}

fn prior_required(values: &[u64], node: NodeId) -> Result<u64, HistoryRequirementError> {
    values
        .get(
            usize::try_from(node.0)
                .map_err(|_| HistoryRequirementError::InvalidNodeReference { node })?,
        )
        .copied()
        .ok_or(HistoryRequirementError::InvalidNodeReference { node })
}

/// Computes the maximum reverse offset required by a validated past formula.
pub fn analyze_required_history(
    formula: Formula<'_>,
    formula_id: impl Into<String>,
) -> Result<HistoryRequirementReport, HistoryRequirementError> {
    if formula.profile() != SemanticProfile::OriginCompleteHistoryV1 {
        return Err(HistoryRequirementError::UnsupportedProfile {
            expected: SemanticProfile::OriginCompleteHistoryV1.as_str(),
            actual: formula.profile().as_str(),
        });
    }
    let formula_id = formula_id.into();
    if formula_id.is_empty() || formula_id.len() > MAX_IDENTITY_BYTES {
        return Err(HistoryRequirementError::InvalidFormulaIdentity);
    }
    let mut values = Vec::with_capacity(formula.nodes().len());
    for (index, node) in formula.nodes().iter().enumerate() {
        let node_id = NodeId(u32::try_from(index).map_err(|_| {
            HistoryRequirementError::InvalidNodeReference {
                node: formula.root(),
            }
        })?);
        let value = match node.kind {
            NodeKind::False | NodeKind::True | NodeKind::Proposition { .. } => 0,
            NodeKind::Not { operand } => prior_required(&values, operand)?,
            NodeKind::And { left, right }
            | NodeKind::Or { left, right }
            | NodeKind::Implies { left, right }
            | NodeKind::Equivalent { left, right } => {
                prior_required(&values, left)?.max(prior_required(&values, right)?)
            }
            NodeKind::Once { interval, operand } | NodeKind::Historically { interval, operand } => {
                u64::from(interval.end())
                    .checked_add(prior_required(&values, operand)?)
                    .ok_or(HistoryRequirementError::ArithmeticOverflow { node: node_id })?
            }
            NodeKind::StrongPrevious { operand } => 1_u64
                .checked_add(prior_required(&values, operand)?)
                .ok_or(HistoryRequirementError::ArithmeticOverflow { node: node_id })?,
            NodeKind::Since {
                interval,
                left,
                right,
            }
            | NodeKind::Triggered {
                interval,
                left,
                right,
            } => u64::from(interval.end())
                .checked_add(prior_required(&values, left)?.max(prior_required(&values, right)?))
                .ok_or(HistoryRequirementError::ArithmeticOverflow { node: node_id })?,
            NodeKind::Future { .. }
            | NodeKind::Globally { .. }
            | NodeKind::Until { .. }
            | NodeKind::Release { .. } => {
                return Err(HistoryRequirementError::FutureNodeUnsupported { node: node_id });
            }
        };
        values.push(value);
    }
    Ok(HistoryRequirementReport {
        schema_version: HistoryRequirementSchemaVersion::V1,
        formula_id,
        formula_sha256: formula_sha256(formula)?,
        formula_root: formula.root().0,
        operator_profile: PAST_OPERATORS_V1.to_owned(),
        semantic_profile: SemanticProfile::OriginCompleteHistoryV1,
        required_positions: prior_required(&values, formula.root())?,
        unit: "positions".to_owned(),
    })
}

/// Effective resource limits for one anchored past evaluation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PastEvaluationLimits {
    /// Maximum combined evaluator steps.
    pub max_steps: u64,
    /// Maximum cardinality of one temporal interval.
    pub max_temporal_span: u64,
    /// Maximum recursive node depth.
    pub max_recursion_depth: u32,
    /// Maximum admitted physical input positions.
    pub max_input_positions: usize,
}

impl Default for PastEvaluationLimits {
    fn default() -> Self {
        Self {
            max_steps: 1_000_000,
            max_temporal_span: 100_000,
            max_recursion_depth: MAX_RECURSION_DEPTH,
            max_input_positions: 100_000,
        }
    }
}

impl PastEvaluationLimits {
    fn clamped(self) -> Self {
        Self {
            max_steps: self.max_steps.min(HARD_MAX_PAST_STEPS),
            max_temporal_span: self.max_temporal_span.min(HARD_MAX_TEMPORAL_SPAN),
            max_recursion_depth: self.max_recursion_depth.min(MAX_RECURSION_DEPTH),
            max_input_positions: self.max_input_positions.min(MAX_HISTORY_POSITIONS),
        }
    }
}

/// Bounded work observations from one past evaluation.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PastEvaluationStats {
    /// Combined node and temporal-loop steps.
    pub steps: u64,
    /// Node/position evaluations.
    pub node_evaluations: u64,
    /// Temporal offsets visited.
    pub temporal_iterations: u64,
    /// Physical history positions admitted.
    pub input_positions: usize,
    /// Greatest recursive depth entered.
    pub max_recursion_depth: u32,
}

/// Immutable history attribution embedded in a result.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PositionHistoryReference {
    /// Stable history identity.
    #[serde(deserialize_with = "deserialize_identity")]
    pub history_id: String,
    /// Immutable history revision.
    pub revision: u64,
    /// Complete history digest.
    #[serde(deserialize_with = "deserialize_digest")]
    pub history_sha256: String,
    /// Origin, always zero.
    pub origin_position: u64,
    /// Final complete position.
    pub through_position: u64,
}

impl From<&PositionHistoryDocument> for PositionHistoryReference {
    fn from(history: &PositionHistoryDocument) -> Self {
        Self {
            history_id: history.history_id.clone(),
            revision: history.revision,
            history_sha256: history.history_sha256.clone(),
            origin_position: history.origin_position,
            through_position: history.through_position,
        }
    }
}

/// Direct predecessor attribution for a corrected result.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PastResultReference {
    /// Predecessor result revision.
    pub result_revision: u64,
    /// Predecessor result digest.
    #[serde(deserialize_with = "deserialize_digest")]
    pub result_sha256: String,
    /// Predecessor history identity.
    #[serde(deserialize_with = "deserialize_identity")]
    pub history_id: String,
    /// Predecessor history revision.
    pub history_revision: u64,
    /// Predecessor history digest.
    #[serde(deserialize_with = "deserialize_digest")]
    pub history_sha256: String,
    /// Predecessor evaluation anchor.
    pub anchor: u64,
}

/// Immutable result relation kind.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PastResultRelationKind {
    /// First result for this exact request identity.
    Original,
    /// Replaces a valid direct predecessor after corrected/late data.
    Superseding,
    /// Invalidates a direct predecessor proven to use invalid input.
    Invalidating,
}

/// Result correction relation and its mandatory references.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PastResultRelation {
    /// Relation kind.
    pub kind: PastResultRelationKind,
    /// Exact predecessor; absent only for originals.
    pub direct_predecessor: Option<PastResultReference>,
    /// New corrected history; absent only for originals.
    pub corrected_history: Option<PositionHistoryReference>,
}

/// Caller-selected relation carrying full predecessor bytes for validation.
#[derive(Clone, Copy, Debug)]
pub enum PastEvaluationRelationInput<'a> {
    /// No predecessor relation.
    Original,
    /// Supersede this exact prior result.
    Superseding(&'a PastEvaluationReport),
    /// Invalidate this exact prior result.
    Invalidating(&'a PastEvaluationReport),
}

/// A pure-past result is final for its exact anchor.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PastResultFinality {
    /// No future observation or physical closure can change this anchored truth.
    Final,
}

/// Complete immutable anchored past-evaluation result.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", try_from = "PastEvaluationReportWire")]
pub struct PastEvaluationReport {
    /// Closed result schema identity.
    pub schema_version: PastEvaluationSchemaVersion,
    /// Monotonic result revision within a correction chain.
    pub result_revision: u64,
    /// Digest of every result field except this digest.
    pub result_sha256: String,
    /// Caller formula identity.
    pub formula_id: String,
    /// Span-independent formula digest.
    pub formula_sha256: String,
    /// Formula root node.
    pub formula_root: u32,
    /// Closed past operator profile.
    pub operator_profile: String,
    /// Fixed past semantic profile.
    pub semantic_profile: SemanticProfile,
    /// Immutable history attribution.
    pub history: PositionHistoryReference,
    /// Explicit in-range evaluation anchor.
    pub anchor: u64,
    /// Exact independently bound clock mapping.
    pub clock: ClockBinding,
    /// Caller-owned total proposition-map identity.
    pub proposition_map_id: String,
    /// Evaluator identity.
    pub evaluator_identity: PastEvaluatorIdentity,
    /// Exact source revision compiled into the evaluator.
    pub evaluator_revision: String,
    /// Exact tl-syntax revision compiled into the evaluator.
    pub syntax_revision: String,
    /// Effective evaluation limits.
    pub limits: PastEvaluationLimits,
    /// Checked maximum reverse offset.
    pub required_history: u64,
    /// Bounded observed work.
    pub stats: PastEvaluationStats,
    /// Final two-valued result; no Pending variant exists on this wire.
    pub verdict: bool,
    /// Explicit finality.
    pub finality: PastResultFinality,
    /// Original or direct-predecessor correction relation.
    pub relation: PastResultRelation,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PastEvaluationReportWire {
    schema_version: PastEvaluationSchemaVersion,
    result_revision: u64,
    #[serde(deserialize_with = "deserialize_digest")]
    result_sha256: String,
    #[serde(deserialize_with = "deserialize_identity")]
    formula_id: String,
    #[serde(deserialize_with = "deserialize_digest")]
    formula_sha256: String,
    formula_root: u32,
    #[serde(deserialize_with = "deserialize_identity")]
    operator_profile: String,
    semantic_profile: SemanticProfile,
    history: PositionHistoryReference,
    anchor: u64,
    clock: ClockBinding,
    #[serde(deserialize_with = "deserialize_identity")]
    proposition_map_id: String,
    evaluator_identity: PastEvaluatorIdentity,
    #[serde(deserialize_with = "deserialize_identity")]
    evaluator_revision: String,
    #[serde(deserialize_with = "deserialize_identity")]
    syntax_revision: String,
    limits: PastEvaluationLimits,
    required_history: u64,
    stats: PastEvaluationStats,
    verdict: bool,
    finality: PastResultFinality,
    relation: PastResultRelation,
}

impl PastEvaluationReport {
    fn from_wire(wire: PastEvaluationReportWire) -> Self {
        Self {
            schema_version: wire.schema_version,
            result_revision: wire.result_revision,
            result_sha256: wire.result_sha256,
            formula_id: wire.formula_id,
            formula_sha256: wire.formula_sha256,
            formula_root: wire.formula_root,
            operator_profile: wire.operator_profile,
            semantic_profile: wire.semantic_profile,
            history: wire.history,
            anchor: wire.anchor,
            clock: wire.clock,
            proposition_map_id: wire.proposition_map_id,
            evaluator_identity: wire.evaluator_identity,
            evaluator_revision: wire.evaluator_revision,
            syntax_revision: wire.syntax_revision,
            limits: wire.limits,
            required_history: wire.required_history,
            stats: wire.stats,
            verdict: wire.verdict,
            finality: wire.finality,
            relation: wire.relation,
        }
    }
}

impl TryFrom<PastEvaluationReportWire> for PastEvaluationReport {
    type Error = PastResultValidationError;

    fn try_from(wire: PastEvaluationReportWire) -> Result<Self, Self::Error> {
        let report = Self::from_wire(wire);
        report.validate()?;
        Ok(report)
    }
}

/// Past-result wire or relation refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PastResultValidationError {
    /// Result revision is zero.
    ResultRevisionZero,
    /// A closed identity/profile/revision is wrong.
    IdentityMismatch { field: &'static str },
    /// A digest is not lowercase SHA-256.
    MalformedDigest { field: &'static str },
    /// Effective limits exceed hard maxima.
    LimitsNotClamped,
    /// Statistics exceed effective limits or disagree with history attribution.
    StatisticsOutOfRange,
    /// Anchor is outside the attributed history.
    AnchorOutOfRange,
    /// Relation references do not match the relation kind.
    RelationShape,
    /// Corrected-history reference is not the current history.
    CorrectedHistoryMismatch,
    /// Predecessor history/result revisions do not precede this result.
    PredecessorNotEarlier,
    /// Predecessor refers to another history or anchor.
    PredecessorContextMismatch,
    /// Supplied predecessor bytes do not match the embedded direct reference.
    PredecessorReferenceMismatch,
    /// Result names itself as predecessor.
    SelfPredecessor,
    /// Result digest is stale.
    StaleResultDigest,
    /// Canonical result serialization failed.
    IdentityEncoding,
}

impl fmt::Display for PastResultValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "past result validation failed: {self:?}")
    }
}

impl std::error::Error for PastResultValidationError {}

impl PastEvaluationReport {
    /// Revalidates every closed identity, bound, relation, and digest.
    pub fn validate(&self) -> Result<(), PastResultValidationError> {
        if self.result_revision == 0 {
            return Err(PastResultValidationError::ResultRevisionZero);
        }
        if self.operator_profile != PAST_OPERATORS_V1 {
            return Err(PastResultValidationError::IdentityMismatch {
                field: "operatorProfile",
            });
        }
        if self.semantic_profile != SemanticProfile::OriginCompleteHistoryV1 {
            return Err(PastResultValidationError::IdentityMismatch {
                field: "semanticProfile",
            });
        }
        if self.syntax_revision != TL_SYNTAX_REVISION {
            return Err(PastResultValidationError::IdentityMismatch {
                field: "syntaxRevision",
            });
        }
        for (field, identity) in [
            ("formulaId", self.formula_id.as_str()),
            ("historyId", self.history.history_id.as_str()),
            ("propositionMapId", self.proposition_map_id.as_str()),
            ("evaluatorRevision", self.evaluator_revision.as_str()),
            ("syntaxRevision", self.syntax_revision.as_str()),
        ] {
            if identity.is_empty() || identity.len() > MAX_IDENTITY_BYTES {
                return Err(PastResultValidationError::IdentityMismatch { field });
            }
        }
        for (field, digest) in [
            ("resultSha256", self.result_sha256.as_str()),
            ("formulaSha256", self.formula_sha256.as_str()),
            ("historySha256", self.history.history_sha256.as_str()),
        ] {
            if !is_sha256(digest) {
                return Err(PastResultValidationError::MalformedDigest { field });
            }
        }
        if self.history.revision == 0 {
            return Err(PastResultValidationError::IdentityMismatch {
                field: "historyRevision",
            });
        }
        self.clock
            .validate_static()
            .map_err(|_| PastResultValidationError::IdentityMismatch { field: "clock" })?;
        if self.limits != self.limits.clamped() {
            return Err(PastResultValidationError::LimitsNotClamped);
        }
        let expected_positions = self
            .history
            .through_position
            .checked_add(1)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or(PastResultValidationError::StatisticsOutOfRange)?;
        let classified_steps = self
            .stats
            .node_evaluations
            .checked_add(self.stats.temporal_iterations)
            .ok_or(PastResultValidationError::StatisticsOutOfRange)?;
        let minimum_nodes_for_depth = u64::from(self.stats.max_recursion_depth)
            .checked_add(1)
            .ok_or(PastResultValidationError::StatisticsOutOfRange)?;
        if self.stats.steps == 0
            || self.stats.node_evaluations == 0
            || self.stats.steps != classified_steps
            || self.stats.steps > self.limits.max_steps
            || self.stats.node_evaluations > self.stats.steps
            || self.stats.temporal_iterations > self.stats.steps
            || self.stats.input_positions != expected_positions
            || self.stats.input_positions > self.limits.max_input_positions
            || self.stats.max_recursion_depth > self.limits.max_recursion_depth
            || minimum_nodes_for_depth > self.stats.node_evaluations
        {
            return Err(PastResultValidationError::StatisticsOutOfRange);
        }
        if self.history.origin_position != 0 || self.anchor > self.history.through_position {
            return Err(PastResultValidationError::AnchorOutOfRange);
        }
        match self.relation.kind {
            PastResultRelationKind::Original => {
                if self.relation.direct_predecessor.is_some()
                    || self.relation.corrected_history.is_some()
                {
                    return Err(PastResultValidationError::RelationShape);
                }
            }
            PastResultRelationKind::Superseding | PastResultRelationKind::Invalidating => {
                let predecessor = self
                    .relation
                    .direct_predecessor
                    .as_ref()
                    .ok_or(PastResultValidationError::RelationShape)?;
                let corrected = self
                    .relation
                    .corrected_history
                    .as_ref()
                    .ok_or(PastResultValidationError::RelationShape)?;
                if corrected != &self.history {
                    return Err(PastResultValidationError::CorrectedHistoryMismatch);
                }
                if predecessor.result_revision >= self.result_revision
                    || predecessor.result_revision == 0
                    || predecessor.history_revision >= self.history.revision
                    || predecessor.history_revision == 0
                {
                    return Err(PastResultValidationError::PredecessorNotEarlier);
                }
                if predecessor.history_id != self.history.history_id
                    || predecessor.anchor != self.anchor
                {
                    return Err(PastResultValidationError::PredecessorContextMismatch);
                }
                if predecessor.history_id.is_empty()
                    || predecessor.history_id.len() > MAX_IDENTITY_BYTES
                {
                    return Err(PastResultValidationError::PredecessorContextMismatch);
                }
                for (field, digest) in [
                    (
                        "predecessorResultSha256",
                        predecessor.result_sha256.as_str(),
                    ),
                    (
                        "predecessorHistorySha256",
                        predecessor.history_sha256.as_str(),
                    ),
                ] {
                    if !is_sha256(digest) {
                        return Err(PastResultValidationError::MalformedDigest { field });
                    }
                }
                if predecessor.result_sha256 == self.result_sha256 {
                    return Err(PastResultValidationError::SelfPredecessor);
                }
                if predecessor.history_sha256 == self.history.history_sha256 {
                    return Err(PastResultValidationError::PredecessorNotEarlier);
                }
            }
        }
        if past_result_sha256(self)? != self.result_sha256 {
            return Err(PastResultValidationError::StaleResultDigest);
        }
        Ok(())
    }

    /// Validates a persisted correction against the exact predecessor bytes.
    ///
    /// Internal validation proves the relation's shape. This method additionally
    /// binds a non-original report to the complete predecessor supplied by the
    /// caller; originals require `None`.
    pub fn validate_with_predecessor(
        &self,
        predecessor: Option<&Self>,
    ) -> Result<(), PastResultValidationError> {
        self.validate()?;
        match self.relation.kind {
            PastResultRelationKind::Original => {
                if predecessor.is_some() {
                    return Err(PastResultValidationError::RelationShape);
                }
            }
            PastResultRelationKind::Superseding | PastResultRelationKind::Invalidating => {
                let predecessor = predecessor.ok_or(PastResultValidationError::RelationShape)?;
                predecessor.validate()?;
                if self.relation.direct_predecessor.as_ref()
                    != Some(&predecessor_reference(predecessor))
                {
                    return Err(PastResultValidationError::PredecessorReferenceMismatch);
                }
                if self.formula_id != predecessor.formula_id
                    || self.formula_sha256 != predecessor.formula_sha256
                    || self.formula_root != predecessor.formula_root
                    || self.history.history_id != predecessor.history.history_id
                    || self.anchor != predecessor.anchor
                    || self.clock != predecessor.clock
                    || self.proposition_map_id != predecessor.proposition_map_id
                    || self.evaluator_identity != predecessor.evaluator_identity
                    || self.evaluator_revision != predecessor.evaluator_revision
                    || self.syntax_revision != predecessor.syntax_revision
                    || self.limits != predecessor.limits
                {
                    return Err(PastResultValidationError::PredecessorContextMismatch);
                }
            }
        }
        Ok(())
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn past_result_sha256(report: &PastEvaluationReport) -> Result<String, PastResultValidationError> {
    let mut value =
        serde_json::to_value(report).map_err(|_| PastResultValidationError::IdentityEncoding)?;
    value
        .as_object_mut()
        .ok_or(PastResultValidationError::IdentityEncoding)?
        .remove("resultSha256");
    domain_sha256("tl-mltl.past-evaluation/v1", &value)
        .map_err(|_| PastResultValidationError::IdentityEncoding)
}

/// Anchored past-evaluation refusal; no variant carries a fallback verdict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PastEvaluationError {
    /// An owner-produced non-value was preserved.
    OwnerStatePreserved { state: OwnerHistoryState },
    /// Position history was invalid or incomplete.
    History(HistoryError),
    /// Formula is not the origin-complete past profile.
    Requirement(HistoryRequirementError),
    /// Anchor is not in the complete history.
    AnchorOutOfRange { anchor: u64, through: u64 },
    /// Physical input exceeds the caller's effective position limit.
    InputPositionLimitExceeded { requested: usize, limit: usize },
    /// One temporal interval exceeds its expansion limit.
    TemporalSpanExceeded { requested: u64, limit: u64 },
    /// Combined evaluation work exceeded its limit.
    StepLimitExceeded { limit: u64 },
    /// Recursive evaluation exceeded its depth limit.
    RecursionDepthExceeded { limit: u32 },
    /// A validated formula exposed a missing node.
    InvalidNodeReference { node: NodeId },
    /// A future node was presented to the past evaluator.
    FutureNodeUnsupported { node: NodeId },
    /// Signed reverse-position arithmetic overflowed.
    PositionArithmeticOverflow,
    /// A non-pre-origin position was absent despite history admission.
    HistoryPositionAbsent { position: u64 },
    /// Formula or proposition-map identity is invalid.
    InvalidIdentity { field: &'static str },
    /// Result revision is zero or does not exceed its direct predecessor.
    ResultRevisionInvalid,
    /// Full predecessor bytes do not validate against their digest.
    InvalidPredecessor(PastResultValidationError),
    /// Correction changes a dimension that must remain in one chain.
    CorrectionContextMismatch { field: &'static str },
    /// Correction does not advance the immutable history revision.
    HistoryRevisionNotAdvanced,
    /// Result identity could not be encoded or validated.
    Result(PastResultValidationError),
}

impl fmt::Display for PastEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "past evaluation refused: {self:?}")
    }
}

impl std::error::Error for PastEvaluationError {}

struct PastEvaluator<'formula, 'history> {
    formula: Formula<'formula>,
    history: &'history PositionHistoryDocument,
    limits: PastEvaluationLimits,
    stats: PastEvaluationStats,
}

impl PastEvaluator<'_, '_> {
    fn consume_node(&mut self, depth: u32) -> Result<(), PastEvaluationError> {
        self.stats.max_recursion_depth = self.stats.max_recursion_depth.max(depth);
        if depth > self.limits.max_recursion_depth {
            return Err(PastEvaluationError::RecursionDepthExceeded {
                limit: self.limits.max_recursion_depth,
            });
        }
        self.stats.node_evaluations = self.stats.node_evaluations.checked_add(1).ok_or(
            PastEvaluationError::StepLimitExceeded {
                limit: self.limits.max_steps,
            },
        )?;
        self.consume_step()
    }

    fn consume_temporal(&mut self) -> Result<(), PastEvaluationError> {
        self.stats.temporal_iterations = self.stats.temporal_iterations.checked_add(1).ok_or(
            PastEvaluationError::StepLimitExceeded {
                limit: self.limits.max_steps,
            },
        )?;
        self.consume_step()
    }

    fn consume_step(&mut self) -> Result<(), PastEvaluationError> {
        self.stats.steps =
            self.stats
                .steps
                .checked_add(1)
                .ok_or(PastEvaluationError::StepLimitExceeded {
                    limit: self.limits.max_steps,
                })?;
        if self.stats.steps > self.limits.max_steps {
            return Err(PastEvaluationError::StepLimitExceeded {
                limit: self.limits.max_steps,
            });
        }
        Ok(())
    }

    fn node(&self, node: NodeId) -> Result<NodeKind, PastEvaluationError> {
        let index = usize::try_from(node.0)
            .map_err(|_| PastEvaluationError::InvalidNodeReference { node })?;
        self.formula
            .nodes()
            .get(index)
            .map(|value| value.kind)
            .ok_or(PastEvaluationError::InvalidNodeReference { node })
    }

    fn endpoints(&self, start: u32, end: u32) -> Result<(u64, u64), PastEvaluationError> {
        let cardinality = u64::from(end)
            .checked_sub(u64::from(start))
            .and_then(|value| value.checked_add(1))
            .ok_or(PastEvaluationError::PositionArithmeticOverflow)?;
        if cardinality > self.limits.max_temporal_span {
            return Err(PastEvaluationError::TemporalSpanExceeded {
                requested: cardinality,
                limit: self.limits.max_temporal_span,
            });
        }
        Ok((u64::from(start), u64::from(end)))
    }

    fn at(
        &mut self,
        node: NodeId,
        position: i128,
        depth: u32,
    ) -> Result<bool, PastEvaluationError> {
        self.consume_node(depth)?;
        let child_depth =
            depth
                .checked_add(1)
                .ok_or(PastEvaluationError::RecursionDepthExceeded {
                    limit: self.limits.max_recursion_depth,
                })?;
        match self.node(node)? {
            NodeKind::False => Ok(false),
            NodeKind::True => Ok(true),
            NodeKind::Proposition { proposition } => self.proposition(position, proposition),
            NodeKind::Not { operand } => Ok(!self.at(operand, position, child_depth)?),
            NodeKind::And { left, right } => {
                Ok(self.at(left, position, child_depth)?
                    && self.at(right, position, child_depth)?)
            }
            NodeKind::Or { left, right } => {
                Ok(self.at(left, position, child_depth)?
                    || self.at(right, position, child_depth)?)
            }
            NodeKind::Implies { left, right } => {
                Ok(!self.at(left, position, child_depth)?
                    || self.at(right, position, child_depth)?)
            }
            NodeKind::Equivalent { left, right } => {
                Ok(self.at(left, position, child_depth)?
                    == self.at(right, position, child_depth)?)
            }
            NodeKind::Once { interval, operand } => {
                let (start, end) = self.endpoints(interval.start(), interval.end())?;
                for offset in start..=end {
                    self.consume_temporal()?;
                    if self.at(operand, reverse(position, offset)?, child_depth)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            NodeKind::Historically { interval, operand } => {
                let (start, end) = self.endpoints(interval.start(), interval.end())?;
                for offset in start..=end {
                    self.consume_temporal()?;
                    if !self.at(operand, reverse(position, offset)?, child_depth)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            NodeKind::StrongPrevious { operand } => {
                self.endpoints(1, 1)?;
                self.consume_temporal()?;
                self.at(operand, reverse(position, 1)?, child_depth)
            }
            NodeKind::Since {
                interval,
                left,
                right,
            } => self.since(
                position,
                interval.start(),
                interval.end(),
                left,
                right,
                false,
                child_depth,
            ),
            NodeKind::Triggered {
                interval,
                left,
                right,
            } => Ok(!self.since(
                position,
                interval.start(),
                interval.end(),
                left,
                right,
                true,
                child_depth,
            )?),
            NodeKind::Future { .. }
            | NodeKind::Globally { .. }
            | NodeKind::Until { .. }
            | NodeKind::Release { .. } => Err(PastEvaluationError::FutureNodeUnsupported { node }),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn since(
        &mut self,
        position: i128,
        start: u32,
        end: u32,
        left: NodeId,
        right: NodeId,
        negate_operands: bool,
        child_depth: u32,
    ) -> Result<bool, PastEvaluationError> {
        let (start, end) = self.endpoints(start, end)?;
        for witness in start..=end {
            self.consume_temporal()?;
            let mut candidate = self.at(right, reverse(position, witness)?, child_depth)?;
            if negate_operands {
                candidate = !candidate;
            }
            for offset in start..witness {
                self.consume_temporal()?;
                if !candidate {
                    break;
                }
                let mut value = self.at(left, reverse(position, offset)?, child_depth)?;
                if negate_operands {
                    value = !value;
                }
                candidate &= value;
            }
            if candidate {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn proposition(
        &self,
        position: i128,
        proposition: PropositionId,
    ) -> Result<bool, PastEvaluationError> {
        if position < 0 {
            return Ok(false);
        }
        let position =
            u64::try_from(position).map_err(|_| PastEvaluationError::PositionArithmeticOverflow)?;
        let index = usize::try_from(position)
            .map_err(|_| PastEvaluationError::HistoryPositionAbsent { position })?;
        let observation = self
            .history
            .observations
            .get(index)
            .ok_or(PastEvaluationError::HistoryPositionAbsent { position })?;
        Ok(observation
            .true_propositions
            .binary_search(&proposition)
            .is_ok())
    }
}

fn reverse(position: i128, offset: u64) -> Result<i128, PastEvaluationError> {
    position
        .checked_sub(i128::from(offset))
        .ok_or(PastEvaluationError::PositionArithmeticOverflow)
}

fn validate_result_identity(value: &str, field: &'static str) -> Result<(), PastEvaluationError> {
    if value.is_empty() || value.len() > MAX_IDENTITY_BYTES {
        Err(PastEvaluationError::InvalidIdentity { field })
    } else {
        Ok(())
    }
}

fn predecessor_reference(report: &PastEvaluationReport) -> PastResultReference {
    PastResultReference {
        result_revision: report.result_revision,
        result_sha256: report.result_sha256.clone(),
        history_id: report.history.history_id.clone(),
        history_revision: report.history.revision,
        history_sha256: report.history.history_sha256.clone(),
        anchor: report.anchor,
    }
}

struct CorrectionContext<'a> {
    formula_id: &'a str,
    formula_sha256: &'a str,
    formula_root: u32,
    proposition_map_id: &'a str,
    limits: PastEvaluationLimits,
    anchor: u64,
}

fn correction_relation(
    input: PastEvaluationRelationInput<'_>,
    history: &PositionHistoryDocument,
    result_revision: u64,
    context: &CorrectionContext<'_>,
) -> Result<PastResultRelation, PastEvaluationError> {
    let (kind, predecessor) = match input {
        PastEvaluationRelationInput::Original => {
            return Ok(PastResultRelation {
                kind: PastResultRelationKind::Original,
                direct_predecessor: None,
                corrected_history: None,
            });
        }
        PastEvaluationRelationInput::Superseding(report) => {
            (PastResultRelationKind::Superseding, report)
        }
        PastEvaluationRelationInput::Invalidating(report) => {
            (PastResultRelationKind::Invalidating, report)
        }
    };
    predecessor
        .validate()
        .map_err(PastEvaluationError::InvalidPredecessor)?;
    if result_revision <= predecessor.result_revision {
        return Err(PastEvaluationError::ResultRevisionInvalid);
    }
    if history.history_id != predecessor.history.history_id {
        return Err(PastEvaluationError::CorrectionContextMismatch { field: "historyId" });
    }
    if history.revision <= predecessor.history.revision {
        return Err(PastEvaluationError::HistoryRevisionNotAdvanced);
    }
    for (matches, field) in [
        (context.formula_id == predecessor.formula_id, "formulaId"),
        (
            context.formula_sha256 == predecessor.formula_sha256,
            "formulaSha256",
        ),
        (
            context.formula_root == predecessor.formula_root,
            "formulaRoot",
        ),
        (context.anchor == predecessor.anchor, "anchor"),
        (history.clock.as_ref() == Some(&predecessor.clock), "clock"),
        (
            context.proposition_map_id == predecessor.proposition_map_id,
            "propositionMapId",
        ),
        (context.limits == predecessor.limits, "limits"),
        (
            predecessor.evaluator_identity == PastEvaluatorIdentity::V1,
            "evaluatorIdentity",
        ),
        (
            predecessor.evaluator_revision == env!("TL_MLTL_SOURCE_REVISION"),
            "evaluatorRevision",
        ),
        (
            predecessor.syntax_revision == TL_SYNTAX_REVISION,
            "syntaxRevision",
        ),
    ] {
        if !matches {
            return Err(PastEvaluationError::CorrectionContextMismatch { field });
        }
    }
    Ok(PastResultRelation {
        kind,
        direct_predecessor: Some(predecessor_reference(predecessor)),
        corrected_history: Some(PositionHistoryReference::from(history)),
    })
}

/// Evaluates one validated past formula at an explicit complete-history anchor.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_past<'history>(
    formula: Formula<'_>,
    formula_id: impl Into<String>,
    history_source: impl Into<PositionHistorySource<'history>>,
    anchor: u64,
    proposition_map_id: impl Into<String>,
    result_revision: u64,
    relation_input: PastEvaluationRelationInput<'_>,
    limits: PastEvaluationLimits,
) -> Result<PastEvaluationReport, PastEvaluationError> {
    evaluate_past_with_stats(
        formula,
        formula_id,
        history_source,
        anchor,
        proposition_map_id,
        result_revision,
        relation_input,
        limits,
    )
    .0
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn evaluate_past_with_stats<'history>(
    formula: Formula<'_>,
    formula_id: impl Into<String>,
    history_source: impl Into<PositionHistorySource<'history>>,
    anchor: u64,
    proposition_map_id: impl Into<String>,
    result_revision: u64,
    relation_input: PastEvaluationRelationInput<'_>,
    limits: PastEvaluationLimits,
) -> (
    Result<PastEvaluationReport, PastEvaluationError>,
    PastEvaluationStats,
) {
    let mut stats = PastEvaluationStats::default();
    let result = evaluate_past_inner(
        formula,
        formula_id,
        history_source,
        anchor,
        proposition_map_id,
        result_revision,
        relation_input,
        limits,
        &mut stats,
    );
    (result, stats)
}

#[allow(clippy::too_many_arguments)]
fn evaluate_past_inner<'history>(
    formula: Formula<'_>,
    formula_id: impl Into<String>,
    history_source: impl Into<PositionHistorySource<'history>>,
    anchor: u64,
    proposition_map_id: impl Into<String>,
    result_revision: u64,
    relation_input: PastEvaluationRelationInput<'_>,
    limits: PastEvaluationLimits,
    observed: &mut PastEvaluationStats,
) -> Result<PastEvaluationReport, PastEvaluationError> {
    let history = match history_source.into() {
        PositionHistorySource::History(history) => history,
        PositionHistorySource::NonValue(state) => {
            return Err(PastEvaluationError::OwnerStatePreserved { state });
        }
    };
    history.validate().map_err(PastEvaluationError::History)?;
    observed.input_positions = history.observations.len();
    if anchor > history.through_position {
        return Err(PastEvaluationError::AnchorOutOfRange {
            anchor,
            through: history.through_position,
        });
    }
    if result_revision == 0 {
        return Err(PastEvaluationError::ResultRevisionInvalid);
    }
    let limits = limits.clamped();
    if history.observations.len() > limits.max_input_positions {
        return Err(PastEvaluationError::InputPositionLimitExceeded {
            requested: history.observations.len(),
            limit: limits.max_input_positions,
        });
    }
    let formula_id = formula_id.into();
    let proposition_map_id = proposition_map_id.into();
    validate_result_identity(&formula_id, "formulaId")?;
    validate_result_identity(&proposition_map_id, "propositionMapId")?;
    let requirement = analyze_required_history(formula, formula_id.clone())
        .map_err(PastEvaluationError::Requirement)?;
    let formula_sha256 = requirement.formula_sha256.clone();
    let relation = correction_relation(
        relation_input,
        history,
        result_revision,
        &CorrectionContext {
            formula_id: &formula_id,
            formula_sha256: &formula_sha256,
            formula_root: formula.root().0,
            proposition_map_id: &proposition_map_id,
            limits,
            anchor,
        },
    )?;
    let input_positions = history.observations.len();
    let mut evaluator = PastEvaluator {
        formula,
        history,
        limits,
        stats: PastEvaluationStats {
            input_positions,
            ..PastEvaluationStats::default()
        },
    };
    let verdict = match evaluator.at(formula.root(), i128::from(anchor), 0) {
        Ok(verdict) => verdict,
        Err(error) => {
            *observed = evaluator.stats;
            return Err(error);
        }
    };
    *observed = evaluator.stats;
    let mut report = PastEvaluationReport {
        schema_version: PastEvaluationSchemaVersion::V1,
        result_revision,
        result_sha256: String::new(),
        formula_id,
        formula_sha256,
        formula_root: formula.root().0,
        operator_profile: PAST_OPERATORS_V1.to_owned(),
        semantic_profile: SemanticProfile::OriginCompleteHistoryV1,
        history: PositionHistoryReference::from(history),
        anchor,
        clock: history
            .clock
            .clone()
            .ok_or(PastEvaluationError::History(HistoryError::MissingClock))?,
        proposition_map_id,
        evaluator_identity: PastEvaluatorIdentity::V1,
        evaluator_revision: env!("TL_MLTL_SOURCE_REVISION").to_owned(),
        syntax_revision: TL_SYNTAX_REVISION.to_owned(),
        limits,
        required_history: requirement.required_positions,
        stats: *observed,
        verdict,
        finality: PastResultFinality::Final,
        relation,
    };
    report.result_sha256 = past_result_sha256(&report).map_err(PastEvaluationError::Result)?;
    report.validate().map_err(PastEvaluationError::Result)?;
    Ok(report)
}
