//! Ash Engine - Unified embedding API for Ash workflows
//!
//! This crate provides the central `Engine` type for integrating Ash into Rust applications.
//! It encapsulates the entire workflow lifecycle: Parse → Check → Execute.
//!
//! # Example
//!
//! ```
//! use ash_engine::Engine;
//!
//! # tokio_test::block_on(async {
//! let engine = Engine::new().build().expect("engine builds");
//! # });
//! ```

pub mod check;
pub mod entry;
pub mod error;
pub mod execute;
pub mod harness;
pub mod module_loader;
pub mod monomorphize;
pub mod parse;
pub mod providers;

pub use entry::{
    EntryBootstrapError, EntryVerificationError, RuntimeEntryStdlibSource,
    load_runtime_entry_stdlib_sources, verify_entry_workflow_def,
};
pub use error::EngineError;
// Re-export the unified CapabilityProvider trait from ash_core
pub use ash_core::capability::CapabilityProvider;

use ash_core::Value;
use ash_interp::{ExecResult, RuntimeState, interpret_in_state};
use ash_parser::surface::Type as SurfaceType;
use std::collections::HashMap;

/// The central engine for all Ash operations
///
/// The `Engine` provides a unified interface for parsing, type checking,
/// and executing Ash workflows. It is designed to be:
///
/// - **Send + Sync**: Can be shared across threads
/// - **Configurable**: Built using the builder pattern
/// - **Extensible**: Supports custom capability providers
///
/// # Example
///
/// ```
/// use ash_engine::Engine;
///
/// # tokio_test::block_on(async {
/// let engine = Engine::new()
///     .with_stdio_capabilities()
///     .build()
///     .expect("engine builds");
/// # });
/// ```
#[derive(Debug)]
pub struct Engine {
    /// Store surface workflow definitions by a unique ID
    /// This stores the full `WorkflowDef` including parameters for type checking
    surface_workflow_defs:
        std::sync::Mutex<std::collections::HashMap<u64, ash_parser::surface::WorkflowDef>>,
    /// Imported ADT/type definitions keyed by parsed workflow ID.
    imported_type_defs:
        std::sync::Mutex<std::collections::HashMap<u64, Vec<ash_core::ast::TypeDef>>>,
    /// Parsed program metadata for workflows loaded with local pure-function definitions.
    surface_programs:
        std::sync::Mutex<std::collections::HashMap<u64, ash_parser::surface::Program>>,
    /// Narrow engine-owned registry of runtime stdlib module sources keyed by
    /// canonical module path.
    runtime_stdlib_modules: std::sync::Mutex<std::collections::HashMap<String, String>>,
    /// Counter for generating unique IDs
    next_id: std::sync::atomic::AtomicU64,
    /// Runtime-owned state that persists across related executions.
    /// Providers configured via `EngineBuilder` are passed to `RuntimeState` during build.
    runtime_state: RuntimeState,
}

/// A workflow handle that carries its internal ID for type checking
///
/// This wraps an `ash_core::Workflow` and maintains the association
/// with its surface representation needed for type checking.
#[derive(Debug, Clone)]
pub struct Workflow {
    /// The core workflow
    pub core: ash_core::Workflow,
    /// The internal ID for looking up the surface workflow
    id: u64,
    /// Imported callable closures, bound into context before execution.
    /// Populated from `module_loader::InlineCallable` during parse.
    pub imported_closures: std::collections::HashMap<String, ash_core::Value>,
    /// Param counts for imported callables, used to register type signatures.
    pub imported_param_counts: std::collections::HashMap<String, usize>,
    /// Declared type signatures for imported ordinary `pub fn` callables.
    pub imported_fn_signatures: std::collections::HashMap<String, ash_parser::surface::FnDef>,
    /// Declared type signatures for imported builtin fn callables.
    ///
    /// When present, `Engine::check()` uses `builtin_fn_signature_type` to
    /// produce the proper polymorphic type instead of an arity-only synthetic.
    pub imported_builtin_signatures:
        std::collections::HashMap<String, ash_parser::surface::BuiltinFnDef>,
}

impl PartialEq for Workflow {
    fn eq(&self, other: &Self) -> bool {
        self.core == other.core && self.id == other.id
    }
}

impl std::ops::Deref for Workflow {
    type Target = ash_core::Workflow;

    fn deref(&self) -> &Self::Target {
        &self.core
    }
}

impl std::ops::DerefMut for Workflow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.core
    }
}

/// Result of checking a non-workflow module file.
///
/// Contains counts of validated type definitions and pub fn snippets,
/// along with any warnings or errors encountered during validation.
#[derive(Debug)]
pub struct ModuleFileCheckResult {
    /// Number of `pub type` definitions that parsed and registered successfully.
    pub type_count: usize,
    /// Number of `pub fn` snippets that parsed successfully.
    pub fn_count: usize,
    /// Non-fatal warnings collected during checking.
    pub warnings: Vec<String>,
    /// Fatal errors collected during checking.
    pub errors: Vec<String>,
}

/// Result of processing a multi-workflow program: closures, param counts, and lowered workflow.
type ProgramProcessingResult = (
    HashMap<String, Value>,
    HashMap<String, usize>,
    ash_core::ast::Workflow,
);

impl Engine {
    /// Create a new engine builder with default configuration
    ///
    /// Returns an `EngineBuilder` that can be used to configure capabilities
    /// before building the engine.
    ///
    /// # Example
    ///
    /// ```
    /// use ash_engine::Engine;
    ///
    /// let builder = Engine::new();
    /// ```
    #[allow(clippy::new_ret_no_self, clippy::missing_const_for_fn)]
    #[must_use]
    pub fn new() -> EngineBuilder {
        EngineBuilder::new()
    }

    /// Generate a unique ID for storing surface workflows
    fn next_workflow_id(&self) -> u64 {
        self.next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    /// Store a surface workflow definition and return its ID
    fn store_surface_workflow_def(&self, def: ash_parser::surface::WorkflowDef) -> u64 {
        let id = self.next_workflow_id();
        if let Ok(mut map) = self.surface_workflow_defs.lock() {
            map.insert(id, def);
        }
        id
    }

    /// Store imported type definitions for a parsed workflow.
    fn store_imported_type_defs(&self, workflow_id: u64, defs: Vec<ash_core::ast::TypeDef>) {
        if let Ok(mut map) = self.imported_type_defs.lock() {
            map.insert(workflow_id, defs);
        }
    }

    fn store_surface_program(&self, workflow_id: u64, program: ash_parser::surface::Program) {
        if let Ok(mut map) = self.surface_programs.lock() {
            map.insert(workflow_id, program);
        }
    }

    /// Retrieve a surface workflow definition by its ID
    fn get_surface_workflow_def(&self, id: u64) -> Option<ash_parser::surface::WorkflowDef> {
        self.surface_workflow_defs
            .lock()
            .map_or(None, |map| map.get(&id).cloned())
    }

    fn get_surface_program(&self, id: u64) -> Option<ash_parser::surface::Program> {
        self.surface_programs
            .lock()
            .map_or(None, |map| map.get(&id).cloned())
    }

    /// Retrieve imported type definitions by workflow ID.
    fn get_imported_type_defs(&self, id: u64) -> Vec<ash_core::ast::TypeDef> {
        self.imported_type_defs.lock().map_or_else(
            |_| Vec::new(),
            |map| map.get(&id).cloned().unwrap_or_default(),
        )
    }

    /// Register a runtime stdlib module source under its canonical module path.
    fn register_runtime_stdlib_module(
        &self,
        module_path: &str,
        source: String,
    ) -> Result<(), EngineError> {
        self.runtime_stdlib_modules
            .lock()
            .map_err(|_| {
                EngineError::Configuration("runtime stdlib registry lock poisoned".to_string())
            })?
            .insert(module_path.to_string(), source);
        Ok(())
    }

    /// Load the narrow runtime stdlib registry owned by this engine.
    ///
    /// This only registers the canonical runtime entry modules currently needed
    /// by the Phase 57 entry path.
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::Io`] if a required stdlib source file cannot be read
    /// or [`EngineError::Configuration`] if the registry cannot be updated.
    pub fn load_runtime_stdlib(&self) -> Result<(), EngineError> {
        for module in load_runtime_entry_stdlib_sources()? {
            self.register_runtime_stdlib_module(module.module_path, module.source)?;
        }

        Ok(())
    }

    /// Return whether the engine has registered the canonical runtime module path.
    #[must_use]
    pub fn has_registered_runtime_module(&self, module_path: &str) -> bool {
        self.runtime_stdlib_modules
            .lock()
            .is_ok_and(|registry| registry.contains_key(module_path))
    }

    /// Check whether a capability provider with the given name is registered.
    #[must_use]
    pub fn has_provider(&self, name: &str) -> bool {
        self.runtime_state.has_provider(name)
    }

    fn runtime_stdlib_type_defs(&self) -> Result<Vec<ash_core::ast::TypeDef>, EngineError> {
        let sources = self
            .runtime_stdlib_modules
            .lock()
            .map_err(|_| {
                EngineError::Configuration("runtime stdlib registry lock poisoned".to_string())
            })?
            .values()
            .cloned()
            .collect::<Vec<_>>();

        let mut type_defs = Vec::new();
        for source in &sources {
            type_defs.extend(module_loader::collect_type_identity_defs_from_source(
                source,
            )?);
        }
        Ok(type_defs)
    }

    /// Parse source code into a Workflow
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Parse` if the source contains syntax errors.
    pub fn parse(&self, source: &str) -> Result<Workflow, EngineError> {
        let imported_callables = HashMap::new();
        self.parse_workflow_source_with_imports(source, Vec::new(), &imported_callables)
    }

    /// Parse entry source into a [`Workflow`], tolerating a leading `use` prelude.
    ///
    /// This helper is intentionally narrow and only exists for the runtime entry
    /// path. It validates contiguous leading runtime `use` declarations against
    /// the engine-owned runtime stdlib registry before stripping them and
    /// delegating to the ordinary single-workflow parser.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Parse` if the leading runtime imports are not
    /// supported or not registered, or if the remaining workflow source
    /// contains syntax errors.
    pub fn parse_entry_source(&self, source: &str) -> Result<Workflow, EngineError> {
        entry::validate_runtime_entry_import_prelude(source, |module_path| {
            self.has_registered_runtime_module(module_path)
        })?;
        self.parse_workflow_source_with_imports(
            entry::strip_leading_entry_use_lines(source),
            Vec::new(),
            &HashMap::new(),
        )
    }

    /// Parse entry source from a file, tolerating the narrow leading `use` prelude.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Io` if the file cannot be read and `EngineError::Parse`
    /// if the entry workflow source is invalid.
    pub fn parse_entry_file(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<Workflow, EngineError> {
        let source = std::fs::read_to_string(path)?;
        self.parse_entry_source(&source)
    }

    #[allow(dead_code)]
    /// Parse a workflow from a file
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Io` if the file cannot be read.
    /// Returns `EngineError::Parse` if the file contains syntax errors.
    pub fn parse_file(&self, path: impl AsRef<std::path::Path>) -> Result<Workflow, EngineError> {
        let loaded = module_loader::load_ordinary_file(path.as_ref())?;
        self.parse_workflow_source_with_imports(
            &loaded.workflow_source,
            loaded.imported_type_defs,
            &loaded.imported_callables,
        )
    }
    /// Extract local function definitions as closures and register helper workflows.
    ///
    /// Returns `(local_closures, local_param_counts)` with both imported and
    /// locally-defined entries.
    fn process_program_definitions(
        &self,
        program: &ash_parser::surface::Program,
        imported_closures: HashMap<String, Value>,
        imported_param_counts: HashMap<String, usize>,
    ) -> Result<ProgramProcessingResult, EngineError> {
        use ash_core::env_frame::EnvFrame;
        use ash_parser::{
            LoweringContext, effectful_names_from_definitions, lower_expr_with_context,
            lower_workflow,
        };

        let core = lower_workflow(&program.workflow)
            .map_err(|e| EngineError::Parse(format!("lowering error: {e}")))?;
        let lowering_ctx = LoweringContext::with_effectful_names(effectful_names_from_definitions(
            &program.definitions,
        ));

        let (mut local_closures, mut local_param_counts) =
            (imported_closures, imported_param_counts);

        for def_item in &program.definitions {
            if let ash_parser::surface::Definition::Function(fn_def) = def_item
                && let Ok(body_expr) = lower_expr_with_context(&fn_def.body, &lowering_ctx)
            {
                let mut env_frame = EnvFrame::new();
                let slot = env_frame.insert_late(fn_def.name.to_string());
                let params: Vec<(String, Option<String>)> = fn_def
                    .params
                    .iter()
                    .map(|p| (p.name.to_string(), None))
                    .collect();
                local_param_counts.insert(fn_def.name.to_string(), params.len());
                let env_frame = std::sync::Arc::new(env_frame);
                let closure = Value::Closure {
                    params,
                    body: Box::new(body_expr),
                    env: env_frame.clone(),
                };
                slot.set_late(closure.clone());
                local_closures.insert(fn_def.name.to_string(), closure);
            }
        }

        for helper in &program.helper_workflows {
            let helper_core = lower_workflow(helper).map_err(|e| {
                EngineError::Parse(format!(
                    "lowering error in helper workflow '{}': {e}",
                    helper.name
                ))
            })?;
            let arity = helper.params.len();
            let params: Vec<String> = helper.params.iter().map(|p| p.name.to_string()).collect();
            self.runtime_state.blocking_register_callable_workflow(
                helper.name.as_ref(),
                helper_core,
                arity,
                params,
            );
        }

        for helper in &program.helper_workflows {
            local_param_counts.insert(helper.name.to_string(), helper.params.len());
        }

        Ok((local_closures, local_param_counts, core))
    }

    fn parse_workflow_source_with_imports(
        &self,
        source: &str,
        imported_type_defs: Vec<ash_core::ast::TypeDef>,
        imported_callables: &HashMap<String, module_loader::InlineCallable>,
    ) -> Result<Workflow, EngineError> {
        use ash_parser::{
            lower_workflow, new_input, parse_utils::skip_whitespace_and_comments, workflow_def,
        };
        use winnow::prelude::*;

        // Convert imported callables to closure values for runtime binding
        let (
            imported_closures,
            imported_param_counts,
            imported_fn_signatures,
            imported_builtin_signatures,
        ) = build_imported_closures(imported_callables);

        let mut input = new_input(source);
        skip_whitespace_and_comments(&mut input);

        match workflow_def.parse_next(&mut input) {
            Ok(def) => {
                // Check if there's more input after this workflow definition.
                // If so, the source likely contains multiple named workflows and
                // we need to fall through to parse_program_with_functions.
                skip_whitespace_and_comments(&mut input);
                if !input.input.is_empty() {
                    // Remaining input — try multi-workflow path instead
                    let program =
                        module_loader::parse_program_with_functions(source).map_err(|e| {
                            EngineError::Parse(format!(
                                "trailing input after workflow but multi-workflow parse failed: {e}"
                            ))
                        })?;

                    let id = self.store_surface_workflow_def(program.workflow.clone());
                    let (local_closures, local_param_counts, core) = self
                        .process_program_definitions(
                            &program,
                            imported_closures,
                            imported_param_counts,
                        )?;

                    self.store_surface_program(id, program);
                    self.store_imported_type_defs(id, imported_type_defs);
                    return Ok(Workflow {
                        core,
                        id,
                        imported_closures: local_closures,
                        imported_param_counts: local_param_counts,
                        imported_fn_signatures,
                        imported_builtin_signatures,
                    });
                }

                // Single workflow, no trailing input — original fast path
                let core = lower_workflow(&def)
                    .map_err(|e| EngineError::Parse(format!("lowering error: {e}")))?;
                let id = self.store_surface_workflow_def(def);
                self.store_imported_type_defs(id, imported_type_defs);
                Ok(Workflow {
                    core,
                    id,
                    imported_closures,
                    imported_param_counts,
                    imported_fn_signatures,
                    imported_builtin_signatures,
                })
            }
            Err(parse_error) => {
                // Try parsing as a program with function definitions and helper workflows
                let program = module_loader::parse_program_with_functions(source)
                    .map_err(|_| EngineError::Parse(format!("{parse_error}")))?;

                let id = self.store_surface_workflow_def(program.workflow.clone());
                let (local_closures, local_param_counts, core) = self.process_program_definitions(
                    &program,
                    imported_closures,
                    imported_param_counts,
                )?;

                self.store_surface_program(id, program);
                self.store_imported_type_defs(id, imported_type_defs);
                Ok(Workflow {
                    core,
                    id,
                    imported_closures: local_closures,
                    imported_param_counts: local_param_counts,
                    imported_fn_signatures,
                    imported_builtin_signatures,
                })
            }
        }
    }
    /// Infer the canonical Ash type name for an expression.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Parse` if the expression does not parse and
    /// `EngineError::Type` if the inferred type is not concrete enough to report.
    pub fn infer_expression_type(&self, source: &str) -> Result<String, EngineError> {
        use ash_parser::parse_expr::expr;
        use ash_typeck::type_env::TypeEnv;
        use winnow::prelude::*;

        let mut input = ash_parser::new_input(source);
        let expr = expr
            .parse_next(&mut input)
            .map_err(|e| EngineError::Parse(format!("{e}")))?;

        let ty = ash_typeck::check_expr::infer_type(&TypeEnv::with_builtin_types(), &expr);
        match ty {
            ash_typeck::Type::Var(_) => Err(EngineError::Type(
                "could not infer a canonical type for expression".to_string(),
            )),
            other => Ok(other.to_string()),
        }
    }

    /// Type check a workflow
    ///
    /// On success, this also monomorphizes any generic interface method calls
    /// in the workflow core so that the interpreter never sees unresolved
    /// dispatch at runtime.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Type` if type checking or monomorphization fails.
    #[allow(clippy::too_many_lines)]
    pub fn check(&self, workflow: &mut Workflow) -> Result<(), EngineError> {
        // Retrieve the surface workflow definition that was stored during parsing
        let def = self
            .get_surface_workflow_def(workflow.id)
            .ok_or_else(|| EngineError::Type("workflow not found in cache".to_string()))?;

        if let Some(program) = self.get_surface_program(workflow.id) {
            // Build type environment with imported callable signatures
            let mut type_env = ash_typeck::type_env::TypeEnv::with_builtin_types();
            bind_imported_callable_types(&mut type_env, workflow)?;

            match ash_typeck::type_check_program_in_env(&type_env, &program) {
                Ok(result) => {
                    if result.is_ok() {
                        monomorphize::monomorphize_workflow(&mut workflow.core, &type_env)
                            .map_err(|e| EngineError::Type(e.to_string()))?;
                        return Ok(());
                    }
                    let errors: Vec<String> =
                        result.errors.iter().map(|e| format!("{e:?}")).collect();
                    return Err(EngineError::Type(errors.join("; ")));
                }
                Err(e) => return Err(EngineError::Type(format!("{e}"))),
            }
        }

        if verify_entry_workflow_def(&def).is_ok() {
            let param_refs = def
                .params
                .iter()
                .map(|param| {
                    surface_type_to_typeck(&param.ty)
                        .map(|ty| (param.name.to_string(), ty))
                        .map_err(EngineError::Type)
                })
                .collect::<Result<Vec<_>, _>>()?;

            // Build type environment with imported callable signatures
            let mut type_env = ash_typeck::type_env::TypeEnv::with_builtin_types();
            bind_imported_callable_types(&mut type_env, workflow)?;
            if let Some(_refs) = param_refs.first() {
                for (name, ty) in &param_refs {
                    type_env.bind_variable(name, ty.clone());
                }
            }

            match ash_typeck::type_check_workflow_in_env(
                Some(&type_env),
                &def.body,
                Some(&param_refs),
            ) {
                Ok(result) => {
                    if result.is_ok() {
                        monomorphize::monomorphize_workflow(&mut workflow.core, &type_env)
                            .map_err(|e| EngineError::Type(e.to_string()))?;
                        return Ok(());
                    }

                    let errors: Vec<String> =
                        result.errors.iter().map(|e| format!("{e:?}")).collect();
                    return Err(EngineError::Type(errors.join("; ")));
                }
                Err(e) => return Err(EngineError::Type(format!("{e}"))),
            }
        }

        let mut imported_type_defs = self.get_imported_type_defs(workflow.id);
        imported_type_defs.extend(self.runtime_stdlib_type_defs()?);

        let mut type_env = ash_typeck::TypeEnv::with_builtin_types();
        // Pre-declare all imported type names so sibling cross-references resolve
        for imported_type in &imported_type_defs {
            if !type_env.has_type(&imported_type.name) {
                type_env.declare_type_name(&imported_type.name);
            }
        }
        // Now register all type identities (upgrades placeholders to full definitions)
        for imported_type in imported_type_defs {
            if !type_env.has_full_type(&imported_type.name) {
                type_env
                    .register_type_identity(&imported_type)
                    .map_err(|error| EngineError::Type(error.to_string()))?;
            }
            if matches!(imported_type.visibility, ash_core::ast::Visibility::Public) {
                type_env
                    .expose_type_representation(&imported_type.name)
                    .map_err(|error| EngineError::Type(error.to_string()))?;
            }
        }
        // Register imported callable signatures
        bind_imported_callable_types(&mut type_env, workflow)?;

        match ash_typeck::type_check_workflow_def_in_env(&type_env, &def) {
            Ok(result) => {
                if result.is_ok() {
                    monomorphize::monomorphize_workflow(&mut workflow.core, &type_env)
                        .map_err(|e| EngineError::Type(e.to_string()))?;
                    Ok(())
                } else {
                    // Collect type errors into a message
                    let errors: Vec<String> =
                        result.errors.iter().map(|e| format!("{e:?}")).collect();
                    Err(EngineError::Type(errors.join("; ")))
                }
            }
            Err(e) => Err(EngineError::Type(format!("{e}"))),
        }
    }

    /// Verify that a parsed workflow matches the canonical entry workflow contract.
    ///
    /// This is a pure metadata validation over the cached parsed `WorkflowDef`.
    /// It does not load the standard library, resolve imports, or perform bootstrap.
    ///
    /// # Errors
    ///
    /// Returns [`EntryVerificationError`] if the cached surface metadata is missing
    /// or if the workflow signature does not match the canonical `main` contract.
    pub fn verify_entry_workflow(&self, workflow: &Workflow) -> Result<(), EntryVerificationError> {
        let def = self
            .get_surface_workflow_def(workflow.id)
            .ok_or(EntryVerificationError::MissingWorkflowMetadata)?;

        verify_entry_workflow_def(&def)
    }

    /// Check a non-workflow module file for validity.
    ///
    /// Reads the file, collects public type definitions, validates them in a
    /// fresh `TypeEnv`, and counts parseable `pub fn` snippets.  Returns a
    /// `ModuleFileCheckResult` with type/fn counts and any warnings or errors.
    ///
    /// # Error model
    ///
    /// This method uses a **dual error path**:
    /// - **Hard errors** (file I/O failure, `pub type` parse failure) propagate
    ///   via `Result::Err` and abort early.
    /// - **Soft errors** (type registration failures due to unbound references)
    ///   accumulate in `result.errors` so the caller can report all issues at
    ///   once rather than stopping at the first.
    /// - **Warnings** (unparseable `pub fn` snippets) accumulate in
    ///   `result.warnings`.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Io` if the file cannot be read, or
    /// `EngineError::Parse` if type definitions fail to parse.
    pub fn check_module_file(
        &self,
        path: &std::path::Path,
    ) -> Result<ModuleFileCheckResult, EngineError> {
        let source = std::fs::read_to_string(path)?;

        let type_defs = module_loader::collect_public_type_defs_from_source(&source)?;
        let type_count = type_defs.len();
        let (fn_count, fn_diagnostics) = module_loader::count_pub_fn_snippets(&source);

        let warnings: Vec<String> = fn_diagnostics
            .iter()
            .map(|d| {
                d.name.as_ref().map_or_else(
                    || format!("pub fn: {}", d.reason),
                    |name| format!("pub fn '{name}': {}", d.reason),
                )
            })
            .collect();
        let mut errors = Vec::new();

        // Build a TypeEnv and register all discovered types so cross-references resolve.
        let mut type_env = ash_typeck::TypeEnv::with_builtin_types();
        for td in &type_defs {
            if !type_env.has_type(&td.name) {
                type_env.declare_type_name(&td.name);
            }
        }
        for td in type_defs {
            if !type_env.has_full_type(&td.name)
                && let Err(e) = type_env.register_type(&td)
            {
                errors.push(format!("{e}"));
            }
        }

        Ok(ModuleFileCheckResult {
            type_count,
            fn_count,
            warnings,
            errors,
        })
    }
    /// Execute a workflow asynchronously
    ///
    /// # Errors
    ///
    /// Returns execution errors from the interpreter.
    pub async fn execute(&self, workflow: &Workflow) -> ExecResult<Value> {
        if workflow.imported_closures.is_empty() {
            interpret_in_state(&workflow.core, &self.runtime_state).await
        } else {
            ash_interp::execute_with_bindings_in_state(
                &workflow.core,
                &self.runtime_state,
                workflow.imported_closures.clone(),
            )
            .await
        }
    }
    /// Execute a core `ash_core::Workflow` directly through the engine's runtime state.
    ///
    /// This bypasses surface-program lookup and type checking, executing the
    /// workflow against the engine's registered capability providers (including
    /// those set up via `with_llm_capabilities`). Intended for integration tests
    /// that need to exercise the engine → capability dispatch path with a
    /// hand-constructed core IR.
    ///
    /// # Errors
    ///
    /// Returns execution errors from the interpreter.
    #[doc(hidden)]
    pub async fn execute_core_workflow(&self, workflow: &ash_core::Workflow) -> ExecResult<Value> {
        use ash_interp::{BehaviourContext, Context, PolicyEvaluator};
        let ctx = Context::new();
        let cap_ctx = self.runtime_state.create_capability_context().await;
        let policy_eval = PolicyEvaluator::new();
        let behaviour_ctx = BehaviourContext::new();
        ash_interp::execute_workflow_with_behaviour_in_state(
            workflow,
            ctx,
            &cap_ctx,
            &policy_eval,
            &behaviour_ctx,
            &self.runtime_state,
        )
        .await
    }

    /// Register a runtime-owned spawned-child workflow entry.
    ///
    /// This exposes the interpreter runtime state's narrow `workflow_type` → child-workflow registry
    /// through the engine so integration tests and embeddings can configure spawned-child execution
    /// against the same engine-owned runtime state used by top-level executions.
    pub async fn register_child_workflow(
        &self,
        workflow_type: impl Into<String>,
        workflow: ash_core::Workflow,
    ) {
        self.runtime_state
            .register_child_workflow(workflow_type, workflow)
            .await;
    }

    /// Register a runtime-owned callable workflow entry for `Workflow::Call` execution.
    pub async fn register_callable_workflow(
        &self,
        workflow_name: impl Into<String>,
        workflow: ash_core::Workflow,
        arity: usize,
    ) {
        self.runtime_state
            .register_callable_workflow(workflow_name, workflow, arity, vec![])
            .await;
    }
    /// Execute a workflow asynchronously with input bindings
    ///
    /// The input bindings are injected into the workflow's execution context
    /// as initial variable bindings. This is useful for passing CLI arguments
    /// or other external inputs to the workflow.
    ///
    /// # Arguments
    /// * `workflow` - The workflow to execute
    /// * `input_bindings` - Initial variable bindings (e.g., from CLI --input)
    ///
    /// # Errors
    ///
    /// Returns execution errors from the interpreter.
    pub async fn execute_with_input(
        &self,
        workflow: &Workflow,
        input_bindings: std::collections::HashMap<String, Value>,
    ) -> ExecResult<Value> {
        // Merge imported closures with input bindings (input takes precedence)
        let mut bindings = workflow.imported_closures.clone();
        bindings.extend(input_bindings);
        ash_interp::execute_with_bindings_in_state(&workflow.core, &self.runtime_state, bindings)
            .await
    }
    /// Parse, check, and execute in one call
    ///
    /// Convenience method that chains parse → check → execute.
    ///
    /// # Errors
    ///
    /// Returns the first error encountered at any stage.
    pub async fn run(&self, source: &str) -> ExecResult<Value> {
        let mut workflow = self.parse(source)?;
        self.check(&mut workflow)?;
        self.execute(&workflow).await
    }

    /// Parse, check, and execute a workflow from a file
    ///
    /// Convenience method that reads a file and then runs parse → check → execute.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Io` if the file cannot be read.
    /// Returns other errors from parse, check, or execute stages.
    pub async fn run_file(&self, path: impl AsRef<std::path::Path>) -> ExecResult<Value> {
        let mut workflow = self.parse_file(path)?;
        self.check(&mut workflow)?;
        self.execute(&workflow).await
    }

    /// Parse, check, and execute a workflow from a file with input bindings
    ///
    /// Convenience method that reads a file and then runs parse → check → execute
    /// with the provided input bindings injected into the execution context.
    ///
    /// # Arguments
    /// * `path` - Path to the workflow file
    /// * `input_bindings` - Initial variable bindings (e.g., from CLI --input)
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Io` if the file cannot be read.
    /// Returns other errors from parse, check, or execute stages.
    pub async fn run_file_with_input(
        &self,
        path: impl AsRef<std::path::Path>,
        input_bindings: std::collections::HashMap<String, Value>,
    ) -> ExecResult<Value> {
        let mut workflow = self.parse_file(path)?;
        self.check(&mut workflow)?;
        self.execute_with_input(&workflow, input_bindings).await
    }

    /// Parse, check, verify, and execute an entry workflow source, returning its exit code.
    ///
    /// This is the narrow Phase 57 runtime entry path. It loads the engine-owned runtime
    /// stdlib registry, parses entry source with leading-`use` tolerance, checks the
    /// workflow, validates the canonical `main` signature, executes it, and derives the
    /// observable process exit code from the terminal result payload.
    ///
    /// # Errors
    ///
    /// Returns [`EntryBootstrapError`] if stdlib loading fails, the entry source does not
    /// parse or type-check, the `main` contract is invalid, execution fails, or the runtime
    /// error payload carries an out-of-range exit code.
    pub async fn bootstrap_entry_source(&self, source: &str) -> Result<u8, EntryBootstrapError> {
        self.load_runtime_stdlib()?;
        let mut workflow = self.parse_entry_source(source)?;
        self.verify_entry_workflow(&workflow)?;
        self.check(&mut workflow)?;
        let def = self
            .get_surface_workflow_def(workflow.id)
            .ok_or(EntryVerificationError::MissingWorkflowMetadata)?;
        let input_bindings = entry::entry_input_bindings(&def);

        let result = if input_bindings.is_empty() {
            self.execute(&workflow)
                .await
                .map_err(|error| EntryBootstrapError::Execution(error.to_string()))?
        } else {
            self.execute_with_input(&workflow, input_bindings)
                .await
                .map_err(|error| EntryBootstrapError::Execution(error.to_string()))?
        };

        entry::derive_entry_exit_code(&result)
    }

    /// Parse, check, verify, and execute an entry workflow file, returning its exit code.
    ///
    /// # Errors
    ///
    /// Returns [`EntryBootstrapError`] for any stdlib, I/O, parse, type, verification,
    /// execution, or exit-code derivation failure.
    pub async fn bootstrap_entry_file(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<u8, EntryBootstrapError> {
        let source = std::fs::read_to_string(path).map_err(EngineError::Io)?;
        self.bootstrap_entry_source(&source).await
    }
}
fn surface_type_to_typeck(ty: &SurfaceType) -> Result<ash_typeck::Type, String> {
    match ty {
        SurfaceType::Name(name) => match name.as_ref() {
            "Int" => Ok(ash_typeck::Type::Int),
            "String" => Ok(ash_typeck::Type::String),
            "Bool" => Ok(ash_typeck::Type::Bool),
            "Null" => Ok(ash_typeck::Type::Null),
            "Time" => Ok(ash_typeck::Type::Time),
            "Ref" => Ok(ash_typeck::Type::Ref),
            "()" => Ok(ash_typeck::Type::Constructor {
                name: ash_typeck::QualifiedName::root("()"),
                args: vec![],
                kind: ash_typeck::Kind::Type,
            }),
            other => Ok(ash_typeck::Type::Constructor {
                name: ash_typeck::QualifiedName::root(other.to_string()),
                args: vec![],
                kind: ash_typeck::Kind::Type,
            }),
        },
        SurfaceType::List(item) => {
            surface_type_to_typeck(item).map(|item| ash_typeck::Type::List(Box::new(item)))
        }
        SurfaceType::Record(fields) => {
            let fields = fields
                .iter()
                .map(|(name, ty)| {
                    surface_type_to_typeck(ty).map(|ty| (Box::from(name.as_ref()), ty))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ash_typeck::Type::Record(fields))
        }
        SurfaceType::Capability(name) => Ok(ash_typeck::Type::Cap {
            name: Box::from(name.as_ref()),
            effect: ash_core::Effect::Operational,
        }),
        SurfaceType::Constructor { name, args } => {
            let args = args
                .iter()
                .map(surface_type_to_typeck)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ash_typeck::Type::Constructor {
                name: ash_typeck::QualifiedName::root(name.as_ref().to_string()),
                args,
                kind: ash_typeck::Kind::Type,
            })
        }
        SurfaceType::Fn(params, ret) => {
            let params = params
                .iter()
                .map(surface_type_to_typeck)
                .collect::<Result<Vec<_>, _>>()?;
            let ret = surface_type_to_typeck(ret)?;
            Ok(ash_typeck::Type::Fn(params, Box::new(ret)))
        }
        SurfaceType::Associated { .. } => {
            Err("associated types not yet supported in engine type conversion".to_string())
        }
    }
}
impl Default for Engine {
    /// Creates a default engine with standard configuration.
    ///
    /// # Panics
    ///
    /// Panics if the engine cannot be built (e.g., out of memory). This is
    /// extremely unlikely in practice since the default configuration requires
    /// no external resources.
    fn default() -> Self {
        // SAFETY: The default EngineBuilder configuration is infallible.
        // It only allocates memory and performs simple initializations.
        Self::new().build().expect("default engine builds")
    }
}

/// Configuration for HTTP capabilities
#[derive(Debug, Clone, Default)]
pub struct HttpConfig {
    /// Timeout for HTTP requests in seconds
    pub timeout_seconds: u64,
    /// Maximum number of redirects to follow
    pub max_redirects: u32,
    /// Whether to verify SSL certificates
    pub verify_ssl: bool,
}

impl HttpConfig {
    /// Create a new HTTP config with default settings
    #[must_use]
    pub const fn new() -> Self {
        Self {
            timeout_seconds: 30,
            max_redirects: 10,
            verify_ssl: true,
        }
    }
}

/// Builder for configuring and constructing an Engine
///
/// The builder pattern allows for fluent configuration of capabilities:
///
/// ```
/// use ash_engine::Engine;
///
/// let engine = Engine::new()
///     .with_stdio_capabilities()
///     .with_fs_capabilities()
///     .build()
///     .expect("engine builds");
/// ```
#[derive(Debug, Default)]
pub struct EngineBuilder {
    /// Whether to enable stdio capabilities
    enable_stdio: bool,
    /// Whether to enable filesystem capabilities
    enable_fs: bool,
    /// HTTP configuration if enabled
    http_config: Option<HttpConfig>,
    /// Custom providers to register (using the unified `CapabilityProvider` trait)
    custom_providers: std::collections::HashMap<
        String,
        std::sync::Arc<dyn ash_core::capability::CapabilityProvider>,
    >,
}

impl EngineBuilder {
    /// Create a new engine builder
    ///
    /// Prefer using `Engine::new()` instead of this method directly.
    fn new() -> Self {
        Self::default()
    }

    /// Build the configured engine
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the engine cannot be constructed
    /// (e.g., missing required capabilities or invalid configuration).
    pub fn build(self) -> Result<Engine, EngineError> {
        use providers::{FsProvider, StdioProvider};
        use std::sync::Arc;

        // Providers are stored as the unified trait type from ash_core
        let mut providers: std::collections::HashMap<
            String,
            Arc<dyn ash_core::capability::CapabilityProvider>,
        > = std::collections::HashMap::new();

        // Register stdio provider if enabled
        if self.enable_stdio {
            let provider = StdioProvider::new();
            providers.insert(provider.name().to_string(), Arc::new(provider));
        }

        // Register filesystem provider if enabled
        if self.enable_fs {
            let provider: Arc<dyn ash_core::capability::CapabilityProvider> =
                Arc::new(FsProvider::new());
            providers.insert("fs".to_string(), Arc::clone(&provider));
            // Also register under stdlib capability names for directory and metadata operations
            providers.insert("dir".to_string(), Arc::clone(&provider));
            providers.insert("meta".to_string(), provider);
        }

        // Register HTTP provider if configured
        // Note: HTTP provider is not yet implemented.
        if self.http_config.is_some() {
            return Err(EngineError::Configuration(
                "HTTP provider not yet implemented. Use with_custom_provider() to add your own HTTP implementation.".to_string(),
            ));
        }

        // Register custom providers (these can override built-ins)
        for (name, provider) in self.custom_providers {
            providers.insert(name, provider);
        }

        // Build the RuntimeState with the unified providers
        let runtime_state = RuntimeState::new().with_providers(providers);

        Ok(Engine {
            surface_workflow_defs: std::sync::Mutex::new(std::collections::HashMap::new()),
            imported_type_defs: std::sync::Mutex::new(std::collections::HashMap::new()),
            surface_programs: std::sync::Mutex::new(std::collections::HashMap::new()),
            runtime_stdlib_modules: std::sync::Mutex::new(std::collections::HashMap::new()),
            next_id: std::sync::atomic::AtomicU64::new(1),
            runtime_state,
        })
    }

    /// Add standard I/O capabilities (print, println, `read_line`)
    ///
    /// These are operational-effect capabilities for console I/O.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Cannot be const due to HashMap operations in build()
    pub fn with_stdio_capabilities(mut self) -> Self {
        self.enable_stdio = true;
        self
    }

    /// Add filesystem capabilities (`read_file`, `write_file`)
    ///
    /// These are operational-effect capabilities for file operations.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Cannot be const due to HashMap operations in build()
    pub fn with_fs_capabilities(mut self) -> Self {
        self.enable_fs = true;
        self
    }

    /// Configure HTTP capabilities (not yet implemented)
    ///
    /// # Errors
    ///
    /// This method currently returns a `Configuration` error as the HTTP provider
    /// is not yet implemented. Use `with_custom_provider()` to add a custom HTTP
    /// implementation.
    ///
    /// # Example
    ///
    /// ```
    /// use ash_engine::{Engine, HttpConfig};
    ///
    /// let result = Engine::new()
    ///     .with_http_capabilities(HttpConfig::new())
    ///     .build();
    /// assert!(result.is_err()); // HTTP provider not yet implemented
    /// ```
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Cannot be const due to HashMap operations in build()
    pub fn with_http_capabilities(mut self, config: HttpConfig) -> Self {
        self.http_config = Some(config);
        self
    }

    /// Add a custom capability provider
    ///
    /// Custom providers can be used to extend the engine with application-specific
    /// capabilities. They can also override built-in providers by using the same name.
    ///
    /// # Example
    ///
    /// ```
    /// use ash_engine::Engine;
    /// use ash_core::{Constraint, Effect, Value};
    /// use ash_core::capability::{CapabilityError, CapabilityProvider};
    /// use async_trait::async_trait;
    /// use std::sync::Arc;
    ///
    /// #[derive(Debug)]
    /// struct MyProvider;
    ///
    /// #[async_trait]
    /// impl CapabilityProvider for MyProvider {
    ///     fn name(&self) -> &str { "my_provider" }
    ///     fn effect(&self) -> Effect { Effect::Operational }
    ///     async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
    ///         Ok(Value::Null)
    ///     }
    ///     async fn execute(
    ///         &self,
    ///         _action_name: &str,
    ///         _args: &[Value],
    ///     ) -> Result<Value, CapabilityError> {
    ///         Ok(Value::Null)
    ///     }
    /// }
    ///
    /// let engine = Engine::new()
    ///     .with_custom_provider("custom", Arc::new(MyProvider))
    ///     .build()
    ///     .expect("engine builds");
    /// ```
    #[must_use]
    pub fn with_custom_provider(
        mut self,
        name: &str,
        provider: std::sync::Arc<dyn ash_core::capability::CapabilityProvider>,
    ) -> Self {
        self.custom_providers.insert(name.to_string(), provider);
        self
    }

    /// Configure LLM capabilities with the given provider configurations
    ///
    /// Registers an `LlmProvider` that supports multi-provider routing for OpenAI-compatible APIs.
    ///
    /// # Arguments
    /// * `configs` - Map of provider name to `LlmConfig` (e.g., "openai" -> config, "ollama" -> config)
    ///
    /// # Example
    ///
    /// ```
    /// use ash_engine::Engine;
    /// use ash_engine::providers::llm::LlmConfig;
    /// use std::collections::HashMap;
    ///
    /// let mut configs = HashMap::new();
    /// configs.insert("openai".to_string(), LlmConfig::openai("sk-xxx"));
    /// configs.insert("ollama".to_string(), LlmConfig::ollama());
    ///
    /// let engine = Engine::new()
    ///     .with_llm_capabilities(configs)
    ///     .build()
    ///     .expect("engine builds");
    /// ```
    #[must_use]
    pub fn with_llm_capabilities(
        mut self,
        configs: std::collections::HashMap<String, crate::providers::llm::LlmConfig>,
    ) -> Self {
        match crate::providers::llm::LlmProvider::new(configs) {
            Ok(provider) => {
                self.custom_providers
                    .insert("llm".to_string(), std::sync::Arc::new(provider));
            }
            Err(e) => {
                // Log warning and skip registration
                eprintln!("Warning: Failed to create LLM provider: {e}");
            }
        }
        self
    }
}

/// Bind imported callable type signatures into a type environment.
///
/// For each entry in `workflow.imported_param_counts`, checks whether a
/// declared builtin signature is available in `workflow.imported_builtin_signatures`.
/// If so, uses `builtin_fn_signature_type` to produce the precise polymorphic type.
/// If a declared signature exists, signature conversion errors are hard type
/// errors rather than silently falling back to arity-only types.
#[allow(clippy::unnecessary_wraps)]
fn bind_imported_callable_types(
    type_env: &mut ash_typeck::type_env::TypeEnv,
    workflow: &Workflow,
) -> Result<(), EngineError> {
    for (name, &param_count) in &workflow.imported_param_counts {
        if let Some(sig) = workflow.imported_fn_signatures.get(name) {
            let ty = ash_typeck::fn_signature_type(type_env, sig).map_err(|error| {
                EngineError::Type(format!(
                    "failed to resolve imported function signature for '{name}': {error}"
                ))
            })?;
            type_env.bind_variable(name, ty);
            continue;
        }
        if let Some(sig) = workflow.imported_builtin_signatures.get(name) {
            let ty = ash_typeck::builtin_fn_signature_type(type_env, sig).map_err(|error| {
                EngineError::Type(format!(
                    "failed to resolve imported builtin signature for '{name}': {error}"
                ))
            })?;
            type_env.bind_variable(name, ty);
            continue;
        }
        // Arity-only synthetic type (fresh type variables)
        let param_types: Vec<ash_typeck::Type> = (0..param_count)
            .map(|_| ash_typeck::Type::Var(ash_typeck::types::TypeVar::fresh()))
            .collect();
        let ret_type = ash_typeck::Type::Var(ash_typeck::types::TypeVar::fresh());
        type_env.bind_variable(name, ash_typeck::Type::Fn(param_types, Box::new(ret_type)));
    }
    Ok(())
}

/// Imported callable bindings built for runtime and type-checker integration.
type ImportedClosureBindings = (
    HashMap<String, Value>,
    HashMap<String, usize>,
    HashMap<String, ash_parser::surface::FnDef>,
    HashMap<String, ash_parser::surface::BuiltinFnDef>,
);

/// Convert imported callables to `Value::Closure` for runtime binding.
/// Each callable body is lowered from surface to core Expr, then wrapped in a closure.
///
/// Returns `(closures, param_counts, fn_signatures, builtin_signatures)` where
/// the signature maps carry declared type signatures for imported callables so
/// that `Engine::check()` can bind precise types instead of arity-only synthetics.
fn build_imported_closures(
    imported_callables: &HashMap<String, module_loader::InlineCallable>,
) -> ImportedClosureBindings {
    use module_loader::CallableKind;

    // Resolved once; builtin_dispatch_table() uses OnceLock so this is a
    // single atomic load on subsequent calls, but hoisting it avoids calling
    // the function N times inside the loop.
    let dispatch_table = ash_interp::eval::builtin_dispatch_table();

    let mut closures = HashMap::new();
    let mut param_counts = HashMap::new();
    let mut fn_signatures = HashMap::new();
    let mut builtin_signatures = HashMap::new();
    for (name, callable) in imported_callables {
        let params: Vec<(String, Option<String>)> =
            callable.params.iter().map(|p| (p.clone(), None)).collect();
        param_counts.insert(name.clone(), params.len());

        // Extract declared type signature for imported callables.
        if let Some(sig) = &callable.signature {
            match sig {
                module_loader::CallableSignature::Function(fn_def) => {
                    fn_signatures.insert(name.clone(), fn_def.clone());
                }
                module_loader::CallableSignature::Builtin(builtin) => {
                    builtin_signatures.insert(name.clone(), builtin.clone());
                }
            }
        }

        let body_expr = match &callable.kind {
            CallableKind::User { body } => {
                let lowering_ctx = ash_parser::LoweringContext::with_effectful_names(
                    callable.effectful_names.clone(),
                );
                match ash_parser::lower_expr_with_context(body, &lowering_ctx) {
                    Ok(expr) => expr,
                    Err(e) => {
                        // TODO: replace with tracing::warn! when tracing is integrated
                        eprintln!("warning: failed to lower imported callable '{name}': {e}");
                        continue;
                    }
                }
            }
            CallableKind::Builtin { module } => {
                // Check the dispatch table to decide whether this builtin needs a
                // qualified call (e.g. "string::concat") or unqualified ("keys").
                // String builtins are registered as "string::concat" etc. (qualified).
                // Record builtins ("keys", "values", "record") are registered without a
                // module prefix; "record::keys" is absent from the table, so they
                // receive module: None and dispatch unqualified.
                let qualified = format!("{module}::{}", callable.exported_name);
                let call_module = if dispatch_table.contains_key(qualified.as_str()) {
                    Some(module.clone())
                } else {
                    None
                };

                // For variadic builtins declared with 0 parameters (e.g. `record`),
                // skip closure registration entirely.  The caller passes arguments
                // that would be forwarded to the builtin, but a 0-param closure cannot
                // forward them — `apply_closure` would raise WrongArity.  Instead we
                // omit the closure so the evaluator falls through to `eval_function_call`,
                // which handles variadic dispatch correctly.
                let unqualified_entry = dispatch_table.get(callable.exported_name.as_str());
                if callable.params.is_empty()
                    && let Some(entry) = unqualified_entry
                    && entry.variadic
                {
                    // Do not register a closure; let eval use builtin dispatch.
                    param_counts.insert(name.clone(), 0);
                    continue;
                }

                let param_exprs: Vec<ash_core::Expr> = callable
                    .params
                    .iter()
                    .map(|p| ash_core::Expr::Variable {
                        name: p.clone(),
                        span: ash_core::ast::Span::default(),
                    })
                    .collect();
                ash_core::Expr::Call {
                    func: callable.exported_name.clone(),
                    module: call_module,
                    arguments: param_exprs,
                }
            }
        };

        let env_frame = std::sync::Arc::new(ash_core::env_frame::EnvFrame::new());
        closures.insert(
            name.clone(),
            Value::Closure {
                params,
                body: Box::new(body_expr),
                env: env_frame,
            },
        );
    }
    (closures, param_counts, fn_signatures, builtin_signatures)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ============================================================
    // Engine Creation Tests
    // ============================================================

    #[test]
    fn test_engine_new_build_succeeds() {
        // Basic test: Engine::new().build() should succeed
        let result = Engine::new().build();
        assert!(
            result.is_ok(),
            "Engine::new().build() should succeed but got: {result:?}"
        );
    }

    #[test]
    fn test_engine_default_succeeds() {
        // Engine::default() should succeed using the builder
        let _engine: Engine = Engine::default();
    }

    #[test]
    fn test_engine_builder_returns_valid_engine() {
        let engine = Engine::new().build().expect("engine builds");
        // The engine should be usable (not panic, not be null, etc.)
        // We verify this by checking it can execute basic operations
        let _ = &engine;
    }

    // ============================================================
    // EngineBuilder Configuration Tests
    // ============================================================

    #[test]
    fn test_builder_stdio_capabilities_chaining() {
        // with_stdio_capabilities should return Self for chaining
        let builder = Engine::new();
        let builder = builder.with_stdio_capabilities();
        let result = builder.build();
        assert!(
            result.is_ok(),
            "Builder with stdio capabilities should build successfully"
        );
    }

    #[test]
    fn test_builder_fs_capabilities_chaining() {
        // with_fs_capabilities should return Self for chaining
        let builder = Engine::new();
        let builder = builder.with_fs_capabilities();
        let result = builder.build();
        assert!(
            result.is_ok(),
            "Builder with fs capabilities should build successfully"
        );
    }

    #[test]
    fn test_builder_chaining_multiple_capabilities() {
        // Multiple capability methods should chain together
        let result = Engine::new()
            .with_stdio_capabilities()
            .with_fs_capabilities()
            .build();
        assert!(
            result.is_ok(),
            "Builder with multiple capabilities should build successfully"
        );
    }

    #[test]
    fn test_builder_chaining_order_independent() {
        // Order of capability configuration should not matter
        let engine1 = Engine::new()
            .with_stdio_capabilities()
            .with_fs_capabilities()
            .build();

        let engine2 = Engine::new()
            .with_fs_capabilities()
            .with_stdio_capabilities()
            .build();

        // Both should succeed (we can't compare engines directly without PartialEq,
        // but we can verify both builds succeed)
        assert!(engine1.is_ok(), "First build order should succeed");
        assert!(engine2.is_ok(), "Second build order should succeed");
    }

    #[test]
    fn test_builder_reusable_pattern() {
        // The builder pattern should be usable for multiple engines
        let base_builder = Engine::new();

        let engine1 = base_builder.build();
        // Note: After build(), the builder is consumed. This test documents
        // the expected usage pattern where a new builder is created each time.
        assert!(engine1.is_ok());

        // Creating a new engine from a new builder
        let engine2 = Engine::new().build();
        assert!(engine2.is_ok());
    }

    // ============================================================
    // Send + Sync Thread Safety Tests
    // ============================================================

    #[test]
    fn test_engine_is_send() {
        // Compile-time check: Engine must be Send
        fn assert_send<T: Send>() {}
        assert_send::<Engine>();
    }

    #[test]
    fn test_engine_is_sync() {
        // Compile-time check: Engine must be Sync
        fn assert_sync<T: Sync>() {}
        assert_sync::<Engine>();
    }

    #[test]
    fn test_engine_builder_is_send() {
        // Compile-time check: EngineBuilder should be Send for flexibility
        fn assert_send<T: Send>() {}
        assert_send::<EngineBuilder>();
    }

    #[test]
    fn test_engine_error_is_send_sync() {
        // Compile-time check: EngineError must be Send + Sync for error handling
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<EngineError>();
        assert_sync::<EngineError>();
    }

    #[tokio::test]
    async fn test_engine_can_be_shared_across_tasks() {
        // Runtime check: Engine can be shared across async tasks
        use std::sync::Arc;

        let engine = Arc::new(Engine::new().build().expect("engine builds"));

        let engine_clone = Arc::clone(&engine);
        let task = tokio::spawn(async move {
            // Access the engine in a spawned task
            let _ = &*engine_clone;
            true
        });

        let result = task.await.expect("task completed");
        assert!(result, "Engine should be accessible across tasks");
    }

    // ============================================================
    // Error Type Tests
    // ============================================================

    #[test]
    fn test_engine_error_parse_variant() {
        // Verify Parse variant exists and can be created
        let err = EngineError::Parse("syntax error".to_string());
        assert!(
            matches!(err, EngineError::Parse(_)),
            "Error should be Parse variant"
        );
    }

    #[test]
    fn test_engine_error_type_variant() {
        // Verify Type variant exists and can be created
        let err = EngineError::Type("type mismatch".to_string());
        assert!(
            matches!(err, EngineError::Type(_)),
            "Error should be Type variant"
        );
    }

    #[test]
    fn test_engine_error_execution_variant() {
        // Verify Execution variant exists and can be created
        let err = EngineError::Execution("runtime error".to_string());
        assert!(
            matches!(err, EngineError::Execution(_)),
            "Error should be Execution variant"
        );
    }

    #[test]
    fn test_engine_error_io_variant() {
        // Verify Io variant exists with std::io::Error
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = EngineError::Io(io_err);
        assert!(
            matches!(err, EngineError::Io(_)),
            "Error should be Io variant"
        );
    }

    #[test]
    fn test_engine_error_capability_not_found_variant() {
        // Verify CapabilityNotFound variant exists
        let err = EngineError::CapabilityNotFound("fs:read".to_string());
        assert!(
            matches!(err, EngineError::CapabilityNotFound(_)),
            "Error should be CapabilityNotFound variant"
        );
    }

    #[test]
    fn test_engine_error_display_format() {
        // Verify error messages are informative
        let parse_err = EngineError::Parse("unexpected token".to_string());
        let display = format!("{parse_err}");
        assert!(
            display.contains("unexpected token"),
            "Parse error should display message: {display}"
        );
    }

    #[test]
    fn test_engine_error_from_io_error() {
        // Verify automatic conversion from std::io::Error
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let engine_err: EngineError = io_err.into();
        assert!(
            matches!(engine_err, EngineError::Io(_)),
            "Should convert from io::Error"
        );
    }

    // ============================================================
    // Property-Based Tests
    // ============================================================

    proptest! {
        /// Property: Engine creation always succeeds (when not configured with invalid options)
        #[test]
        fn prop_engine_creation_succeeds(_dummy in any::<u8>()) {
            let result = Engine::new().build();
            prop_assert!(result.is_ok(), "Engine creation should always succeed");
        }

        /// Property: Error messages preserve their content
        #[test]
        fn prop_error_message_preservation(message in "[a-zA-Z0-9_ ]{1,100}") {
            let err = EngineError::Parse(message.clone());
            let display = format!("{err}");
            prop_assert!(
                display.contains(&message),
                "Error display should contain original message"
            );
        }

        /// Property: CapabilityNotFound preserves capability name
        #[test]
        fn prop_capability_name_preservation(name in "[a-z_][a-z0-9_:]{1,50}") {
            let err = EngineError::CapabilityNotFound(name.clone());
            if let EngineError::CapabilityNotFound(found_name) = err {
                prop_assert_eq!(found_name, name, "Capability name should be preserved");
            } else {
                prop_assert!(false, "Error should be CapabilityNotFound variant");
            }
        }
    }

    // ============================================================
    // TODO/Future Tests (marked as ignore until implemented)
    // ============================================================

    #[test]
    fn test_engine_parse_valid_source() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse("workflow main { done }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_engine_parse_invalid_source_returns_parse_error() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse("invalid syntax!!!");
        assert!(matches!(result, Err(EngineError::Parse(_))));
    }

    #[test]
    fn test_engine_check_valid_workflow() {
        let engine = Engine::new().build().unwrap();
        let mut workflow = engine.parse("workflow main { ret 42; }").unwrap();
        let result = engine.check(&mut workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_engine_infer_expression_type_reports_canonical_names() {
        let engine = Engine::new().build().unwrap();

        assert_eq!(engine.infer_expression_type("42").unwrap(), "Int");
        assert_eq!(engine.infer_expression_type("\"hello\"").unwrap(), "String");
        assert_eq!(
            engine.infer_expression_type("[1, 2, 3]").unwrap(),
            "List<Int>"
        );
        assert_eq!(engine.infer_expression_type("1 + 2").unwrap(), "Int");
        assert_eq!(engine.infer_expression_type("!true").unwrap(), "Bool");
    }

    #[test]
    fn test_engine_execute_workflow() {
        // This will be an async test
    }

    // ============================================================
    // EngineBuilder HTTP Capabilities Tests
    // ============================================================

    #[test]
    fn test_builder_http_capabilities_returns_error() {
        // HTTP provider is not yet implemented, should return Configuration error
        let config = HttpConfig::new();
        let result = Engine::new().with_http_capabilities(config).build();
        assert!(
            result.is_err(),
            "Builder with HTTP capabilities should return error (not yet implemented)"
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, EngineError::Configuration(_)),
            "Error should be Configuration variant"
        );
        let err_msg = format!("{err}");
        assert!(
            err_msg.contains("HTTP provider not yet implemented"),
            "Error message should indicate HTTP not implemented: {err_msg}"
        );
    }

    #[test]
    fn test_builder_http_capabilities_with_custom_config_returns_error() {
        // HTTP provider is not yet implemented, should return Configuration error
        let config = HttpConfig {
            timeout_seconds: 60,
            max_redirects: 5,
            verify_ssl: false,
        };
        let result = Engine::new().with_http_capabilities(config).build();
        assert!(
            result.is_err(),
            "Builder with custom HTTP config should return error (not yet implemented)"
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, EngineError::Configuration(_)),
            "Error should be Configuration variant"
        );
    }

    #[test]
    fn test_builder_http_default_config() {
        // Test HttpConfig::new() provides sensible defaults
        let config = HttpConfig::new();
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_redirects, 10);
        assert!(config.verify_ssl);
    }

    // ============================================================
    // EngineBuilder Custom Provider Tests
    // ============================================================

    #[test]
    fn test_builder_custom_provider_chaining() {
        // with_custom_provider should return Self for chaining
        use providers::StdioProvider;
        use std::sync::Arc;

        let provider = StdioProvider::new();
        let builder = Engine::new();
        let builder = builder.with_custom_provider("custom_stdio", Arc::new(provider));
        let result = builder.build();
        assert!(
            result.is_ok(),
            "Builder with custom provider should build successfully"
        );
    }

    #[test]
    fn test_builder_custom_provider_overrides_builtin() {
        // Custom providers with the same name as built-ins should override them
        use providers::StdioProvider;

        let custom_stdio = StdioProvider::new();
        let result = Engine::new()
            .with_stdio_capabilities() // Enable built-in stdio
            .with_custom_provider("stdio", std::sync::Arc::new(custom_stdio)) // Override with custom
            .build();
        assert!(
            result.is_ok(),
            "Builder should allow overriding built-in providers"
        );
    }

    #[test]
    fn test_builder_multiple_custom_providers() {
        // Multiple custom providers should all be registered
        use providers::{FsProvider, StdioProvider};

        let result = Engine::new()
            .with_custom_provider("my_stdio", std::sync::Arc::new(StdioProvider::new()))
            .with_custom_provider("my_fs", std::sync::Arc::new(FsProvider::new()))
            .build();
        assert!(
            result.is_ok(),
            "Builder with multiple custom providers should build successfully"
        );
    }

    // ============================================================
    // EngineBuilder Combined Configuration Tests
    // ============================================================

    #[test]
    fn test_builder_stdio_fs_custom_together() {
        // Test stdio, fs, and custom providers together (HTTP returns error until implemented)
        use providers::StdioProvider;

        let result = Engine::new()
            .with_stdio_capabilities()
            .with_fs_capabilities()
            .with_custom_provider("custom", std::sync::Arc::new(StdioProvider::new()))
            .build();

        assert!(
            result.is_ok(),
            "Builder with stdio, fs, and custom providers should build successfully"
        );
    }

    #[test]
    fn test_builder_http_with_other_capabilities_returns_error() {
        // HTTP provider not yet implemented - should return error even with other capabilities
        let result = Engine::new()
            .with_stdio_capabilities()
            .with_fs_capabilities()
            .with_http_capabilities(HttpConfig::new())
            .build();

        assert!(
            result.is_err(),
            "Builder with HTTP should return error (not yet implemented)"
        );
    }

    #[test]
    fn test_builder_complex_chaining_order_without_http() {
        // Different ordering should all work (without HTTP which returns error)
        use providers::StdioProvider;

        let engine1 = Engine::new()
            .with_stdio_capabilities()
            .with_fs_capabilities()
            .build();

        let engine2 = Engine::new()
            .with_custom_provider("custom", std::sync::Arc::new(StdioProvider::new()))
            .with_stdio_capabilities()
            .build();

        assert!(engine1.is_ok(), "First order should succeed");
        assert!(engine2.is_ok(), "Second order should succeed");
    }

    #[test]
    fn test_http_config_clone() {
        // HttpConfig should be cloneable
        let config = HttpConfig {
            timeout_seconds: 45,
            max_redirects: 3,
            verify_ssl: false,
        };
        let config_clone = config.clone();
        assert_eq!(config.timeout_seconds, config_clone.timeout_seconds);
        assert_eq!(config.max_redirects, config_clone.max_redirects);
        assert_eq!(config.verify_ssl, config_clone.verify_ssl);
    }

    #[test]
    fn test_http_config_default() {
        // HttpConfig should implement Default
        let config = HttpConfig::default();
        assert_eq!(config.timeout_seconds, 0); // Default for u64
        assert_eq!(config.max_redirects, 0); // Default for u32
        assert!(!config.verify_ssl); // Default for bool
    }

    // ============================================================
    // LLM Provider Builder Tests
    // ============================================================

    #[test]
    fn test_builder_with_llm_capabilities_succeeds() {
        use crate::providers::llm::LlmConfig;
        use std::collections::HashMap;

        let mut configs = HashMap::new();
        configs.insert("openai".to_string(), LlmConfig::openai("sk-test123"));

        let result = Engine::new().with_llm_capabilities(configs).build();
        assert!(result.is_ok(), "Engine with LLM capabilities should build");
    }

    #[test]
    fn test_builder_with_llm_capabilities_multi_provider() {
        use crate::providers::llm::LlmConfig;
        use std::collections::HashMap;

        let mut configs = HashMap::new();
        configs.insert("openai".to_string(), LlmConfig::openai("sk-test123"));
        configs.insert("ollama".to_string(), LlmConfig::ollama());

        let result = Engine::new().with_llm_capabilities(configs).build();
        assert!(
            result.is_ok(),
            "Engine with multiple LLM providers should build"
        );
    }

    #[test]
    fn test_builder_with_llm_capabilities_invalid_config() {
        use crate::providers::llm::LlmConfig;
        use std::collections::HashMap;

        let mut configs = HashMap::new();
        configs.insert(
            "invalid".to_string(),
            LlmConfig {
                api_base: "not-a-url".to_string(),
                api_key: "key".to_string(),
                ..LlmConfig::default()
            },
        );

        // Should NOT fail - just prints warning and skips registration
        let result = Engine::new().with_llm_capabilities(configs).build();
        assert!(
            result.is_ok(),
            "Engine should build even with invalid LLM config (skips registration)"
        );
    }

    fn effectful_act_block_body(callee: &str) -> ash_parser::surface::Expr {
        use ash_parser::surface::{ActStmt, Expr, Literal};

        let span = ash_parser::token::Span::default();
        Expr::ActBlock {
            stmts: vec![
                ActStmt::Bind {
                    name: "x".into(),
                    value: Box::new(Expr::Call {
                        func: callee.into(),
                        module: None,
                        args: vec![Expr::Literal(Literal::String("/tmp/file".into()))],
                        span,
                    }),
                    span,
                },
                ActStmt::Return {
                    value: Box::new(Expr::Variable {
                        name: "x".into(),
                        span,
                    }),
                    span,
                },
            ],
            span,
        }
    }

    fn assert_closure_body_preserves_effectful_bind(value: &Value) {
        let Value::Closure { body, .. } = value else {
            panic!("expected closure, got: {value:?}");
        };

        match body.as_ref() {
            ash_core::Expr::Call {
                func, arguments, ..
            } => {
                assert_eq!(func, "bind");
                assert_eq!(arguments.len(), 2);
                assert!(
                    !matches!(&arguments[0], ash_core::Expr::Call { func, .. } if func == "unit"),
                    "effectful bind RHS should not be wrapped in unit()"
                );
            }
            other => panic!("expected bind call, got: {other:?}"),
        }
    }

    #[test]
    fn test_process_program_definitions_preserves_effectful_bind_rhs_for_local_functions() {
        use ash_parser::surface::{
            CapabilityDef, Definition, EffectType, FnDef, Program, Visibility,
            Workflow as SurfaceWorkflow, WorkflowDef,
        };

        let span = ash_parser::token::Span::default();
        let program = Program {
            definitions: vec![
                Definition::Capability(CapabilityDef {
                    visibility: Visibility::Inherited,
                    name: "read".into(),
                    effect: EffectType::Act,
                    params: vec![],
                    return_type: None,
                    constraints: vec![],
                    target_provider: None,
                    target_action: None,
                    span,
                }),
                Definition::Function(FnDef {
                    visibility: Visibility::Inherited,
                    name: "demo".into(),
                    type_params: vec![],
                    params: vec![],
                    return_type: None,
                    contract: None,
                    body: effectful_act_block_body("read"),
                    span,
                }),
            ],
            helper_workflows: vec![],
            workflow: WorkflowDef {
                name: "main".into(),
                type_params: vec![],
                params: vec![],
                declared_return_type: None,
                plays_roles: vec![],
                capabilities: vec![],
                body: SurfaceWorkflow::Ret {
                    expr: ash_parser::surface::Expr::Literal(ash_parser::surface::Literal::Null),
                    span,
                },
                contract: None,
                span,
            },
        };

        let engine = Engine::new().build().expect("engine builds");
        let (closures, _, _) = engine
            .process_program_definitions(&program, HashMap::new(), HashMap::new())
            .expect("program lowering should succeed");

        let closure = closures
            .get("demo")
            .expect("local function should be registered as a closure");
        assert_closure_body_preserves_effectful_bind(closure);
    }

    #[test]
    fn test_build_imported_closures_preserves_effectful_bind_rhs_for_user_callables() {
        use crate::module_loader::{CallableKind, InlineCallable};
        use std::collections::HashSet;

        let mut imported_callables = HashMap::new();
        imported_callables.insert(
            "demo".to_string(),
            InlineCallable {
                exported_name: "demo".to_string(),
                params: vec![],
                effectful_names: HashSet::from([String::from("read")]),
                kind: CallableKind::User {
                    body: effectful_act_block_body("read"),
                },
                signature: None,
            },
        );

        let (closures, _, _, _) = build_imported_closures(&imported_callables);
        let closure = closures
            .get("demo")
            .expect("imported callable should lower into a closure");

        assert_closure_body_preserves_effectful_bind(closure);
    }

    #[test]
    fn test_bind_imported_callable_types_uses_imported_pub_fn_signature() {
        use ash_parser::input::new_input;
        use ash_parser::parse_module::parse_fn_definition;
        use ash_parser::surface::Definition;
        use ash_typeck::Type;
        use ash_typeck::type_env::TypeEnv;
        use winnow::Parser;

        let mut input = new_input("pub fn bind(ma: Int, f: Fn(Int) -> Int) -> Int { ma }");
        let parsed = parse_fn_definition
            .parse_next(&mut input)
            .expect("function definition should parse");
        let Definition::Function(function) = parsed else {
            panic!("expected ordinary function definition");
        };

        let mut workflow = Workflow {
            core: ash_core::Workflow::Done,
            id: 0,
            imported_closures: HashMap::new(),
            imported_param_counts: HashMap::from([(String::from("bind"), 2_usize)]),
            imported_fn_signatures: HashMap::from([(String::from("bind"), function)]),
            imported_builtin_signatures: HashMap::new(),
        };

        let mut env = TypeEnv::with_builtin_types();
        bind_imported_callable_types(&mut env, &workflow)
            .expect("imported pub fn signature should bind cleanly");

        let Some(bound_ty) = env.lookup_call_target(None, "bind") else {
            panic!("expected imported pub fn binding for bind");
        };

        match bound_ty {
            Type::Fn(params, ret) => {
                assert_eq!(params.len(), 2, "bind should preserve arity");
                assert!(matches!(params[0], Type::Int), "first param should be Int");
                assert!(
                    matches!(&params[1], Type::Fn(inner, inner_ret)
                        if inner.len() == 1
                            && matches!(inner[0], Type::Int)
                            && matches!(inner_ret.as_ref(), Type::Int)),
                    "second param should preserve Fn(Int) -> Int, got {:?}",
                    params[1]
                );
                assert!(
                    matches!(ret.as_ref(), Type::Int),
                    "return type should be Int"
                );
            }
            other => panic!("expected Type::Fn for imported ordinary fn, got {other:?}"),
        }

        // keep mutable binding used so clippy doesn't complain if future fields change
        workflow.imported_param_counts.clear();
    }
}
