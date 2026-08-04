//! Compatibility projection for canonical simple-import planning.
//!
//! The planner owns collection, resolution, dependency edges, and cycle
//! rejection. This module preserves the earlier binding-only return shape
//! without creating an independent resolution path.

use ash_parser::CanonicalModuleGraph;

use crate::{CanonicalBoundModuleSet, CanonicalModuleBindError, resolve_simple_parsed_imports};

/// Resolves simple parsed imports through the canonical planner and projects
/// the successful plan into the legacy binding-only result shape.
///
/// # Errors
///
/// Returns [`CanonicalModuleBindError`] when planning rejects an unsupported,
/// unresolved, inaccessible, duplicate, or cyclic import set.
#[allow(
    clippy::result_large_err,
    reason = "the public diagnostic contract exposes its anchored fields without boxing"
)]
pub fn bind_simple_parsed_uses(
    graph: &CanonicalModuleGraph,
) -> Result<CanonicalBoundModuleSet, CanonicalModuleBindError> {
    resolve_simple_parsed_imports(graph).map(|plan| plan.into_bound_set())
}
