use ash_parser::CanonicalModuleGraph;

use crate::{
    CanonicalBoundModuleSet, CanonicalProvisionalModuleScopes, CanonicalStructuralImportError,
    resolve_scoped_glob_local_precedence_imports_with_scopes,
    resolve_scoped_glob_ordinary_function_imports_with_scopes,
    resolve_scoped_grouped_ordinary_function_imports_with_scopes,
    resolve_scoped_simple_local_precedence_imports_with_scopes,
    resolve_scoped_simple_ordinary_function_imports_with_scopes,
    resolve_scoped_super_grouped_ordinary_function_imports_with_scopes,
    resolve_scoped_super_ordinary_function_imports_with_scopes,
    resolve_simple_parsed_imports_with_scopes,
};

/// Projects successful scoped structural aliases into binding-only facts.
///
/// This admits only inherited explicit `use crate::<child>...::<function> as
/// <alias>` declarations accepted by the scoped resolver. The result contains
/// bindings only; path resolution, visibility, and cycle checks remain owned
/// by that resolver. Its [`CanonicalStructuralImportError`] is propagated
/// unchanged.
pub fn bind_scoped_structural_parsed_uses(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<CanonicalBoundModuleSet, CanonicalStructuralImportError> {
    resolve_simple_parsed_imports_with_scopes(graph, scopes).map(|plan| plan.into_bound_set())
}

/// Projects scoped simple ordinary-function imports into binding-only facts.
///
/// This admits inherited `use crate::<function>` imports at the crate root or
/// through direct structural children, with an optional explicit alias. An
/// omitted alias binds the final function segment. The scoped resolver owns
/// path, visibility, collision, and cycle checks; its
/// [`CanonicalStructuralImportError`] is propagated unchanged.
pub fn bind_scoped_simple_ordinary_function_imports(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<CanonicalBoundModuleSet, CanonicalStructuralImportError> {
    resolve_scoped_simple_ordinary_function_imports_with_scopes(graph, scopes)
        .map(|plan| plan.into_bound_set())
}

/// Projects simple imports after same-module local-name precedence.
///
/// This admits only the dedicated inherited, unaliased simple route. Its
/// resolver retains candidate edges through cycle detection and removes a
/// natural-name binding only when an importer-local ordinary function wins.
///
/// # Errors
///
/// Returns [`CanonicalStructuralImportError`] unchanged when the dedicated
/// resolver rejects the supplied graph or scope facts.
pub fn bind_scoped_simple_local_precedence_imports(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<CanonicalBoundModuleSet, CanonicalStructuralImportError> {
    resolve_scoped_simple_local_precedence_imports_with_scopes(graph, scopes)
        .map(|plan| plan.into_bound_set())
}

/// Projects scoped `super` ordinary-function imports into binding-only facts.
///
/// This admits only inherited `use super::<function>` imports from non-root
/// modules, optionally through structural children and with an optional local
/// alias. The scoped resolver owns path, visibility, collision, and cycle
/// checks; its [`CanonicalStructuralImportError`] is propagated unchanged.
///
/// # Errors
///
/// Returns [`CanonicalStructuralImportError`] unchanged when the scoped-super
/// resolver rejects the supplied graph or scope facts.
pub fn bind_scoped_super_ordinary_function_imports(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<CanonicalBoundModuleSet, CanonicalStructuralImportError> {
    resolve_scoped_super_ordinary_function_imports_with_scopes(graph, scopes)
        .map(|plan| plan.into_bound_set())
}

/// Projects scoped grouped `super` ordinary-function imports into bindings.
///
/// This admits only inherited non-root `use super::<children>::{function,
/// function as local}` declarations accepted by the scoped-super grouped
/// resolver. The resolver owns member spans, path, visibility, collision, and
/// cycle checks; its [`CanonicalStructuralImportError`] is propagated
/// unchanged.
///
/// # Errors
///
/// Returns [`CanonicalStructuralImportError`] unchanged when the scoped-super
/// grouped resolver rejects the supplied graph or scope facts.
pub fn bind_scoped_super_grouped_ordinary_function_imports(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<CanonicalBoundModuleSet, CanonicalStructuralImportError> {
    resolve_scoped_super_grouped_ordinary_function_imports_with_scopes(graph, scopes)
        .map(|plan| plan.into_bound_set())
}

/// Projects scoped grouped ordinary-function imports into binding-only facts.
///
/// This admits only inherited `use crate::<children>::{function, function as
/// local}` declarations accepted by the scoped grouped resolver. The resolver
/// retains parser member spans and owns path, visibility, collision, and cycle
/// checks; its [`CanonicalStructuralImportError`] is propagated unchanged.
///
/// # Errors
///
/// Returns [`CanonicalStructuralImportError`] unchanged when the scoped
/// grouped resolver rejects the supplied graph or scope facts.
pub fn bind_scoped_grouped_ordinary_function_imports(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<CanonicalBoundModuleSet, CanonicalStructuralImportError> {
    resolve_scoped_grouped_ordinary_function_imports_with_scopes(graph, scopes)
        .map(|plan| plan.into_bound_set())
}

/// Projects scoped glob ordinary-function imports into binding-only facts.
///
/// This admits only inherited `use crate::<public-child>...::*` declarations
/// accepted by the scoped glob resolver. The resolver retains full-use-span
/// provenance and owns all boundary, path, visibility, collision, and cycle
/// checks; this projection introduces no precedence or wider binder authority.
///
/// # Errors
///
/// Returns [`CanonicalStructuralImportError`] unchanged when the scoped glob
/// resolver rejects the supplied graph or scope facts.
pub fn bind_scoped_glob_ordinary_function_imports(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<CanonicalBoundModuleSet, CanonicalStructuralImportError> {
    resolve_scoped_glob_ordinary_function_imports_with_scopes(graph, scopes)
        .map(|plan| plan.into_bound_set())
}

/// Projects scoped glob imports with same-module local precedence.
///
/// This admits only the dedicated inherited `use crate::<public-child>...::*`
/// route. The resolver retains every selected edge through cycle detection,
/// then applies local-over-glob precedence to its returned binding projection.
///
/// # Errors
///
/// Returns [`CanonicalStructuralImportError`] unchanged when the dedicated
/// resolver rejects the supplied graph or scope facts.
pub fn bind_scoped_glob_local_precedence_imports(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<CanonicalBoundModuleSet, CanonicalStructuralImportError> {
    resolve_scoped_glob_local_precedence_imports_with_scopes(graph, scopes)
        .map(|plan| plan.into_bound_set())
}
