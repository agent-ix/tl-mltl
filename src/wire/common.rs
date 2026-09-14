//! Shared bounded canonical-document machinery for tl-mltl owner contracts.

use std::{fmt, io};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

/// Immutable maxima enforced by every tl-mltl owner reader and producer.
pub const OWNER_MAX: OwnerLimits = OwnerLimits {
    max_input_bytes: 8 * 1024 * 1024,
    max_output_bytes: 8 * 1024 * 1024,
    max_depth: 64,
    max_string_bytes: 1024 * 1024,
    max_formula_nodes: 100_000,
    max_formula_depth: 4_096,
    max_positions: 1_000_000,
    max_propositions: 100_000,
    max_support: 100_000,
    max_history_span: 100_000,
    max_evaluation_steps: 4_000_000,
    max_recursion_depth: crate::MAX_RECURSION_DEPTH,
    max_visited_fields: 1_000_000,
};

/// Caller-lowerable ceilings for all temporal wire operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OwnerLimits {
    /// Maximum accepted input bytes.
    pub max_input_bytes: usize,
    /// Maximum emitted canonical bytes.
    pub max_output_bytes: usize,
    /// Maximum JSON container depth.
    pub max_depth: usize,
    /// Maximum encoded bytes in one JSON string.
    pub max_string_bytes: usize,
    /// Maximum formula nodes.
    pub max_formula_nodes: usize,
    /// Maximum root-to-leaf formula graph depth.
    pub max_formula_depth: usize,
    /// Maximum trace or history positions.
    pub max_positions: usize,
    /// Maximum propositions in a valuation population.
    pub max_propositions: usize,
    /// Maximum decision-support identities.
    pub max_support: usize,
    /// Maximum required history span.
    pub max_history_span: u64,
    /// Maximum evaluator steps.
    pub max_evaluation_steps: u64,
    /// Maximum evaluator recursion depth.
    pub max_recursion_depth: u32,
    /// Maximum object members and array elements visited while reading.
    pub max_visited_fields: usize,
}

impl OwnerLimits {
    /// Returns immutable owner maxima.
    #[must_use]
    pub const fn owner_max() -> Self {
        OWNER_MAX
    }

    /// Clamps every caller ceiling to the owner maximum.
    #[must_use]
    pub const fn effective(self) -> Self {
        Self {
            max_input_bytes: min_usize(self.max_input_bytes, OWNER_MAX.max_input_bytes),
            max_output_bytes: min_usize(self.max_output_bytes, OWNER_MAX.max_output_bytes),
            max_depth: min_usize(self.max_depth, OWNER_MAX.max_depth),
            max_string_bytes: min_usize(self.max_string_bytes, OWNER_MAX.max_string_bytes),
            max_formula_nodes: min_usize(self.max_formula_nodes, OWNER_MAX.max_formula_nodes),
            max_formula_depth: min_usize(self.max_formula_depth, OWNER_MAX.max_formula_depth),
            max_positions: min_usize(self.max_positions, OWNER_MAX.max_positions),
            max_propositions: min_usize(self.max_propositions, OWNER_MAX.max_propositions),
            max_support: min_usize(self.max_support, OWNER_MAX.max_support),
            max_history_span: min_u64(self.max_history_span, OWNER_MAX.max_history_span),
            max_evaluation_steps: min_u64(
                self.max_evaluation_steps,
                OWNER_MAX.max_evaluation_steps,
            ),
            max_recursion_depth: min_u32(self.max_recursion_depth, OWNER_MAX.max_recursion_depth),
            max_visited_fields: min_usize(self.max_visited_fields, OWNER_MAX.max_visited_fields),
        }
    }
}

const fn min_usize(left: usize, right: usize) -> usize {
    if left < right {
        left
    } else {
        right
    }
}

const fn min_u64(left: u64, right: u64) -> u64 {
    if left < right {
        left
    } else {
        right
    }
}

const fn min_u32(left: u32, right: u32) -> u32 {
    if left < right {
        left
    } else {
        right
    }
}

impl Default for OwnerLimits {
    fn default() -> Self {
        OWNER_MAX
    }
}

/// Bounded work observed while producing or reading an owner document.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnerUsage {
    /// Canonical wire bytes consumed or emitted.
    pub wire_bytes: usize,
    /// Deepest JSON container visited.
    pub depth: usize,
    /// Largest encoded JSON string visited.
    pub string_bytes: usize,
    /// Formula nodes admitted.
    pub formula_nodes: usize,
    /// Root-to-leaf formula graph depth admitted by the syntax owner.
    pub formula_depth: usize,
    /// Trace or history positions admitted.
    pub positions: usize,
    /// Proposition entries admitted.
    pub propositions: usize,
    /// Decision-support identities admitted.
    pub support: usize,
    /// Required history span.
    pub history_span: u64,
    /// Evaluator steps performed.
    pub evaluation_steps: u64,
    /// Deepest evaluator recursion entered.
    pub recursion_depth: u32,
    /// Object members and array elements visited.
    pub visited_fields: usize,
}

/// Stable machine-readable temporal owner refusal codes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerReadErrorCode {
    /// Input is not UTF-8.
    InvalidUtf8,
    /// Input is not exactly one valid JSON document.
    InvalidJson,
    /// Input is valid but not the one canonical encoding.
    NonCanonical,
    /// A resource ceiling was exceeded.
    ResourceIncomplete,
    /// The selected contract or schema is wrong.
    ContractMismatch,
    /// An independently supplied owner input does not match.
    ExpectedMismatch,
    /// A document identity or digest is stale.
    IdentityMismatch,
    /// A profile, topology, state, or lineage combination is invalid.
    InvalidCombination,
    /// Canonical encoding failed.
    Encoding,
    /// Evaluation refused the selected input.
    EvaluationRefused,
}

impl OwnerReadErrorCode {
    /// Stable wire label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidUtf8 => "TL-OWNER-INVALID-UTF8",
            Self::InvalidJson => "TL-OWNER-INVALID-JSON",
            Self::NonCanonical => "TL-OWNER-NONCANONICAL",
            Self::ResourceIncomplete => "TL-OWNER-RESOURCE-INCOMPLETE",
            Self::ContractMismatch => "TL-OWNER-CONTRACT-MISMATCH",
            Self::ExpectedMismatch => "TL-OWNER-EXPECTED-MISMATCH",
            Self::IdentityMismatch => "TL-OWNER-IDENTITY-MISMATCH",
            Self::InvalidCombination => "TL-OWNER-INVALID-COMBINATION",
            Self::Encoding => "TL-OWNER-ENCODING",
            Self::EvaluationRefused => "TL-OWNER-EVALUATION-REFUSED",
        }
    }
}

/// Typed fail-closed owner boundary error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnerReadError {
    code: OwnerReadErrorCode,
    field: &'static str,
    usage: OwnerUsage,
}

impl OwnerReadError {
    pub(crate) const fn new(
        code: OwnerReadErrorCode,
        field: &'static str,
        usage: OwnerUsage,
    ) -> Self {
        Self { code, field, usage }
    }

    /// Stable refusal code.
    #[must_use]
    pub const fn code(&self) -> OwnerReadErrorCode {
        self.code
    }

    /// Bounded field/category identifying the refusal locus.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    /// Work observed before refusal.
    #[must_use]
    pub const fn usage(&self) -> OwnerUsage {
        self.usage
    }
}

impl fmt::Display for OwnerReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.field)
    }
}

impl std::error::Error for OwnerReadError {}

/// Canonical immutable owner bytes produced by tl-mltl.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnerDocument<T> {
    value: T,
    bytes: Vec<u8>,
    usage: OwnerUsage,
}

impl<T> OwnerDocument<T> {
    /// Typed snapshot represented by the canonical bytes.
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Exact canonical bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Bounded work retained by production.
    #[must_use]
    pub const fn usage(&self) -> OwnerUsage {
        self.usage
    }

    pub(crate) fn into_parts(self) -> (T, Vec<u8>, OwnerUsage) {
        (self.value, self.bytes, self.usage)
    }
}

pub(crate) fn produce<T: Serialize + Clone>(
    value: T,
    semantic_usage: OwnerUsage,
    limits: OwnerLimits,
) -> Result<OwnerDocument<T>, OwnerReadError> {
    let effective = limits.effective();
    validate_semantic_usage(semantic_usage, effective)?;
    let mut writer = BoundedWriter::new(effective.max_output_bytes);
    if serde_json::to_writer(&mut writer, &value).is_err() {
        if let Some(attempted) = writer.rejected_size {
            return Err(OwnerReadError::new(
                OwnerReadErrorCode::ResourceIncomplete,
                "outputBytes",
                OwnerUsage {
                    wire_bytes: attempted,
                    ..semantic_usage
                },
            ));
        }
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::Encoding,
            "document",
            semantic_usage,
        ));
    }
    let bytes = writer.bytes;
    let lexical = preflight(&bytes, effective, false)?;
    Ok(OwnerDocument {
        value,
        bytes,
        usage: merge_usage(semantic_usage, lexical),
    })
}

struct BoundedWriter {
    bytes: Vec<u8>,
    limit: usize,
    rejected_size: Option<usize>,
}

impl BoundedWriter {
    fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(limit.min(4_096)),
            limit,
            rejected_size: None,
        }
    }
}

impl io::Write for BoundedWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let attempted = self.bytes.len().saturating_add(buffer.len());
        if attempted > self.limit {
            self.rejected_size = Some(attempted);
            return Err(io::Error::other("owner output byte limit exceeded"));
        }
        self.bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(crate) fn read<T, F>(
    bytes: &[u8],
    limits: OwnerLimits,
    validate: F,
) -> Result<(T, OwnerUsage), OwnerReadError>
where
    T: DeserializeOwned + Serialize,
    F: FnOnce(&T, OwnerLimits) -> Result<OwnerUsage, OwnerReadError>,
{
    let effective = limits.effective();
    let lexical = preflight(bytes, effective, true)?;
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = T::deserialize(&mut deserializer)
        .map_err(|_| OwnerReadError::new(OwnerReadErrorCode::InvalidJson, "document", lexical))?;
    deserializer.end().map_err(|_| {
        OwnerReadError::new(OwnerReadErrorCode::InvalidJson, "trailingData", lexical)
    })?;
    let semantic = validate(&value, effective)?;
    let canonical = serde_json::to_vec(&value)
        .map_err(|_| OwnerReadError::new(OwnerReadErrorCode::Encoding, "document", lexical))?;
    if canonical != bytes {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::NonCanonical,
            "document",
            merge_usage(semantic, lexical),
        ));
    }
    Ok((value, merge_usage(semantic, lexical)))
}

pub(crate) fn read_expected<T, F>(
    bytes: &[u8],
    expected: &T,
    limits: OwnerLimits,
    validate: F,
) -> Result<(T, OwnerUsage), OwnerReadError>
where
    T: DeserializeOwned + Serialize + Eq,
    F: FnOnce(&T, OwnerLimits) -> Result<OwnerUsage, OwnerReadError>,
{
    let (value, usage) = read(bytes, limits, validate)?;
    if &value != expected {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::ExpectedMismatch,
            "expected",
            usage,
        ));
    }
    Ok((value, usage))
}

pub(crate) fn identity<T: Serialize>(
    contract: &str,
    value: &T,
    identity_field: &str,
) -> Result<String, OwnerReadError> {
    let bytes = serde_json::to_vec(value).map_err(|_| {
        OwnerReadError::new(
            OwnerReadErrorCode::Encoding,
            "identityPreimage",
            OwnerUsage::default(),
        )
    })?;
    let bytes = omit_top_level_field(&bytes, identity_field)?;
    let mut digest = Sha256::new();
    digest.update(contract.as_bytes());
    digest.update([0]);
    digest.update(bytes);
    Ok(hex(digest.finalize()))
}

fn omit_top_level_field(bytes: &[u8], omitted: &str) -> Result<Vec<u8>, OwnerReadError> {
    if bytes.first() != Some(&b'{') || bytes.last() != Some(&b'}') {
        return Err(encoding_error("identityPreimage"));
    }
    let mut member_start = 1usize;
    while member_start < bytes.len() - 1 {
        if bytes.get(member_start) != Some(&b'"') {
            return Err(encoding_error("identityPreimage"));
        }
        let key_end = scan_string(bytes, member_start)?;
        if bytes.get(key_end + 1) != Some(&b':') {
            return Err(encoding_error("identityPreimage"));
        }
        let value_end = scan_value(bytes, key_end + 2)?;
        if bytes.get(member_start + 1..key_end) == Some(omitted.as_bytes()) {
            let (remove_start, remove_end) = if member_start == 1 {
                let end = if bytes.get(value_end) == Some(&b',') {
                    value_end + 1
                } else {
                    value_end
                };
                (member_start, end)
            } else {
                (member_start - 1, value_end)
            };
            let mut preimage = Vec::with_capacity(bytes.len() - (remove_end - remove_start));
            preimage.extend_from_slice(&bytes[..remove_start]);
            preimage.extend_from_slice(&bytes[remove_end..]);
            return Ok(preimage);
        }
        if bytes.get(value_end) == Some(&b',') {
            member_start = value_end + 1;
        } else {
            break;
        }
    }
    Err(encoding_error("identityField"))
}

fn scan_string(bytes: &[u8], start: usize) -> Result<usize, OwnerReadError> {
    let mut index = start + 1;
    let mut escaped = false;
    while let Some(byte) = bytes.get(index).copied() {
        match (byte, escaped) {
            (_, true) => escaped = false,
            (b'\\', false) => escaped = true,
            (b'"', false) => return Ok(index),
            _ => {}
        }
        index += 1;
    }
    Err(encoding_error("identityPreimage"))
}

fn scan_value(bytes: &[u8], start: usize) -> Result<usize, OwnerReadError> {
    let mut index = start;
    let mut depth = 0usize;
    while let Some(byte) = bytes.get(index).copied() {
        match byte {
            b'"' => index = scan_string(bytes, index)?,
            b'{' | b'[' => {
                depth = depth
                    .checked_add(1)
                    .ok_or_else(|| encoding_error("identityPreimage"))?;
            }
            b'}' | b']' if depth > 0 => depth -= 1,
            b',' | b'}' if depth == 0 => return Ok(index),
            _ => {}
        }
        index += 1;
    }
    Err(encoding_error("identityPreimage"))
}

fn encoding_error(field: &'static str) -> OwnerReadError {
    OwnerReadError::new(OwnerReadErrorCode::Encoding, field, OwnerUsage::default())
}

pub(crate) fn raw_sha256(bytes: &[u8]) -> String {
    hex(Sha256::digest(bytes))
}

pub(crate) fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn validate_semantic_usage(usage: OwnerUsage, limits: OwnerLimits) -> Result<(), OwnerReadError> {
    let exceeded = if usage.formula_nodes > limits.max_formula_nodes {
        Some("formulaNodes")
    } else if usage.formula_depth > limits.max_formula_depth {
        Some("formulaDepth")
    } else if usage.positions > limits.max_positions {
        Some("positions")
    } else if usage.propositions > limits.max_propositions {
        Some("propositions")
    } else if usage.support > limits.max_support {
        Some("support")
    } else if usage.history_span > limits.max_history_span {
        Some("historySpan")
    } else if usage.evaluation_steps > limits.max_evaluation_steps {
        Some("evaluationSteps")
    } else if usage.recursion_depth > limits.max_recursion_depth {
        Some("recursionDepth")
    } else {
        None
    };
    if let Some(field) = exceeded {
        Err(OwnerReadError::new(
            OwnerReadErrorCode::ResourceIncomplete,
            field,
            usage,
        ))
    } else {
        Ok(())
    }
}

fn merge_usage(semantic: OwnerUsage, lexical: OwnerUsage) -> OwnerUsage {
    OwnerUsage {
        wire_bytes: lexical.wire_bytes.max(semantic.wire_bytes),
        depth: lexical.depth.max(semantic.depth),
        string_bytes: lexical.string_bytes.max(semantic.string_bytes),
        formula_nodes: semantic.formula_nodes,
        formula_depth: semantic.formula_depth,
        positions: semantic.positions,
        propositions: semantic.propositions,
        support: semantic.support,
        history_span: semantic.history_span,
        evaluation_steps: semantic.evaluation_steps,
        recursion_depth: semantic.recursion_depth,
        visited_fields: lexical.visited_fields.max(semantic.visited_fields),
    }
}

#[derive(Clone, Copy)]
enum Container {
    Object,
    Array { expects_value: bool },
}

fn preflight(bytes: &[u8], limits: OwnerLimits, input: bool) -> Result<OwnerUsage, OwnerReadError> {
    let byte_limit = if input {
        limits.max_input_bytes
    } else {
        limits.max_output_bytes
    };
    let mut usage = OwnerUsage {
        wire_bytes: bytes.len(),
        ..OwnerUsage::default()
    };
    if bytes.len() > byte_limit {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::ResourceIncomplete,
            if input { "inputBytes" } else { "outputBytes" },
            usage,
        ));
    }
    std::str::from_utf8(bytes)
        .map_err(|_| OwnerReadError::new(OwnerReadErrorCode::InvalidUtf8, "document", usage))?;

    let mut stack = Vec::new();
    let mut index = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte.is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if !matches!(byte, b',' | b']') {
            count_array_value(&mut stack, &mut usage);
        }
        match byte {
            b'{' => stack.push(Container::Object),
            b'[' => stack.push(Container::Array {
                expects_value: true,
            }),
            b'}' | b']' => {
                stack.pop();
            }
            b':' => {
                usage.visited_fields = usage.visited_fields.saturating_add(1);
            }
            b',' => {
                if let Some(Container::Array { expects_value }) = stack.last_mut() {
                    *expects_value = true;
                }
            }
            b'"' => {
                let start = index + 1;
                index += 1;
                let mut escaped = false;
                while index < bytes.len() {
                    match (bytes[index], escaped) {
                        (_, true) => escaped = false,
                        (b'\\', false) => escaped = true,
                        (b'"', false) => break,
                        _ => {}
                    }
                    index += 1;
                }
                let encoded = index.saturating_sub(start);
                usage.string_bytes = usage.string_bytes.max(encoded);
            }
            _ => {}
        }
        usage.depth = usage.depth.max(stack.len());
        if usage.depth > limits.max_depth {
            return Err(OwnerReadError::new(
                OwnerReadErrorCode::ResourceIncomplete,
                "depth",
                usage,
            ));
        }
        if usage.string_bytes > limits.max_string_bytes {
            return Err(OwnerReadError::new(
                OwnerReadErrorCode::ResourceIncomplete,
                "stringBytes",
                usage,
            ));
        }
        if usage.visited_fields > limits.max_visited_fields {
            return Err(OwnerReadError::new(
                OwnerReadErrorCode::ResourceIncomplete,
                "visitedFields",
                usage,
            ));
        }
        index += 1;
    }
    Ok(usage)
}

fn count_array_value(stack: &mut [Container], usage: &mut OwnerUsage) {
    let Some(Container::Array { expects_value }) = stack.last_mut() else {
        return;
    };
    if *expects_value {
        usage.visited_fields = usage.visited_fields.saturating_add(1);
        *expects_value = false;
    }
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    let bytes = bytes.as_ref();
    let mut result = String::with_capacity(bytes.len().saturating_mul(2));
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        result.push(char::from(HEX[usize::from(byte >> 4)]));
        result.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    result
}
