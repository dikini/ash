//! Structural computation-row facts used by source handler checking.
//!
//! This module intentionally walks parsed [`ComputationRow`] values.  It does
//! not recover rows from formatted text, and its facts are requirements only:
//! normalizing a row never installs a provider, capability, or handler frame.

use std::collections::BTreeMap;
use std::fmt;

use ash_core::semantic_summary::{
    EffectRowClosureStatus, ModuleSemanticSummary, StructuralEffectRowItemSummary, SummaryVersion,
};
use ash_parser::surface::{ComputationRow, ComputationRowItem, Definition, Program};

use crate::{TypeCheckError, TypeEnv, standalone_program_module_identity};

/// A source anchor plus the alias/group expansion path that reached it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandlerRowProvenance {
    expansion_path: Vec<String>,
    source_span: ash_parser::token::Span,
}

impl HandlerRowProvenance {
    fn new(expansion_path: &[String], source_span: ash_parser::token::Span) -> Self {
        Self {
            expansion_path: expansion_path.to_vec(),
            source_span,
        }
    }

    /// A stable, source-visible expansion path for diagnostics.
    #[must_use]
    pub fn expansion_path(&self) -> String {
        self.expansion_path.join(" -> ")
    }

    /// The source item anchor retained independently of normalized identity.
    #[must_use]
    pub const fn source_span(&self) -> ash_parser::token::Span {
        self.source_span
    }
}

/// One canonical, non-granting handler-computation row item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedHandlerRowItem {
    canonical_key: String,
    provenance: Vec<HandlerRowProvenance>,
}

impl NormalizedHandlerRowItem {
    /// Canonical semantic identity, suitable for deterministic comparisons.
    #[must_use]
    pub fn canonical_key(&self) -> String {
        self.canonical_key.clone()
    }

    /// Every source occurrence contributing this compatible item.
    #[must_use]
    pub fn source_provenance(&self) -> &[HandlerRowProvenance] {
        &self.provenance
    }

    /// Rows describe requirements; they never grant authority.
    #[must_use]
    pub const fn grants_authority(&self) -> bool {
        false
    }
}

/// An immutable normalized computation row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedHandlerRow {
    /// Canonical items in deterministic semantic-family order.
    pub items: Vec<NormalizedHandlerRowItem>,
    /// The single retained open tail, if present.
    pub tail: Option<String>,
    tail_provenances: Vec<HandlerRowProvenance>,
}

impl NormalizedHandlerRow {
    /// The source provenance of the retained open tail.
    #[must_use]
    pub fn tail_provenance(&self) -> Option<&HandlerRowProvenance> {
        self.tail_provenances.first()
    }

    /// Every compatible source occurrence of the retained open tail, in
    /// structural traversal order.
    #[must_use]
    pub fn tail_provenances(&self) -> &[HandlerRowProvenance] {
        &self.tail_provenances
    }
}

/// Build one normalized declared-operation row item without granting any
/// authority.  This is kept crate-visible because checked computation
/// inference must retain the same row/provenance representation as row
/// annotations rather than reconstructing it from formatted text.
pub(crate) fn normalized_declared_operation(
    declared: &crate::DeclaredConcreteOperation,
    source_span: ash_parser::token::Span,
) -> NormalizedHandlerRow {
    NormalizedHandlerRow {
        items: vec![NormalizedHandlerRowItem {
            canonical_key: format!(
                "operation:{}::{}::{}",
                declared.impl_type, declared.interface, declared.operation
            ),
            provenance: vec![HandlerRowProvenance::new(&[], source_span)],
        }],
        tail: None,
        tail_provenances: Vec::new(),
    }
}

/// Build the open residual row retained by a synthesized source handler.
///
/// This remains structural source-type evidence: it does not represent a
/// Core row variable or install any runtime handler state.
pub(crate) fn normalized_open_handler_row_tail(
    tail: &str,
    source_span: ash_parser::token::Span,
) -> NormalizedHandlerRow {
    NormalizedHandlerRow {
        items: Vec::new(),
        tail: Some(tail.to_string()),
        tail_provenances: vec![HandlerRowProvenance::new(&[], source_span)],
    }
}

/// Structurally merge normalized requirement rows. Compatible entries retain
/// every source occurrence; distinct open tails are rejected rather than
/// silently choosing one.
pub(crate) fn union_normalized_handler_rows(
    rows: &[NormalizedHandlerRow],
) -> NormalizeResult<NormalizedHandlerRow> {
    let mut accumulator = RowAccumulator::default();
    for row in rows {
        for item in &row.items {
            for provenance in &item.provenance {
                let already_present = accumulator
                    .items
                    .get(&item.canonical_key)
                    .is_some_and(|existing| existing.provenance.contains(provenance));
                if !already_present {
                    accumulator.add_item(item.canonical_key.clone(), provenance.clone());
                }
            }
        }
        if let Some(tail) = &row.tail {
            for provenance in &row.tail_provenances {
                let already_present =
                    accumulator
                        .tail
                        .as_ref()
                        .is_some_and(|(existing_tail, provenances)| {
                            existing_tail == tail && provenances.contains(provenance)
                        });
                if !already_present {
                    accumulator.add_tail(tail, provenance.clone())?;
                }
            }
        }
    }
    Ok(accumulator.finish())
}

/// Compare row requirements by their canonical identities and open tail only.
/// Source provenance is intentionally excluded: an annotation and a direct
/// operation expression can describe the same row from different anchors.
pub(crate) fn normalized_handler_rows_semantically_equal(
    left: &NormalizedHandlerRow,
    right: &NormalizedHandlerRow,
) -> bool {
    left.tail == right.tail
        && left.items.len() == right.items.len()
        && left
            .items
            .iter()
            .zip(&right.items)
            .all(|(left_item, right_item)| left_item.canonical_key == right_item.canonical_key)
}

/// Remove only the exact canonical operations handled by a declaration.
/// All non-operation requirements and any open tail are retained verbatim.
pub(crate) fn subtract_handled_operations(
    row: &NormalizedHandlerRow,
    handled_operation_keys: &[String],
) -> Result<NormalizedHandlerRow, HandlerRowNormalizationError> {
    let mut remaining = row.clone();
    let mut seen = std::collections::BTreeSet::new();
    for key in handled_operation_keys {
        if !seen.insert(key) {
            return Err(HandlerRowNormalizationError(format!(
                "duplicate handler operation clause for {key}"
            )));
        }
        let Some(index) = remaining
            .items
            .iter()
            .position(|item| item.canonical_key == *key)
        else {
            return Err(HandlerRowNormalizationError(format!(
                "handler clause operation '{key}' is absent from the handled computation row"
            )));
        };
        remaining.items.remove(index);
    }
    Ok(remaining)
}

/// Whether a normalized residual has no requirements and no open tail.
pub(crate) fn is_closed_empty_row(row: &NormalizedHandlerRow) -> bool {
    row.items.is_empty() && row.tail.is_none()
}

/// Fail-closed row-normalization error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandlerRowNormalizationError(String);

impl fmt::Display for HandlerRowNormalizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for HandlerRowNormalizationError {}

impl From<TypeCheckError> for HandlerRowNormalizationError {
    fn from(error: TypeCheckError) -> Self {
        Self(error.to_string())
    }
}

type NormalizeResult<T> = Result<T, HandlerRowNormalizationError>;

#[derive(Default)]
struct RowAccumulator {
    items: BTreeMap<String, NormalizedHandlerRowItem>,
    tail: Option<(String, Vec<HandlerRowProvenance>)>,
}

impl RowAccumulator {
    fn add_item(&mut self, canonical_key: String, provenance: HandlerRowProvenance) {
        match self.items.get_mut(&canonical_key) {
            Some(existing) => existing.provenance.push(provenance),
            None => {
                self.items.insert(
                    canonical_key.clone(),
                    NormalizedHandlerRowItem {
                        canonical_key,
                        provenance: vec![provenance],
                    },
                );
            }
        }
    }

    fn add_tail(&mut self, tail: &str, provenance: HandlerRowProvenance) -> NormalizeResult<()> {
        match self.tail.as_mut() {
            None => self.tail = Some((tail.to_string(), vec![provenance])),
            Some((existing, provenances)) if existing == tail => {
                provenances.push(provenance);
            }
            Some((existing, _)) => {
                return Err(HandlerRowNormalizationError(format!(
                    "conflicting handler-computation row tails: {existing} vs {tail}"
                )));
            }
        }
        Ok(())
    }

    fn finish(self) -> NormalizedHandlerRow {
        let mut items = self.items.into_values().collect::<Vec<_>>();
        items.sort_by(|left, right| {
            item_family_rank(&left.canonical_key)
                .cmp(&item_family_rank(&right.canonical_key))
                .then_with(|| left.canonical_key.cmp(&right.canonical_key))
        });
        let (tail, tail_provenances) =
            self.tail.map_or((None, Vec::new()), |(tail, provenance)| {
                (Some(tail), provenance)
            });
        NormalizedHandlerRow {
            items,
            tail,
            tail_provenances,
        }
    }
}

fn item_family_rank(key: &str) -> u8 {
    [
        "operation:",
        "resource:",
        "role:",
        "policy:",
        "channel:",
        "process:",
        "fail:",
        "evidence:",
    ]
    .iter()
    .position(|prefix| key.starts_with(prefix))
    .map_or(u8::MAX, |index| index as u8)
}

struct Normalizer<'a> {
    env: &'a TypeEnv,
    local_rows: BTreeMap<String, &'a ComputationRow>,
    imported_rows: BTreeMap<
        String,
        (
            SummaryVersion,
            &'a ash_core::semantic_summary::EffectRowExportSummary,
        ),
    >,
    expansion_stack: Vec<String>,
    accumulator: RowAccumulator,
}

impl<'a> Normalizer<'a> {
    fn new(
        env: &'a TypeEnv,
        program: &'a Program,
        imported_summaries: &'a [ModuleSemanticSummary],
    ) -> Self {
        let local_rows = program
            .definitions
            .iter()
            .filter_map(|definition| match definition {
                Definition::EffectAlias(alias) => Some((alias.name.to_string(), &alias.row)),
                Definition::EffectGroup(group) => Some((group.name.to_string(), &group.row)),
                _ => None,
            })
            .collect();
        let imported_rows = imported_summaries
            .iter()
            .flat_map(|summary| {
                summary
                    .exported_effect_rows
                    .iter()
                    .map(move |row| (row.exported_name.to_string(), (summary.version, row)))
            })
            .collect();
        Self {
            env,
            local_rows,
            imported_rows,
            expansion_stack: Vec::new(),
            accumulator: RowAccumulator::default(),
        }
    }

    fn normalize(mut self, row: &ComputationRow) -> NormalizeResult<NormalizedHandlerRow> {
        // The supplied root can itself be a named alias/group.  Include that
        // declaration in the stack so a cycle is rendered from the caller's
        // root (`A -> B -> A`), not from an arbitrary later hop.
        let root_name = self
            .local_rows
            .iter()
            .find_map(|(name, candidate)| std::ptr::eq(*candidate, row).then(|| name.clone()));
        if let Some(root_name) = root_name {
            self.expansion_stack.push(root_name);
            let result = self.walk_row(row);
            self.expansion_stack.pop();
            result?;
        } else {
            self.walk_row(row)?;
        }
        Ok(self.accumulator.finish())
    }

    fn walk_row(&mut self, row: &ComputationRow) -> NormalizeResult<()> {
        for item in &row.items {
            self.walk_item(item)?;
        }
        Ok(())
    }

    fn walk_item(&mut self, item: &ComputationRowItem) -> NormalizeResult<()> {
        let provenance = HandlerRowProvenance::new(&self.expansion_stack, item_span(item));
        match item {
            ComputationRowItem::Operation {
                path, separator, ..
            } => {
                if path.len() == 1 && separator.is_none() {
                    return self.expand_named(path[0].as_ref(), item_span(item));
                }
                let [impl_type, operation] = path.as_slice() else {
                    return Err(HandlerRowNormalizationError(
                        "malformed handler-computation operation row identity".to_string(),
                    ));
                };
                if *separator != Some(ash_parser::surface::RowPathSeparator::DoubleColon) {
                    return Err(HandlerRowNormalizationError(
                        "handler-computation operation rows require declared concrete 'Impl::operation' identity".to_string(),
                    ));
                }
                let declared = self
                    .env
                    .resolve_declared_concrete_operation(impl_type, operation)
                    .map_err(HandlerRowNormalizationError)?;
                self.accumulator.add_item(
                    format!(
                        "operation:{}::{}::{}",
                        declared.impl_type, declared.interface, declared.operation
                    ),
                    provenance,
                );
            }
            ComputationRowItem::WholeRow { variable, .. } => {
                self.expand_named(variable, item_span(item))?
            }
            ComputationRowItem::Group { path, .. } => {
                let [name] = path.as_slice() else {
                    return Err(HandlerRowNormalizationError(
                        "malformed handler-computation row group reference".to_string(),
                    ));
                };
                self.expand_named(name, item_span(item))?
            }
            ComputationRowItem::Resource { path, mode, .. } => self.accumulator.add_item(
                match mode {
                    Some(mode) => format!("resource:{mode}:{}", path_text(path)),
                    None => format!("resource:{}", path_text(path)),
                },
                provenance,
            ),
            ComputationRowItem::Role { path, .. } => self
                .accumulator
                .add_item(format!("role:{}", path_text(path)), provenance),
            ComputationRowItem::Policy { path, .. } => self
                .accumulator
                .add_item(format!("policy:{}", path_text(path)), provenance),
            ComputationRowItem::Channel { path, mode, .. } => self.accumulator.add_item(
                match mode {
                    Some(mode) => format!("channel:{mode}:{}", path_text(path)),
                    None => format!("channel:{}", path_text(path)),
                },
                provenance,
            ),
            ComputationRowItem::Process { operation, .. } => self.accumulator.add_item(
                operation.as_ref().map_or_else(
                    || "process".to_string(),
                    |operation| format!("process:{operation}"),
                ),
                provenance,
            ),
            ComputationRowItem::Fail { path, .. } => self.accumulator.add_item(
                path.as_ref().map_or_else(
                    || "fail".to_string(),
                    |path| format!("fail:{}", path_text(path)),
                ),
                provenance,
            ),
            ComputationRowItem::Evidence { path, .. } => self
                .accumulator
                .add_item(format!("evidence:{}", path_text(path)), provenance),
            ComputationRowItem::Tail { variable, .. } => {
                self.accumulator.add_tail(variable, provenance)?
            }
        }
        Ok(())
    }

    fn expand_named(
        &mut self,
        name: &str,
        imported_use_span: ash_parser::token::Span,
    ) -> NormalizeResult<()> {
        if let Some(cycle_start) = self.expansion_stack.iter().position(|entry| entry == name) {
            let mut cycle = self.expansion_stack[cycle_start..].to_vec();
            cycle.push(name.to_string());
            return Err(HandlerRowNormalizationError(format!(
                "cyclic handler-computation row expansion: {}",
                cycle.join(" -> ")
            )));
        }

        if let Some(row) = self.local_rows.get(name).copied() {
            self.expansion_stack.push(name.to_string());
            let result = self.walk_row(row);
            self.expansion_stack.pop();
            return result;
        }

        if let Some((summary_version, row)) = self.imported_rows.get(name).copied() {
            if matches!(
                row.binding.closure_status,
                EffectRowClosureStatus::OpaqueInaccessibleDependency(_)
            ) {
                return Err(HandlerRowNormalizationError(
                    "malformed imported-effect-row-summary: provider-binding effect-row closure is inaccessible at public boundary".to_string(),
                ));
            }
            // V7 remains decodable for compatibility, but its text-only rows
            // cannot produce typed handler facts. Never parse
            // `row_items[*].text` here.
            if summary_version == SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7 {
                return Err(HandlerRowNormalizationError(
                    "malformed imported-effect-row-summary: legacy V7 provider/binding row is ineligible for typed-handler normalization; require V8 structural content".to_string(),
                ));
            }

            if summary_version != SummaryVersion::STRUCTURAL_EFFECT_ROW_PROVIDER_BINDINGS_V8
                || row.row_items.iter().any(|item| item.structural().is_none())
            {
                return Err(HandlerRowNormalizationError(
                    "malformed imported-effect-row-summary: structural effect-row payload is unavailable"
                        .to_string(),
                ));
            }

            self.expansion_stack.push(name.to_string());
            let result = row.row_items.iter().try_for_each(|item| {
                let Some(structural) = item.structural() else {
                    unreachable!("legacy items returned above");
                };
                match structural {
                    StructuralEffectRowItemSummary::Operation {
                        impl_type,
                        interface,
                        operation,
                    } => {
                        let declared = self
                            .env
                            .resolve_declared_concrete_operation(impl_type, operation)
                            .map_err(HandlerRowNormalizationError)?;
                        if declared.interface != *interface {
                            return Err(HandlerRowNormalizationError(
                                "malformed imported-effect-row-summary: structural operation identity disagrees with the declared concrete operation".to_string(),
                            ));
                        }
                        self.accumulator.add_item(
                            format!(
                                "operation:{}::{}::{}",
                                declared.impl_type, declared.interface, declared.operation
                            ),
                            HandlerRowProvenance::new(
                                &self.expansion_stack,
                                imported_use_span,
                            ),
                        );
                    }
                    StructuralEffectRowItemSummary::Evidence { path } => {
                        self.accumulator.add_item(
                            format!("evidence:{}", path.join(".")),
                            HandlerRowProvenance::new(
                                &self.expansion_stack,
                                imported_use_span,
                            ),
                        );
                    }
                    StructuralEffectRowItemSummary::Tail { variable } => self
                        .accumulator
                        .add_tail(
                            variable,
                            HandlerRowProvenance::new(
                                &self.expansion_stack,
                                imported_use_span,
                            ),
                        )?,
                    StructuralEffectRowItemSummary::NamedRow { name } => {
                        self.expand_named(name, imported_use_span)?;
                    }
                    StructuralEffectRowItemSummary::Resource { path, mode } => self
                        .accumulator
                        .add_item(
                            mode.as_ref().map_or_else(
                                || format!("resource:{}", path.join(".")),
                                |mode| format!("resource:{mode}:{}", path.join(".")),
                            ),
                            HandlerRowProvenance::new(
                                &self.expansion_stack,
                                imported_use_span,
                            ),
                        ),
                    StructuralEffectRowItemSummary::Role { path } => self
                        .accumulator
                        .add_item(
                            format!("role:{}", path.join(".")),
                            HandlerRowProvenance::new(
                                &self.expansion_stack,
                                imported_use_span,
                            ),
                        ),
                    StructuralEffectRowItemSummary::Policy { path } => self
                        .accumulator
                        .add_item(
                            format!("policy:{}", path.join(".")),
                            HandlerRowProvenance::new(
                                &self.expansion_stack,
                                imported_use_span,
                            ),
                        ),
                    StructuralEffectRowItemSummary::Channel { path, mode } => self
                        .accumulator
                        .add_item(
                            mode.as_ref().map_or_else(
                                || format!("channel:{}", path.join(".")),
                                |mode| format!("channel:{mode}:{}", path.join(".")),
                            ),
                            HandlerRowProvenance::new(
                                &self.expansion_stack,
                                imported_use_span,
                            ),
                        ),
                    StructuralEffectRowItemSummary::Process { keyword, operation } => self
                        .accumulator
                        .add_item(
                            operation.as_ref().map_or_else(
                                || keyword.clone(),
                                |operation| format!("{keyword}:{operation}"),
                            ),
                            HandlerRowProvenance::new(
                                &self.expansion_stack,
                                imported_use_span,
                            ),
                        ),
                    StructuralEffectRowItemSummary::Fail { path } => self
                        .accumulator
                        .add_item(
                            path.as_ref().map_or_else(
                                || "fail".to_string(),
                                |path| format!("fail:{}", path.join(".")),
                            ),
                            HandlerRowProvenance::new(
                                &self.expansion_stack,
                                imported_use_span,
                            ),
                        ),
                }
                Ok(())
            });
            self.expansion_stack.pop();
            return result;
        }

        Err(HandlerRowNormalizationError(format!(
            "unknown handler-computation row '{name}'"
        )))
    }
}

fn path_text(path: &[ash_parser::surface::Name]) -> String {
    path.iter().map(AsRef::as_ref).collect::<Vec<_>>().join(".")
}

fn item_span(item: &ComputationRowItem) -> ash_parser::token::Span {
    match item {
        ComputationRowItem::Operation { span, .. }
        | ComputationRowItem::WholeRow { span, .. }
        | ComputationRowItem::Resource { span, .. }
        | ComputationRowItem::Role { span, .. }
        | ComputationRowItem::Policy { span, .. }
        | ComputationRowItem::Channel { span, .. }
        | ComputationRowItem::Process { span, .. }
        | ComputationRowItem::Fail { span, .. }
        | ComputationRowItem::Evidence { span, .. }
        | ComputationRowItem::Group { span, .. }
        | ComputationRowItem::Tail { span, .. } => *span,
    }
}

pub(crate) fn row_normalization_env(
    program: &Program,
    imported_summaries: &[ModuleSemanticSummary],
) -> NormalizeResult<TypeEnv> {
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(standalone_program_module_identity());
    env.register_surface_declarations(&program.definitions)
        .map_err(|error| HandlerRowNormalizationError(error.to_string()))?;
    for definition in &program.definitions {
        if let Definition::Interface(interface) = definition {
            env.register_interface(interface)
                .map_err(|error| HandlerRowNormalizationError(error.to_string()))?;
        }
    }
    for definition in &program.definitions {
        if let Definition::Type(ty) = definition
            && !env.has_type(ty.name.as_ref())
        {
            env.register_type(&ash_parser::lower_surface_type_def(ty))
                .map_err(|error| HandlerRowNormalizationError(error.to_string()))?;
        }
    }
    for definition in &program.definitions {
        if let Definition::Impl(implementation) = definition {
            env.register_impl(implementation)
                .map_err(|error| HandlerRowNormalizationError(error.to_string()))?;
        }
    }
    // Registering imports is transactional and validates the V7 boundary.
    // Opaque rows are intentionally diagnosed later without exposing names.
    let transparent = imported_summaries
        .iter()
        .filter(|summary| {
            summary.exported_effect_rows.iter().all(|row| {
                !matches!(
                    row.binding.closure_status,
                    EffectRowClosureStatus::OpaqueInaccessibleDependency(_)
                )
            })
        })
        .cloned()
        .collect::<Vec<_>>();
    if !transparent.is_empty() {
        env.register_module_semantic_summaries(&transparent)
            .map_err(|error| HandlerRowNormalizationError(error.to_string()))?;
    }
    Ok(env)
}

pub(crate) fn normalize_handler_row_in_env(
    env: &TypeEnv,
    program: &Program,
    row: &ComputationRow,
) -> NormalizeResult<NormalizedHandlerRow> {
    Normalizer::new(env, program, &[]).normalize(row)
}

/// Normalize one local source row for the TASK-2013 test seam.
#[doc(hidden)]
pub fn normalize_handler_row_for_test(
    program: &Program,
    row: &ComputationRow,
) -> Result<NormalizedHandlerRow, HandlerRowNormalizationError> {
    normalize_handler_row_with_imported_summaries_for_test(program, row, &[])
}

/// Normalize one source row with read-only imported summary facts for tests.
#[doc(hidden)]
pub fn normalize_handler_row_with_imported_summaries_for_test(
    program: &Program,
    row: &ComputationRow,
    imported_summaries: &[ModuleSemanticSummary],
) -> Result<NormalizedHandlerRow, HandlerRowNormalizationError> {
    let env = row_normalization_env(program, imported_summaries)?;
    Normalizer::new(&env, program, imported_summaries).normalize(row)
}
