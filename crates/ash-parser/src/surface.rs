//! Surface AST types for the Ash parser.
//!
//! This module defines the parsed surface syntax before lowering to Core IR.
//! Surface AST is more flexible and syntactic than Core IR, preserving
//! all source-level constructs with full span information.

use std::fmt;

use ash_core::Kind;

use crate::token::Span;

/// A name/identifier in the source code.
pub type Name = Box<str>;

/// Explicit source kind annotation preserved at parser-audited binder sites.
#[derive(Debug, Clone, PartialEq)]
pub struct KindAnnotation {
    /// Parsed kind syntax, such as `*` or `* -> *`.
    pub kind: Kind,
    /// Source span covering the kind annotation only.
    pub span: Span,
}

/// Crate root metadata for cross-crate dependency management.
///
/// This struct represents the crate identity and dependencies declared
/// at the beginning of a crate root file.
#[derive(Debug, Clone, PartialEq)]
pub struct CrateRootMetadata {
    /// The name of this crate
    pub crate_name: Box<str>,
    /// Declared external dependencies
    pub dependencies: Vec<DependencyDecl>,
    /// Source span covering the entire metadata
    pub span: Span,
}

/// A dependency declaration for external crate roots.
///
/// Syntax: `dependency <alias> from "<path>";`
#[derive(Debug, Clone, PartialEq)]
pub struct DependencyDecl {
    /// The alias used to refer to this dependency in imports
    pub alias: Box<str>,
    /// The filesystem path to the dependency's crate root
    pub root_path: Box<str>,
    /// Source span covering this declaration
    pub span: Span,
}

/// A program consists of definitions, optional helper workflows, and a main workflow.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// Top-level definitions (capabilities, policies, roles, functions)
    pub definitions: Vec<Definition>,
    /// Helper workflow definitions preceding the main entry workflow.
    ///
    /// These are registered as callable targets at runtime so that
    /// `Workflow::Call` can dispatch to them by name.
    pub helper_workflows: Vec<WorkflowDef>,
    /// The main workflow definition (entry point)
    pub workflow: WorkflowDef,
}

/// The authoritative file-level parse result for a `.ash` source file.
///
/// Every `.ash` source file parses as a `ModuleFile` containing a collection
/// of module items (definitions, module declarations, and an optional workflow).
/// `Program` is reserved for entry-point loading/validation only.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ModuleFile {
    /// Top-level definitions in this file
    pub definitions: Vec<Definition>,
    /// Module declarations (`mod foo;`, `mod foo { ... }`)
    pub module_decls: Vec<crate::module::ModuleDecl>,
    /// Optional workflow (entry point)
    pub workflow: Option<WorkflowDef>,
    /// Source span covering the entire file
    pub span: Span,
    /// Captured comment trivia.
    pub comments: crate::parse_utils::CommentTable,
    /// Optional filesystem path of the source file.
    pub path: Option<Box<str>>,
}

/// A top-level definition.
#[derive(Debug, Clone, PartialEq)]
pub enum Definition {
    /// Source-level notation declaration.
    Notation(NotationDecl),
    /// Parser-first expression macro declaration.
    Macro(MacroDef),
    /// Capability definition
    Capability(CapabilityDef),
    /// Capability interface definition
    CapabilityInterface(CapabilityInterfaceDef),
    /// Capability implementation recipe definition
    CapabilityImplementation(CapabilityImplementationDef),
    /// Resource type definition
    ResourceType(ResourceTypeDef),
    /// Ordinary type declaration
    Type(TypeDef),
    /// Explicit named data-kind promotion declaration
    DataKind(DataKindDef),
    /// Module-level type function declaration
    TypeFn(TypeFnDef),
    /// Explicit named type-level proposition predicate declaration
    PropositionPredicate(PropositionPredicateDecl),
    /// Policy definition
    Policy(PolicyDef),
    /// Role definition
    Role(RoleDef),
    /// Proxy definition
    Proxy(ProxyDef),
    /// Interface definition
    Interface(InterfaceDef),
    /// Interface impl definition
    Impl(ImplDef),
    /// Pure function definition
    Function(FnDef),
    /// Builtin function definition (no Ash-level body)
    BuiltinFn(BuiltinFnDef),
    /// Sealed type-level domain declaration
    SealedDomain(SealedDomainDef),
    /// Law definition
    Law(LawDef),
    /// Proof definition
    Proof(ProofDef),
}

/// Parser-first expression macro declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroDef {
    /// Visibility modifier retained for scope/export validation.
    pub visibility: Visibility,
    /// Local macro name.
    pub name: Name,
    /// Macro parameter names.
    pub params: Vec<Name>,
    /// Optional syntax-phase typed macro signature carrier.
    pub typed_signature: Option<MacroTypeSignatureSummary>,
    /// Parsed expression template body.
    pub body: Expr,
    /// Source span covering the complete declaration.
    pub span: Span,
}

/// Syntax-phase identity for a macro declaration or imported macro summary.
///
/// This is tooling/expansion metadata only. It is deliberately separate from
/// [`CallableDeclarationIdentity`] and must not grant runtime callability,
/// effect rows, provider authority, contracts, or proof evidence.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MacroDeclarationIdentity {
    /// Where the identity was proven.
    pub origin: MacroIdentityOrigin,
    /// Name visible at the current use site. Imported aliases change this field
    /// without changing the exported origin name.
    pub local_name: Name,
    /// Source span covering the declaration in the source that produced the
    /// identity, when available.
    pub origin_span: Span,
    /// Arity included so summaries can reject stale or malformed carriers.
    pub param_count: usize,
}

/// Provenance for a syntax-phase macro identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MacroIdentityOrigin {
    /// Declaration proven in the current parsed file/module.
    Local,
    /// Declaration summarized from an imported module.
    Imported {
        /// Module path that exported the macro summary.
        module_path: Box<str>,
        /// Name exported by the origin module before local import aliasing.
        exported_name: Name,
    },
}

impl MacroDeclarationIdentity {
    /// Build a local syntax-phase macro identity.
    #[must_use]
    pub fn local(name: Name, span: Span, param_count: usize) -> Self {
        Self {
            origin: MacroIdentityOrigin::Local,
            local_name: name,
            origin_span: span,
            param_count,
        }
    }

    /// Build an imported syntax-phase macro identity from an export summary and
    /// the name visible at the current import site.
    #[must_use]
    pub fn imported(summary: &MacroSummary, local_name: Name) -> Self {
        let mut identity = summary.identity.clone();
        identity.local_name = local_name;
        identity
    }
}

/// Callable declaration identity for ordinary functions and builtin functions.
///
/// Unlike [`MacroDeclarationIdentity`], this names runtime-callable source
/// declarations. Macro identities must never be coerced into this shape.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CallableDeclarationIdentity {
    /// Callable name visible in the current file.
    pub name: Name,
    /// Callable declaration kind.
    pub kind: CallableDeclarationKind,
    /// Source span covering the declaration.
    pub origin_span: Span,
    /// Declared parameter count.
    pub param_count: usize,
}

/// Runtime-callable declaration kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallableDeclarationKind {
    /// User-defined Ash function.
    Function,
    /// Bodyless builtin function.
    BuiltinFn,
}

/// Syntax-phase summary for an importable public macro declaration.
///
/// Macro summaries are not callable summaries: they carry only expansion-phase
/// metadata and must not grant rows, authority, contracts, failures, proof
/// evidence, providers, or runtime effects.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroSummary {
    /// Module path that produced the summary.
    pub module_path: Box<str>,
    /// Source-visible macro name.
    pub name: Name,
    /// Canonical syntax-phase identity for this public macro summary.
    pub identity: MacroDeclarationIdentity,
    /// Visibility retained from the source declaration.
    pub visibility: Visibility,
    /// Macro parameter names.
    pub params: Vec<Name>,
    /// Accepted invocation input shape.
    pub input_kind: MacroInputKind,
    /// Expansion output shape.
    pub output_kind: MacroOutputKind,
    /// Conservative template identity used to detect malformed or stale carriers.
    pub template_fingerprint: MacroTemplateFingerprint,
    /// Hygiene policy guaranteed by the summarized template.
    pub hygiene_policy: MacroHygienePolicy,
    /// Optional syntax-phase typed macro signature metadata.
    pub typed_signature: Option<MacroTypeSignatureSummary>,
    /// Source span covering the macro declaration.
    pub origin_span: Span,
}

/// Syntax accepted by a macro summary's invocation site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MacroInputKind {
    /// Parenthesized expression arguments, the Phase 172 executable subset.
    ExprArgs,
    /// Delimiter-preserving token trees. Later Phase 173 tasks populate this.
    TokenTree { delimiter: MacroDelimiter },
}

/// Syntax produced by macro expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MacroOutputKind {
    /// Parsed expression output.
    Expr,
    /// Token-tree output that remains syntax-phase metadata.
    TokenTree,
    /// Token-tree output that must cross one audited surface reparse boundary.
    ReparseExpr,
}

/// Conservative hygiene policy for a summarized macro template.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MacroHygienePolicy {
    /// Binder-free expression substitution only.
    BinderFreeExpression,
}

/// Stable-enough local fingerprint for a parsed expression-template summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacroTemplateFingerprint {
    /// Number of declared macro parameters.
    pub param_count: usize,
    /// Span of the parsed template body.
    pub body_span: Span,
}

/// Syntax-phase typed macro signature carrier.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroTypeSignatureSummary {
    /// Macro parameter type annotations, aligned with `MacroDef::params`.
    pub param_types: Vec<Option<Type>>,
    /// Optional macro result type annotation.
    pub return_type: Option<Type>,
    /// Source span covering the signature.
    pub span: Span,
}

/// Source-level notation declaration parsed before expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotationDecl {
    /// Visibility modifier for exported notation surfaces.
    pub visibility: Visibility,
    /// Declared fixity and precedence.
    pub fixity: NotationFixity,
    /// Raw notation pattern as written in source.
    pub pattern: NotationPattern,
    /// Callable target path. Resolution/type checking is deferred to later phases.
    pub target: CallablePath,
    /// Source span covering the complete declaration.
    pub span: Span,
}

/// Notation fixity declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotationFixity {
    /// Prefix notation, optionally with precedence.
    Prefix { precedence: Option<u16> },
    /// Binary infix notation with associativity and required precedence.
    Infix {
        associativity: NotationAssociativity,
        precedence: u16,
    },
    /// Suffix notation, optionally with precedence.
    Suffix { precedence: Option<u16> },
    /// Mixfix notation; binder-introducing semantics remain deferred.
    Mixfix,
}

/// Infix associativity for notation declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotationAssociativity {
    /// Left-associative infix.
    Left,
    /// Right-associative infix.
    Right,
    /// Non-associative infix.
    Nonassoc,
}

/// Source-preserving notation pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotationPattern {
    /// Raw pattern text between the fixity declaration and target separator.
    pub raw: Box<str>,
    /// Symbolic tokens found in the pattern, preserving spelling/spans for diagnostics.
    pub tokens: Vec<RawOperatorToken>,
    /// Source span covering the raw pattern.
    pub span: Span,
}

/// A callable path used as the target of notation expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallablePath {
    /// Optional module qualifier.
    pub module: Option<Name>,
    /// Callable name.
    pub name: Name,
    /// Source span covering the callable path.
    pub span: Span,
}

/// Explicit named data-kind promotion declaration parsed as surface syntax.
///
/// Syntax: `[pub] data kind <KindName> from type <SourceAdt>;`
#[derive(Debug, Clone, PartialEq)]
pub struct DataKindDef {
    /// Visibility modifier retained for downstream validation.
    pub visibility: Visibility,
    /// Source-visible promoted kind name.
    pub name: Name,
    /// Source ADT name whose constructors are promoted by this declaration.
    pub source_adt: Name,
    /// Source span covering the complete declaration.
    pub span: Span,
}

/// An ordinary type declaration parsed as a file/module surface definition.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    /// Visibility modifier (pub, pub(crate), etc.)
    pub visibility: Visibility,
    /// Name of the declared type
    pub name: Name,
    /// Generic type parameters (e.g., `<T, U>`)
    pub params: Vec<Name>,
    /// Type body (struct, enum, or alias)
    pub body: TypeBody,
    /// Whether this type is declared as runtime-managed builtin substrate.
    pub builtin: bool,
    /// Source span covering the declaration.
    pub span: Span,
    /// Optional filesystem path of the source file/module that produced this declaration.
    pub source: Option<Box<str>>,
}

/// A module-level type function declaration parsed as raw surface syntax.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeFnDef {
    /// Visibility modifier retained for downstream SPEC-F semantic validation.
    pub visibility: Visibility,
    /// Name of the type function.
    pub name: Name,
    /// Header parameters.
    pub params: Vec<TypeFnParam>,
    /// Declared return type.
    pub return_type: Type,
    /// Optional decreasing parameter clause.
    pub decreases: Option<TypeFnDecreases>,
    /// Optional raw type-level proposition requirements after `where`.
    pub proposition_tail: Option<PropositionTail>,
    /// Ordered case equations.
    pub equations: Vec<TypeFnEquation>,
    /// Source span covering the declaration header through the return type.
    pub header_span: Span,
    /// Source span covering the entire declaration.
    pub span: Span,
}

/// A raw proposition tail introduced by `where` on enabled declaration surfaces.
#[derive(Debug, Clone, PartialEq)]
pub struct PropositionTail {
    /// Ordered source clauses in the proposition list.
    pub clauses: Vec<PropositionClause>,
    /// Optional `row` block parsed from a `where row { ... }` section.
    pub row: Option<PropositionWhereRow>,
    /// Source span covering the `where` keyword.
    pub where_span: Span,
    /// Source span covering the complete tail.
    pub span: Span,
}

/// A callable-row block inside a proposition-tail `where` clause.
#[derive(Debug, Clone, PartialEq)]
pub struct PropositionWhereRow {
    /// Parsed row entries.
    pub row: ComputationRow,
    /// Source span covering the `row` keyword.
    pub row_keyword_span: Span,
    /// Source span covering the complete `row { ... }` block.
    pub span: Span,
}

/// A source-preserving computation row.
#[derive(Debug, Clone, PartialEq)]
pub struct ComputationRow {
    /// Row entries in source order.
    pub items: Vec<ComputationRowItem>,
    /// Source span covering the complete row braces content.
    pub span: Span,
}

/// Separator used before the final segment of an operation row path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowPathSeparator {
    /// Dot separator, e.g. `fs.read`.
    Dot,
    /// Double-colon separator, e.g. `PosixFs::read`.
    DoubleColon,
}

/// A typed row entry family.
#[derive(Debug, Clone, PartialEq)]
pub enum ComputationRowItem {
    /// A default operation entry represented by a qualified path.
    ///
    /// Examples: `fs.read`, `PosixFs::read`
    Operation {
        /// Operation path segments.
        path: Vec<Name>,
        /// Source separator before the final path segment, when present.
        separator: Option<RowPathSeparator>,
        /// Full source span for this item.
        span: Span,
    },
    /// Whole-row variable entry, e.g. `{r}`.
    WholeRow {
        /// Row variable name.
        variable: Name,
        /// Full source span for this item.
        span: Span,
    },
    /// Resource family entry.
    Resource {
        /// Resource path.
        path: Vec<Name>,
        /// Optional mode token.
        mode: Option<Name>,
        /// Full source span for this item.
        span: Span,
    },
    /// Role family entry.
    Role {
        /// Role path.
        path: Vec<Name>,
        /// Full source span for this item.
        span: Span,
    },
    /// Policy family entry.
    Policy {
        /// Policy path.
        path: Vec<Name>,
        /// Full source span for this item.
        span: Span,
    },
    /// Channel family entry.
    Channel {
        /// Optional mode token.
        mode: Option<Name>,
        /// Channel path.
        path: Vec<Name>,
        /// Optional message payload.
        payload: Option<Type>,
        /// Full source span for this item.
        span: Span,
    },
    /// Process family entry (`proc`/`process`) with optional operation.
    Process {
        /// Keyword token (`proc` or `process`).
        keyword: Name,
        /// Optional operation token, e.g. `spawn`.
        operation: Option<Name>,
        /// Full source span for this item.
        span: Span,
    },
    /// Failure family entry.
    Fail {
        /// Optional failure path.
        path: Option<Vec<Name>>,
        /// Full source span for this item.
        span: Span,
    },
    /// Evidence family entry.
    Evidence {
        /// Evidence path.
        path: Vec<Name>,
        /// Full source span for this item.
        span: Span,
    },
    /// Group family entry.
    Group {
        /// Group path.
        path: Vec<Name>,
        /// Full source span for this item.
        span: Span,
    },
    /// Open-row tail entry.
    Tail {
        /// Tail variable name.
        variable: Name,
        /// Full source span for this item.
        span: Span,
    },
}

/// A raw source proposition clause with its complete source span.
#[derive(Debug, Clone, PartialEq)]
pub struct PropositionClause {
    /// Clause payload preserved without semantic resolution.
    pub kind: PropositionClauseKind,
    /// Source span covering the complete clause.
    pub span: Span,
}

/// Raw proposition clause variants parsed by ash-parser.
#[derive(Debug, Clone, PartialEq)]
pub enum PropositionClauseKind {
    /// Type-level equality: `lhs == rhs`.
    Equality {
        /// Raw left operand.
        lhs: Type,
        /// Raw right operand.
        rhs: Type,
        /// Source span covering `==`.
        op_span: Span,
    },
    /// Type-level disequality: `lhs != rhs`.
    Disequality {
        /// Raw left operand.
        lhs: Type,
        /// Raw right operand.
        rhs: Type,
        /// Source span covering `!=`.
        op_span: Span,
    },
    /// Interface-bound proposition: `subject: Interface<...>`.
    InterfaceBound {
        /// Raw subject type expression.
        subject: Type,
        /// Raw interface type application; parser does not resolve the name.
        interface: Type,
        /// Source span covering `:`.
        colon_span: Span,
    },
    /// Named predicate proposition: `Predicate<args...>` or `Predicate`.
    NamedPredicate {
        /// Source-visible predicate name.
        name: Name,
        /// Source span covering the predicate name.
        name_span: Span,
        /// Raw type argument list.
        args: Vec<Type>,
    },
}

/// Explicit named proposition predicate declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct PropositionPredicateDecl {
    /// Visibility modifier.
    pub visibility: Visibility,
    /// Predicate name.
    pub name: Name,
    /// Explicitly annotated predicate parameters.
    pub params: Vec<PropositionPredicateParam>,
    /// Source span covering the declaration.
    pub span: Span,
}

/// Parameter in a named proposition predicate declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct PropositionPredicateParam {
    /// Parameter name.
    pub name: Name,
    /// Raw parser-owned domain/type annotation.
    pub domain: Type,
    /// Explicit kind annotation when the parameter is constructor-kinded.
    pub kind: Option<KindAnnotation>,
    /// Source span covering `name: domain`.
    pub span: Span,
}

/// A named type-function parameter with its annotation and source span.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeFnParam {
    /// Parameter name.
    pub name: Name,
    /// Parameter type annotation.
    pub ty: Type,
    /// Explicit kind annotation when the parameter is constructor-kinded.
    pub kind: Option<KindAnnotation>,
    /// Source span covering `name: type`.
    pub span: Span,
}

/// A `decreases <param>` clause on a type function.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeFnDecreases {
    /// Declared decreasing parameter name.
    pub param: Name,
    /// Source span covering the clause.
    pub span: Span,
}

/// A raw type-function case equation.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeFnEquation {
    /// Case head name after `case`.
    pub head: Name,
    /// Source span covering the case head name.
    pub head_span: Span,
    /// Raw type-level patterns.
    pub patterns: Vec<TypePattern>,
    /// Raw RHS type expression.
    pub result: Type,
    /// Source span covering the RHS type expression.
    pub result_span: Span,
    /// Source span covering the whole equation.
    pub span: Span,
}

/// Raw type-level pattern syntax for type-function case equations.
#[derive(Debug, Clone, PartialEq)]
pub enum TypePattern {
    /// Constructor pattern: `Name` or `Name<...>`.
    Constructor {
        /// Constructor name spelling.
        name: Name,
        /// Nested type-level patterns.
        args: Vec<TypePattern>,
        /// Source span covering the pattern.
        span: Span,
    },
    /// Syntactic lowercase bare-name pattern.
    ///
    /// This is raw parser syntax, not final semantic resolution. Later type-checking may
    /// reinterpret this spelling as a lowercase sealed-domain marker constructor when the
    /// expected sealed-domain constructor namespace resolves it that way.
    Var {
        /// Binding-or-lowercase-constructor candidate name.
        name: Name,
        /// Source span covering the pattern.
        span: Span,
    },
    /// Wildcard pattern `_`.
    Wildcard {
        /// Source span covering the wildcard.
        span: Span,
    },
}

/// Body of an ordinary type declaration.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeBody {
    /// Struct type: `type Point = { x: Int, y: Int };`
    Struct(Vec<TypeField>),
    /// Enum type: `type Status = Pending | Processing;`
    Enum(Vec<VariantDef>),
    /// Type alias: `type Name = String;`
    Alias(Type),
}

/// A named field in a type declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeField {
    /// Field name
    pub name: Name,
    /// Field type
    pub ty: Type,
    /// Source span. When fine-grained spans are unavailable, this is the declaration span.
    pub span: Span,
}

/// Variant definition for ordinary enum type declarations.
#[derive(Debug, Clone, PartialEq)]
pub struct VariantDef {
    /// Variant name
    pub name: Name,
    /// Named record fields, if present.
    pub fields: Vec<TypeField>,
    /// Explicit payload shape for the variant.
    pub payload: VariantPayload,
    /// Source span. When fine-grained spans are unavailable, this is the declaration span.
    pub span: Span,
}

/// Explicit payload shape for enum variants.
#[derive(Debug, Clone, PartialEq)]
pub enum VariantPayload {
    /// Unit variant with no payload
    Unit,
    /// Record variant with named fields
    Record(Vec<TypeField>),
    /// Tuple variant with positional items
    Tuple(Vec<Type>),
}

/// Sealed type-level domain declaration.
///
/// Syntax: `[pub] sealed type domain Name { Constructor; ... }`
#[derive(Debug, Clone, PartialEq)]
pub struct SealedDomainDef {
    /// Visibility modifier (`pub`, `pub(crate)`, etc.)
    pub visibility: Visibility,
    /// Domain name
    pub name: Name,
    /// Marker constructors within this sealed domain
    pub constructors: Vec<DomainConstructor>,
    /// Source span covering the entire declaration
    pub span: Span,
}

/// Marker constructor within a sealed domain.
///
/// Syntax: `Name` or `Name<field: Slot, ...>;`
#[derive(Debug, Clone, PartialEq)]
pub struct DomainConstructor {
    /// Constructor name
    pub name: Name,
    /// Named fields (empty for unit constructors)
    pub fields: Vec<DomainField>,
    /// Source span covering the constructor
    pub span: Span,
}

/// Field in a marker constructor.
///
/// Syntax: `name: Slot`
#[derive(Debug, Clone, PartialEq)]
pub struct DomainField {
    /// Field name
    pub name: Name,
    /// Field slot annotation
    pub slot: DomainSlot,
    /// Source span covering the field
    pub span: Span,
}

/// Allowed field slot annotations in a sealed domain constructor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainSlot {
    /// Unconstrained `Type`-kind slot
    Type,
    /// Constrained to a named domain reference
    DomainRef(Name),
}

/// A named resource type definition.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceTypeDef {
    /// Visibility modifier (pub, pub(crate), etc.)
    pub visibility: Visibility,
    /// Name of the resource type
    pub name: Name,
    /// Named fields declared by this resource type
    pub fields: Vec<ResourceField>,
    /// Source span
    pub span: Span,
}

/// A field in a resource type definition.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceField {
    /// Field name
    pub name: Name,
    /// Field type
    pub ty: Type,
    /// Source span
    pub span: Span,
}

/// A named capability implementation recipe.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityImplementationDef {
    /// Visibility modifier (pub, pub(crate), etc.)
    pub visibility: Visibility,
    /// Name of this implementation recipe
    pub name: Name,
    /// Target capability interface name
    pub interface: Name,
    /// Explicit dependencies required at binding/admission time
    pub dependencies: Vec<CapabilityImplementationDependency>,
    /// Operation implementations provided by this recipe
    pub operations: Vec<CapabilityImplementationOperation>,
    /// Source span
    pub span: Span,
}

/// An explicit dependency required by a capability implementation recipe.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityImplementationDependency {
    /// Dependency kind
    pub kind: CapabilityImplementationDependencyKind,
    /// Dependency binding name visible to operation bodies
    pub name: Name,
    /// Dependency type/interface/config type name
    pub ty: Type,
    /// Source span
    pub span: Span,
}

/// Capability implementation dependency forms parsed without resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapabilityImplementationDependencyKind {
    /// `requires resource name: Type`
    Resource,
    /// `requires capability name: Interface`
    Capability,
    /// `requires config name: Type`
    Config,
}

/// An operation body inside a capability implementation recipe.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityImplementationOperation {
    /// Operation effect mode
    pub mode: CapabilityOperationMode,
    /// Operation name
    pub name: Name,
    /// Named operation parameters
    pub params: Vec<Param>,
    /// Required return type
    pub return_type: Type,
    /// Body expression/block preserved for later semantics
    pub body: Expr,
    /// Source span
    pub span: Span,
}

/// A capability interface definition.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityInterfaceDef {
    /// Visibility modifier (pub, pub(crate), etc.)
    pub visibility: Visibility,
    /// Name of the capability interface
    pub name: Name,
    /// Operation signatures exposed by this interface
    pub operations: Vec<CapabilityOperationSig>,
    /// Source span
    pub span: Span,
}

/// A capability interface operation signature.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityOperationSig {
    /// Operation effect mode
    pub mode: CapabilityOperationMode,
    /// Operation name
    pub name: Name,
    /// Named operation parameters
    pub params: Vec<Param>,
    /// Required return type
    pub return_type: Type,
    /// Source span
    pub span: Span,
}

/// Operation modes supported by capability interfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapabilityOperationMode {
    /// Read-only observation
    Observe,
    /// Effectful execution
    Execute,
}

impl CapabilityOperationMode {
    /// Check whether this mode is `observe`.
    pub fn is_observe(self) -> bool {
        matches!(self, Self::Observe)
    }

    /// Check whether this mode is `execute`.
    pub fn is_execute(self) -> bool {
        matches!(self, Self::Execute)
    }
}

/// A pure function definition.
///
/// Syntax: `[pub] fn <name>[<type_params>](<params>) [-> <return_type>] [contract*] { <body> }`
#[derive(Debug, Clone, PartialEq)]
pub struct FnDef {
    /// Visibility modifier (pub, pub(crate), etc.)
    pub visibility: Visibility,
    /// Function name
    pub name: Name,
    /// Generic type parameters (e.g., `<T, U>` or `<F : * -> *>`)
    pub type_params: Vec<TypeParam>,
    /// Function parameters with name and type
    pub params: Vec<Param>,
    /// Optional return type annotation
    pub return_type: Option<Type>,
    /// Optional raw type-level proposition requirements after `where`.
    pub proposition_tail: Option<PropositionTail>,
    /// Optional contract (requires/ensures)
    pub contract: Option<Contract>,
    /// Function body (a block expression)
    pub body: Expr,
    /// Source span
    pub span: Span,
}

/// A builtin function definition (runtime-provided, no Ash-level body).
///
/// Syntax: `[pub] builtin fn <name>[<type_params>](<params>) -> <return_type>;`
#[derive(Debug, Clone, PartialEq)]
pub struct BuiltinFnDef {
    /// Visibility modifier (pub, pub(crate), etc.)
    pub visibility: Visibility,
    /// Function name
    pub name: Name,
    /// Generic type parameters (e.g., `<T, U>` or `<F : * -> *>`)
    pub type_params: Vec<TypeParam>,
    /// Function parameters with name and type
    pub params: Vec<Param>,
    /// Required return type annotation
    pub return_type: Type,
    /// Optional raw type-level proposition requirements after `where`.
    pub proposition_tail: Option<PropositionTail>,
    /// Source span
    pub span: Span,
}

/// A capability definition.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityDef {
    /// Visibility modifier (pub, pub(crate), etc.)
    pub visibility: Visibility,
    /// Name of the capability
    pub name: Name,
    /// Effect type of the capability
    pub effect: EffectType,
    /// Parameters to the capability
    pub params: Vec<Param>,
    /// Return type (optional)
    pub return_type: Option<Type>,
    /// Constraints on the capability
    pub constraints: Vec<Constraint>,
    /// Target provider name for operational capabilities (optional)
    pub target_provider: Option<Name>,
    /// Target action name for operational capabilities (optional)
    pub target_action: Option<Name>,
    /// Source span
    pub span: Span,
}

impl CapabilityDef {
    /// Check if this capability has explicit target metadata.
    pub fn has_target(&self) -> bool {
        self.target_provider.is_some() && self.target_action.is_some()
    }

    /// Get the target pair for this capability if defined.
    pub fn target(&self) -> Option<(Name, Name)> {
        match (&self.target_provider, &self.target_action) {
            (Some(provider), Some(action)) => Some((provider.clone(), action.clone())),
            _ => None,
        }
    }
}

/// A policy definition.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyDef {
    /// Name of the policy
    pub name: Name,
    /// Type parameters for generic policies
    pub type_params: Vec<Name>,
    /// Fields of the policy
    pub fields: Vec<PolicyField>,
    /// Where clause for invariants
    pub where_clause: Option<Expr>,
    /// Source span
    pub span: Span,
}

/// A field in a policy definition.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyField {
    /// Name of the field
    pub name: Name,
    /// Type of the field
    pub ty: Type,
    /// Default value (optional)
    pub default: Option<Expr>,
    /// Source span
    pub span: Span,
}

/// A policy instance (usage of a policy).
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyInstance {
    /// Name of the policy being instantiated
    pub name: Name,
    /// Field initializations
    pub fields: Vec<(Name, Expr)>,
    /// Source span
    pub span: Span,
}

/// A role definition.
#[derive(Debug, Clone, PartialEq)]
pub struct RoleDef {
    /// Name of the role
    pub name: Name,
    /// Capabilities granted to this role (with optional constraints)
    pub capabilities: Vec<CapabilityDecl>,
    /// Named obligations exposed by this role
    pub obligations: Vec<Name>,
    /// Source span
    pub span: Span,
}

/// A capability reference for proxy declarations.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityRef {
    /// The capability name
    pub name: Name,
    /// Optional channel (e.g., `requests:approval_request`)
    pub channel: Option<Name>,
}

/// A proxy definition.
#[derive(Debug, Clone, PartialEq)]
pub struct ProxyDef {
    /// Visibility modifier
    pub visibility: Visibility,
    /// Name of the proxy
    pub name: Name,
    /// Role this proxy handles
    pub role: Name,
    /// Capabilities this proxy observes (reads from)
    pub observes: Vec<CapabilityRef>,
    /// Capabilities this proxy receives (handles)
    pub receives: Vec<CapabilityRef>,
    /// The proxy workflow body
    pub body: Workflow,
    /// Source span
    pub span: Span,
}

/// An associated type declaration inside an interface.
#[derive(Debug, Clone, PartialEq)]
pub struct AssociatedTypeDecl {
    /// Associated type name
    pub name: Name,
    /// Ordinary associated type vs sealed associated family metadata.
    pub kind: AssociatedTypeKind,
    /// Source span
    pub span: Span,
}

/// Parsed associated member declaration kind.
#[derive(Debug, Clone, PartialEq)]
pub enum AssociatedTypeKind {
    /// Ordinary SPEC-035 associated type declaration.
    Ordinary,
    /// Sealed associated family declaration with raw parser-owned metadata.
    SealedFamily {
        /// Mandatory result-domain annotation after `:`.
        result_domain: Type,
        /// Optional raw decreases clause.
        decreases: Option<AssociatedFamilyDecreases>,
        /// Source span covering the sealed family declaration.
        span: Span,
    },
}

/// Raw `decreases Param` clause for a sealed associated family.
#[derive(Debug, Clone, PartialEq)]
pub struct AssociatedFamilyDecreases {
    /// Parameter named by the decreases clause.
    pub param: Name,
    /// Source span covering the decreases clause.
    pub span: Span,
}

/// An associated type binding inside an impl block.
#[derive(Debug, Clone, PartialEq)]
pub struct AssociatedTypeBinding {
    /// Associated type name
    pub name: Name,
    /// Bound type expression
    pub ty: Type,
    /// Source span
    pub span: Span,
}

/// A where-bound clause `T: Interface`.
#[derive(Debug, Clone, PartialEq)]
pub struct WhereBound {
    /// Type parameter name
    pub param: Name,
    /// Interface bound name
    pub bound: Name,
    /// Source span
    pub span: Span,
}

/// An interface-level evidence constraint declared after an interface `where`.
///
/// This preserves the surface shape `subject: InterfaceApplication` without
/// assigning impl-scheme semantics to the declaration. Downstream type checking
/// validates that the subject names an interface parameter and that the target
/// names available required evidence.
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceEvidenceConstraint {
    /// Raw constraint subject before `:`.
    pub subject: Type,
    /// Raw required interface evidence application after `:`.
    pub interface: Type,
    /// Source span covering the `:` separator.
    pub colon_span: Span,
    /// Source span covering the complete constraint.
    pub span: Span,
}

/// A law declaration inside an interface.
#[derive(Debug, Clone, PartialEq)]
pub struct LawDef {
    /// Law name
    pub name: Name,
    /// Law parameters (name: type pairs)
    pub params: Vec<Param>,
    /// Optional constraints
    pub constraints: Vec<Constraint>,
    /// Proposition expression
    pub proposition: Expr,
    /// Source span
    pub span: Span,
}

/// A proof declaration inside an impl block or at module scope.
#[derive(Debug, Clone, PartialEq)]
pub struct ProofDef {
    /// Proof name
    pub name: Name,
    /// Proof parameters (name: type pairs)
    pub params: Vec<Param>,
    /// Optional constraints
    pub constraints: Vec<Constraint>,
    /// Proof body
    pub body: ProofBody,
    /// Source span
    pub span: Span,
}

/// Body of a proof declaration.
#[derive(Debug, Clone, PartialEq)]
pub enum ProofBody {
    /// `by_definition`
    ByDefinition,
    /// `by test "test_name"` or `by test authored "test_name"`
    ByTest { test_name: String },
    /// `by test property` or `by test quickcheck` — law proposition is executed over generated bindings.
    /// Optional strategy overrides for parameters.
    ByTestProperty {
        /// Source-visible strategy overrides: parameter name -> strategy expression.
        strategies: Vec<PropertyStrategyBinding>,
    },
    /// `by test small_world` — law proposition is executed over finite worlds.
    ByTestSmallWorld,
    /// Explicit proof term (future)
    Expr(Expr),
}

/// A strategy override binding in a `by test property` proof body.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyStrategyBinding {
    /// Parameter name being overridden (e.g., `x` in `x <- expr`).
    pub param_name: String,
    /// Strategy expression (e.g., `qc::int::positive()`).
    pub strategy_expr: Expr,
}

/// An interface definition.
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceDef {
    /// Visibility modifier
    pub visibility: Visibility,
    /// Interface name
    pub name: Name,
    /// Interface type parameters
    pub type_params: Vec<InterfaceTypeParam>,
    /// Interface-level required evidence constraints.
    pub evidence_constraints: Vec<InterfaceEvidenceConstraint>,
    /// Associated type declarations
    pub associated_types: Vec<AssociatedTypeDecl>,
    /// Declared method signatures
    pub methods: Vec<InterfaceMethodSig>,
    /// Law declarations
    pub laws: Vec<LawDef>,
    /// Source span
    pub span: Span,
}

/// Raw interface/impl type parameter with an optional domain annotation.
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceTypeParam {
    /// Parameter name.
    pub name: Name,
    /// Optional raw domain annotation after `:`.
    pub domain: Option<Type>,
    /// Explicit kind annotation when the parameter is constructor-kinded.
    pub kind: Option<KindAnnotation>,
    /// Source span.
    pub span: Span,
}

impl AsRef<str> for InterfaceTypeParam {
    fn as_ref(&self) -> &str {
        self.name.as_ref()
    }
}

impl From<&str> for InterfaceTypeParam {
    fn from(value: &str) -> Self {
        Self {
            name: value.into(),
            domain: None,
            kind: None,
            span: Span::default(),
        }
    }
}

impl From<String> for InterfaceTypeParam {
    fn from(value: String) -> Self {
        Self {
            name: value.into(),
            domain: None,
            kind: None,
            span: Span::default(),
        }
    }
}

impl fmt::Display for InterfaceTypeParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name.as_ref())
    }
}

/// A method signature declared inside an interface.
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceMethodSig {
    /// Method name
    pub name: Name,
    /// Positional parameter types
    pub params: Vec<Type>,
    /// Return type
    pub return_type: Type,
    /// Source span
    pub span: Span,
}

/// An explicit interface implementation.
#[derive(Debug, Clone, PartialEq)]
pub struct ImplDef {
    /// Visibility modifier
    pub visibility: Visibility,
    /// Implemented interface name
    pub interface: Name,
    /// Generic type parameters (e.g., `<T>`)
    pub type_params: Vec<InterfaceTypeParam>,
    /// Concrete type arguments for the interface head
    pub type_args: Vec<Type>,
    /// Where bounds
    pub where_bounds: Vec<WhereBound>,
    /// Associated type bindings
    pub associated_type_bindings: Vec<AssociatedTypeBinding>,
    /// Implemented methods
    pub methods: Vec<ImplMethodDef>,
    /// Proof declarations
    pub proofs: Vec<ProofDef>,
    /// Source span
    pub span: Span,
}

/// A method implementation inside an impl block.
#[derive(Debug, Clone, PartialEq)]
pub struct ImplMethodDef {
    /// Method name
    pub name: Name,
    /// Parameter names (multi-parameter per SPEC-032)
    pub params: Vec<Name>,
    /// Method body expression preserved at the parser surface
    pub body: Expr,
    /// Source span
    pub span: Span,
}

/// A generic type parameter with interface bounds.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    /// Type parameter name
    pub name: Name,
    /// Explicit kind annotation when the parameter is constructor-kinded.
    pub kind: Option<KindAnnotation>,
    /// Interface bounds in canonical `T: Interface` form
    pub bounds: Vec<InterfaceBound>,
    /// Source span
    pub span: Span,
}

impl AsRef<str> for TypeParam {
    fn as_ref(&self) -> &str {
        self.name.as_ref()
    }
}

impl From<&str> for TypeParam {
    fn from(value: &str) -> Self {
        Self {
            name: value.into(),
            kind: None,
            bounds: Vec::new(),
            span: Span::default(),
        }
    }
}

impl From<String> for TypeParam {
    fn from(value: String) -> Self {
        Self {
            name: value.into(),
            kind: None,
            bounds: Vec::new(),
            span: Span::default(),
        }
    }
}

impl fmt::Display for TypeParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name.as_ref())
    }
}

/// A canonical interface bound `T: Interface`.
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceBound {
    /// Interface name referenced by the bound
    pub interface: Name,
    /// Source span
    pub span: Span,
}

/// Visibility modifiers for definitions (pub, pub(crate), etc.)
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub enum Visibility {
    /// Default visibility (private to module)
    #[default]
    Inherited,
    /// `pub` - visible everywhere
    Public,
    /// `pub(crate)` - visible within the crate/package
    Crate,
    /// `pub(super)` - visible to parent module and its descendants
    ///
    /// The `levels` field indicates how many levels up (1 = parent, 2 = grandparent, etc.)
    Super {
        /// Number of parent levels this item is visible to (1 = parent, 2 = grandparent, etc.)
        levels: usize,
    },
    /// `pub(self)` - equivalent to private (explicit)
    Self_,
    /// `pub(in path)` - visible in specific module path
    Restricted { path: Box<str> },
}

impl Visibility {
    /// Check if this visibility is public (not inherited/private)
    pub fn is_pub(&self) -> bool {
        !matches!(self, Visibility::Inherited)
    }

    /// Check if an item with this visibility is accessible from the given module
    ///
    /// # Arguments
    /// * `from` - the module path where the access is occurring
    /// * `owner` - the module path where the item is defined
    pub fn is_visible_in_module(&self, from: &str, owner: &str) -> bool {
        match self {
            Visibility::Inherited => from == owner,
            Visibility::Public => true,
            Visibility::Crate => !from.starts_with("external"),
            Visibility::Super { .. } => from.starts_with(owner),
            Visibility::Self_ => from == owner,
            Visibility::Restricted { path } => from.starts_with(path.as_ref()),
        }
    }
}

/// A workflow parameter with name and type.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    /// Parameter name
    pub name: Name,
    /// Parameter type
    pub ty: Type,
    /// Source span
    pub span: Span,
}

/// An ensures clause (postcondition).
#[derive(Debug, Clone, PartialEq)]
pub struct EnsuresClause {
    /// The predicate expression
    pub expr: Expr,
    /// Source span
    pub span: Span,
}

/// A requirement (precondition).
#[derive(Debug, Clone, PartialEq)]
pub enum Requirement {
    /// Required capability with minimum effect level
    HasCapability { cap: Name, min_effect: EffectType },
    /// Required role membership
    HasRole(Name),
    /// Arithmetic constraint on parameter
    Arithmetic { expr: Expr },
}

/// Workflow contract with preconditions and postconditions.
#[derive(Debug, Clone, PartialEq)]
pub struct Contract {
    /// Preconditions that must hold at call site
    pub requires: Vec<Requirement>,
    /// Postconditions guaranteed after workflow completes
    pub ensures: Vec<EnsuresClause>,
}

/// A reference to a role in a `plays role(R)` clause.
#[derive(Debug, Clone, PartialEq)]
pub struct RoleRef {
    /// Name of the role
    pub name: Name,
    /// Source span
    pub span: Span,
}

/// A capability declaration in a workflow header (e.g., `capabilities: [file, network @ { ... }]`).
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityDecl {
    /// The capability name
    pub capability: Name,
    /// Optional constraint refinement (e.g., `@ { paths: ["/tmp/*"] }`)
    pub constraints: Option<ConstraintBlock>,
    /// Source span
    pub span: Span,
}

/// A constraint block for capability refinement.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintBlock {
    /// Fields in the constraint block
    pub fields: Vec<ConstraintField>,
    /// Source span
    pub span: Span,
}

/// A single field in a constraint block.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintField {
    /// Field name
    pub name: Name,
    /// Field value
    pub value: ConstraintValue,
    /// Source span
    pub span: Span,
}

/// A constraint value - can be primitive or composite.
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintValue {
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int(i64),
    /// String value
    String(String),
    /// Array of values
    Array(Vec<ConstraintValue>),
    /// Object with key-value pairs
    Object(Vec<(String, ConstraintValue)>),
}

/// A workflow-owned resource header clause (`owns name: Type`).
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowOwnedResource {
    /// Resource binding name
    pub name: Name,
    /// Resource type
    pub ty: Type,
    /// Source span
    pub span: Span,
}

/// A workflow-used capability binding header clause (`uses name: Interface = Impl(args...)`).
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowUsedBinding {
    /// Binding name visible to the workflow
    pub name: Name,
    /// Required capability interface type
    pub interface: Type,
    /// Implementation expression used to construct/bind the capability
    pub implementation: Expr,
    /// Source span
    pub span: Span,
}

/// A workflow definition.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowDef {
    /// Name of the workflow
    pub name: Name,
    /// Generic type parameters with explicit interface bounds
    pub type_params: Vec<TypeParam>,
    /// Workflow parameters (name: type)
    pub params: Vec<Parameter>,
    /// Optional declared return type from the workflow header
    pub declared_return_type: Option<Type>,
    /// Roles this workflow plays (from `plays role(R)` clauses)
    pub plays_roles: Vec<RoleRef>,
    /// Capabilities this workflow uses (from `capabilities: [...]` clause)
    pub capabilities: Vec<CapabilityDecl>,
    /// Resources this workflow owns (from `owns name: Type` clauses)
    pub owned_resources: Vec<WorkflowOwnedResource>,
    /// Capability bindings this workflow uses (from `uses name: Interface = Impl(...)` clauses)
    pub used_bindings: Vec<WorkflowUsedBinding>,
    /// Source-ordered workflow header clauses, preserving deprecated legacy declaration order.
    pub header_events: Vec<WorkflowHeaderEvent>,
    /// The workflow body
    pub body: Workflow,
    /// Optional contract (requires/ensures)
    pub contract: Option<Contract>,
    /// Source span
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowHeaderEvent {
    PlaysRole(RoleRef),
    Capabilities(Vec<CapabilityDecl>),
    Owns(WorkflowOwnedResource),
    Uses(WorkflowUsedBinding),
    Requires { expr: Expr, span: Span },
    Ensures { expr: Expr, span: Span },
}

/// Surface workflow syntax - more flexible than core IR.
#[derive(Debug, Clone, PartialEq)]
pub enum Workflow {
    /// Observe phase: invoke a capability to observe
    Observe {
        /// Capability to invoke
        capability: Name,
        /// Optional binding for result
        binding: Option<Pattern>,
        /// Optional continuation
        continuation: Option<Box<Workflow>>,
        /// Source span
        span: Span,
    },
    /// Orient phase: evaluate an expression
    Orient {
        /// Expression to evaluate
        expr: Expr,
        /// Optional binding for result
        binding: Option<Pattern>,
        /// Optional continuation
        continuation: Option<Box<Workflow>>,
        /// Source span
        span: Span,
    },
    /// Propose phase: propose an action
    Propose {
        /// Action to propose
        action: ActionRef,
        /// Optional binding for result
        binding: Option<Pattern>,
        /// Optional continuation
        continuation: Option<Box<Workflow>>,
        /// Source span
        span: Span,
    },
    /// Decide phase: apply a policy decision
    Decide {
        /// Condition expression
        expr: Expr,
        /// Optional policy name
        policy: Option<Name>,
        /// Then branch
        then_branch: Box<Workflow>,
        /// Optional else branch
        else_branch: Option<Box<Workflow>>,
        /// Source span
        span: Span,
    },
    /// Check phase: verify an obligation or policy instance
    Check {
        /// The check target - either an obligation reference (legacy) or policy instance
        target: CheckTarget,
        /// Optional continuation
        continuation: Option<Box<Workflow>>,
        /// Source span
        span: Span,
    },
    /// Create an obligation that must be discharged before workflow completes
    Oblige {
        /// Name of the obligation to create
        obligation: Name,
        /// Source span
        span: Span,
    },
    /// Act phase: execute an action
    Act {
        /// Action to execute
        action: ActionRef,
        /// Optional guard
        guard: Option<Guard>,
        /// Optional binding for the action result
        result_name: Option<Name>,
        /// Optional continuation after the action completes.
        /// `None` means terminal (bare act, equivalent to `Done`).
        continuation: Option<Box<Workflow>>,
        /// Source span
        span: Span,
    },
    /// Let binding: bind a pattern to an expression
    Let {
        /// Pattern to bind
        pattern: Pattern,
        /// Expression to evaluate
        expr: Expr,
        /// Optional continuation
        continuation: Option<Box<Workflow>>,
        /// Source span
        span: Span,
    },
    /// Conditional workflow
    If {
        /// Condition expression
        condition: Expr,
        /// Then branch
        then_branch: Box<Workflow>,
        /// Optional else branch
        else_branch: Option<Box<Workflow>>,
        /// Source span
        span: Span,
    },
    /// For loop: iterate over a collection
    For {
        /// Pattern for each element
        pattern: Pattern,
        /// Collection to iterate over
        collection: Expr,
        /// Body of the loop
        body: Box<Workflow>,
        /// Source span
        span: Span,
    },
    /// With clause: scoped capability
    With {
        /// Capability to use
        capability: Name,
        /// Body to execute with the capability
        body: Box<Workflow>,
        /// Source span
        span: Span,
    },
    /// Maybe: try primary, fallback on failure
    Maybe {
        /// Primary workflow
        primary: Box<Workflow>,
        /// Fallback workflow
        fallback: Box<Workflow>,
        /// Source span
        span: Span,
    },
    /// Must: ensure workflow succeeds
    Must {
        /// Body that must succeed
        body: Box<Workflow>,
        /// Source span
        span: Span,
    },
    /// Sequential composition
    Seq {
        /// First workflow
        first: Box<Workflow>,
        /// Second workflow
        second: Box<Workflow>,
        /// Source span
        span: Span,
    },
    /// Done: successful completion
    Done {
        /// Source span
        span: Span,
    },
    /// Ret: return an expression
    Ret {
        /// Expression to return
        expr: Expr,
        /// Source span
        span: Span,
    },
    /// Set: Set a value on an output capability
    Set {
        /// Capability name (e.g., "hvac" in "hvac:target")
        capability: Name,
        /// Channel name (e.g., "target" in "hvac:target")
        channel: Name,
        /// Value expression to set
        value: Expr,
        /// Optional continuation
        continuation: Option<Box<Workflow>>,
        /// Source span
        span: Span,
    },
    /// Send: Send a value to an output stream
    Send {
        /// Capability name (e.g., "kafka" in "kafka:orders")
        capability: Name,
        /// Channel name (e.g., "orders" in "kafka:orders")
        channel: Name,
        /// Value expression to send
        value: Expr,
        /// Optional continuation
        continuation: Option<Box<Workflow>>,
        /// Source span
        span: Span,
    },
    /// Receive: Pattern match on incoming messages from streams
    Receive {
        /// Receive mode (blocking or non-blocking)
        mode: ReceiveMode,
        /// Receive arms for matching messages
        arms: Vec<ReceiveArm>,
        /// Whether this is a control receive
        is_control: bool,
        /// Source span
        span: Span,
    },
    /// Yield: Delegate to a role with resumption
    Yield {
        /// Role to delegate to
        role: Name,
        /// Expression to send to the role
        expr: Expr,
        /// Resume variable binding
        resume_var: Name,
        /// Resume variable type
        resume_type: Type,
        /// Match arms for handling responses
        arms: Vec<YieldArm>,
        /// Source span
        span: Span,
    },
    /// Resume: Resume from a yield with a value
    Resume {
        /// Expression to resume with
        expr: Expr,
        /// Type of the expression
        ty: Type,
        /// Source span
        span: Span,
    },
}

/// A single arm in a yield expression for handling responses.
#[derive(Debug, Clone, PartialEq)]
pub struct YieldArm {
    /// Pattern to match against the response
    pub pattern: Pattern,
    /// Body workflow to execute when pattern matches
    pub body: Workflow,
    /// Source span
    pub span: Span,
}

/// Receive mode: non-blocking, blocking forever, or blocking with timeout.
#[derive(Debug, Clone, PartialEq)]
pub enum ReceiveMode {
    /// Non-blocking receive - check for messages and continue immediately
    NonBlocking,
    /// Blocking receive - wait for messages, optionally with timeout
    Blocking(Option<std::time::Duration>),
}

impl ReceiveMode {
    /// Returns true if this is a blocking receive mode
    pub fn is_blocking(&self) -> bool {
        matches!(self, ReceiveMode::Blocking(_))
    }

    /// Returns the timeout duration if set
    pub fn timeout(&self) -> Option<std::time::Duration> {
        match self {
            ReceiveMode::Blocking(timeout) => *timeout,
            ReceiveMode::NonBlocking => None,
        }
    }
}

/// Stream pattern for matching messages in receive arms.
#[derive(Debug, Clone, PartialEq)]
pub enum StreamPattern {
    /// Wildcard pattern: _
    Wildcard,
    /// String literal pattern (for control messages)
    Literal(String),
    /// Binding pattern: capability:channel as pattern
    Binding {
        /// Capability name
        capability: Name,
        /// Channel name
        channel: Name,
        /// Pattern to bind the message
        pattern: Pattern,
    },
}

/// A receive arm: pattern + optional guard + body.
#[derive(Debug, Clone, PartialEq)]
pub struct ReceiveArm {
    /// Pattern to match against incoming messages
    pub pattern: StreamPattern,
    /// Optional guard expression
    pub guard: Option<Expr>,
    /// Body workflow to execute when matched
    pub body: Workflow,
    /// Source span
    pub span: Span,
}

/// A statement inside an `act { ... }` block expression. SPEC-047 §4.2
///
/// Surface-only lowering carrier — does not survive into core IR.
#[derive(Debug, Clone, PartialEq)]
pub enum ActStmt {
    /// Monadic or pure bind: `name = expr;`
    Bind {
        /// Binding name
        name: Name,
        /// Bound expression (may be pure or effectful)
        value: Box<Expr>,
        /// Source span
        span: Span,
    },
    /// Return statement: `ret expr;`
    Return {
        /// Expression to return
        value: Box<Expr>,
        /// Source span
        span: Span,
    },
}

/// Target kind for a generalized `do:K { ... }` block.
///
/// This is parser-surface substrate only. Later typed elaboration resolves the
/// target and interprets any type arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct DoTarget {
    /// Target constructor name, e.g. `Act` or `Proc`.
    pub name: Name,
    /// Optional target type arguments, e.g. `K<T>`.
    pub args: Vec<Type>,
    /// Source span covering the target head.
    pub span: Span,
}

/// A statement inside a generalized `do:K { ... }` block.
///
/// Surface-only parser substrate. These statements are not lowered in
/// TASK-747; typed elaboration is responsible for assigning semantics later.
#[derive(Debug, Clone, PartialEq)]
pub enum DoStmt {
    /// Pure let statement: `let name = expr;`.
    Let {
        /// Binding name.
        name: Name,
        /// Bound expression.
        value: Box<Expr>,
        /// Source span covering the whole statement.
        span: Span,
    },
    /// Monadic bind statement: `name <- expr;`.
    Bind {
        /// Binding name.
        name: Name,
        /// Bound expression.
        value: Box<Expr>,
        /// Source span covering the whole statement.
        span: Span,
    },
    /// Workflow contract precondition statement: `requires: expr;`.
    WorkflowRequires {
        /// Raw contract expression, classified later.
        expr: Box<Expr>,
        /// Source span covering the whole statement.
        span: Span,
    },
    /// Workflow contract postcondition statement: `ensures: expr;`.
    WorkflowEnsures {
        /// Raw postcondition expression, classified later.
        expr: Box<Expr>,
        /// Source span covering the whole statement.
        span: Span,
    },
    /// Final return statement: `return expr`.
    Return {
        /// Returned expression.
        value: Box<Expr>,
        /// Source span covering the return statement.
        span: Span,
    },
}

/// A qualifier inside a bracket comprehension expression.
///
/// Surface-only parser substrate for SPEC-055. These mirror the non-return
/// statement forms accepted by generalized typed do-notation.
#[derive(Debug, Clone, PartialEq)]
pub enum ComprehensionQualifier {
    /// Monadic bind qualifier: `name <- expr`.
    Bind {
        /// Binding name.
        name: Name,
        /// Bound expression.
        value: Box<Expr>,
        /// Source span covering the whole qualifier.
        span: Span,
    },
    /// Discarding monadic bind qualifier: `_ <- expr`.
    DiscardBind {
        /// Bound expression.
        value: Box<Expr>,
        /// Source span covering the whole qualifier.
        span: Span,
    },
    /// Pure let qualifier: `let name = expr`.
    Let {
        /// Binding name.
        name: Name,
        /// Bound expression.
        value: Box<Expr>,
        /// Source span covering the whole qualifier.
        span: Span,
    },
}

/// Source-preserved operator token for notation-sensitive syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawOperatorToken {
    /// Source spelling of the operator token.
    pub spelling: Box<str>,
    /// Span covering only the operator token.
    pub span: Span,
}

/// Binary infix operator-section shape preserved before notation resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatorSectionKind {
    /// Bare operator value, such as `(<op>)`.
    Bare,
    /// Left section, such as `(a <op>)`.
    Left,
    /// Right section, such as `(<op> b)`.
    Right,
}

/// Source-preserving AST payload for binary infix operator sections.
#[derive(Debug, Clone, PartialEq)]
pub struct OperatorSection {
    /// Section kind.
    pub kind: OperatorSectionKind,
    /// Raw operator token.
    pub operator: RawOperatorToken,
    /// Left operand for left sections.
    pub left: Option<Box<Expr>>,
    /// Right operand for right sections.
    pub right: Option<Box<Expr>>,
    /// Span covering the full parenthesized section.
    pub span: Span,
}

/// Delimiter shape preserved for a fail-closed macro invocation.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MacroDelimiter {
    /// Parenthesized invocation, e.g. `m!(...)`.
    Paren,
    /// Bracketed invocation, e.g. `m![...]`.
    Bracket,
    /// Braced invocation, e.g. `m!{...}`.
    Brace,
}

/// Delimiter-preserving token-tree payload for macro invocation syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MacroTokenTree {
    /// A raw token spelling with its source span.
    Token {
        /// Token text exactly as written in the invocation body.
        spelling: Box<str>,
        /// Span covering this token.
        span: Span,
    },
    /// A nested delimited token-tree group.
    Group {
        /// Delimiter that opened the group.
        delimiter: MacroDelimiter,
        /// Child token trees inside the delimiters.
        tokens: Vec<MacroTokenTree>,
        /// Span covering the complete delimited group.
        span: Span,
    },
}

/// Structured body carrier for a macro invocation.
#[derive(Debug, Clone, PartialEq)]
pub enum MacroInvocationBody {
    /// Parenthesized invocation parsed as expression arguments for the Phase 172 MVP subset.
    ExprArgs(Vec<Expr>),
    /// Delimiter-preserving token-tree body for bracket/brace and unsupported token-tree shapes.
    TokenTrees(Vec<MacroTokenTree>),
}

/// Parsed macro invocation payload preserved only for fail-closed diagnostics.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroInvocation {
    /// Unqualified macro name before `!`.
    pub name: Name,
    /// Delimiter used by the invocation.
    pub delimiter: MacroDelimiter,
    /// Raw delimited body text inside the invocation delimiter.
    pub raw_body: Box<str>,
    /// Structured invocation body for syntax-phase macro consumers.
    pub body: MacroInvocationBody,
    /// Delimiter-preserving token-tree body for syntax-phase consumers.
    pub token_trees: Vec<MacroTokenTree>,
    /// Structured expression arguments for the executable parenthesized MVP subset.
    pub args: Option<Vec<Expr>>,
    /// Span covering the full invocation.
    pub span: Span,
}

/// Origin metadata for surface nodes that are copied, expanded, or desugared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceOrigin {
    /// Node originates directly from source text.
    Source { span: Span },
    /// Node was produced by macro expansion.
    MacroExpansion {
        call_span: Span,
        expansion_id: Box<str>,
    },
    /// Node was produced by notation expansion.
    NotationExpansion {
        notation_span: Span,
        target: Box<str>,
    },
    /// Node was produced by operator-section expansion.
    OperatorSection {
        section_span: Span,
        operator_span: Span,
    },
    /// Node was produced by a named desugaring rule.
    Desugaring { source_span: Span, rule: Box<str> },
}

/// Parsed surface AST wrapper used to name the pre-expansion boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedSurfaceModule {
    /// Parsed source module before macro or notation expansion.
    pub module: ModuleFile,
    /// Origin for the parsed module.
    pub origin: SurfaceOrigin,
}

/// Expanded surface AST wrapper used to name the post-expansion boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct ExpandedSurfaceModule {
    /// Expanded module. In Phase 168 this is a no-op clone for syntax that needs no expansion.
    pub module: ModuleFile,
    /// Boundary diagnostics collected or rejected during expansion.
    pub diagnostics: Vec<ExpansionDiagnostic>,
    /// Narrow surface-side origin sidecars for nodes generated during expansion.
    pub origins: Vec<ExpandedSurfaceOrigin>,
    /// Syntax-side hygiene sidecars for source, call-site, and generated identifiers.
    pub hygiene: Vec<IdentifierHygieneMetadata>,
}

/// Syntax-side identifier hygiene classification for macro/notation expansion.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IdentifierHygieneContext {
    /// Identifier binding comes from the macro or notation definition site.
    DefinitionSite,
    /// Identifier occurrence comes from the macro call site.
    CallSite,
    /// Identifier binding was generated by expansion and is not source-spellable.
    Generated,
}

/// Identifier hygiene metadata carried only at the expanded-surface boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifierHygieneMetadata {
    /// Identifier spelling preserved for diagnostics.
    pub name: Name,
    /// Source or generated span for the identifier occurrence/binding.
    pub span: Span,
    /// Hygiene context for the identifier.
    pub context: IdentifierHygieneContext,
    /// Expansion product that generated or transported this identifier, if any.
    pub expansion_id: Option<ExpansionId>,
}

/// Origin sidecar for a generated surface node in an expanded module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpandedSurfaceOrigin {
    /// Stable identity assigned in expansion traversal order.
    pub expansion_id: ExpansionId,
    /// Span of the generated surface node.
    pub generated_span: Span,
    /// Expansion origin that produced the generated node.
    pub origin: SurfaceOrigin,
    /// Parent expansion origin when this node is generated inside another expansion product.
    pub parent: Option<Box<SurfaceOrigin>>,
}

/// Stable surface-side identity for an expansion product.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ExpansionId(pub u32);

/// Diagnostic emitted by the parsed-surface to expanded-surface boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpansionDiagnostic {
    /// Diagnostic kind.
    pub kind: ExpansionDiagnosticKind,
    /// Diagnostic span.
    pub span: Span,
    /// Human-readable message.
    pub message: Box<str>,
}

/// Expansion diagnostic category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpansionDiagnosticKind {
    /// Operator sections require notation/type elaboration before Core lowering.
    UnresolvedOperatorSection,
    /// Macro expansion is intentionally deferred in Phase 168.
    DeferredMacroExpansion,
    /// Notation resolution is intentionally deferred in Phase 168.
    DeferredNotationResolution,
    /// Local notation declaration duplicated an existing declaration.
    DuplicateNotationDeclaration,
    /// Local notation declarations conflict on precedence or associativity.
    ConflictingNotationDeclaration,
}

/// Expansion error for syntax that cannot honestly cross the expanded-surface boundary yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpansionError {
    /// Boundary encountered an operator section that must be resolved before Core lowering.
    UnresolvedOperatorSection { span: Span, operator: Box<str> },
    /// A local notation declaration duplicated an existing declaration.
    DuplicateNotationDeclaration {
        operator: Box<str>,
        first_span: Span,
        second_span: Span,
    },
    /// Local declarations for the same infix operator disagree on precedence or associativity.
    ConflictingNotationDeclaration {
        operator: Box<str>,
        first_span: Span,
        second_span: Span,
    },
    /// A local macro declaration duplicated an existing declaration.
    DuplicateMacroDeclaration {
        name: Box<str>,
        first_span: Span,
        second_span: Span,
    },
    /// A macro invocation did not resolve to a local macro declaration.
    UnknownMacroInvocation { span: Span, name: Box<str> },
    /// A macro invocation used syntax outside the Phase 172 executable MVP subset.
    UnsupportedMacroInvocation { span: Span, name: Box<str> },
    /// Token-tree macro input failed to reparse through the audited boundary.
    MacroTokenTreeReparseFailed {
        span: Span,
        name: Box<str>,
        reason: Box<str>,
    },
    /// A macro invocation provided the wrong number of expression arguments.
    MacroArityMismatch {
        span: Span,
        name: Box<str>,
        expected: usize,
        actual: usize,
    },
    /// A macro template used syntax outside the expression-template MVP whitelist.
    UnsupportedMacroTemplate {
        span: Span,
        name: Box<str>,
        reason: Box<str>,
    },
    /// A typed macro signature check failed before accepting expansion output.
    MacroTypeMismatch {
        span: Span,
        name: Box<str>,
        expected: Box<str>,
        actual: Box<str>,
        position: Box<str>,
    },
    /// Recursive macro expansion exceeded the conservative explicit depth bound.
    MacroExpansionDepthExceeded {
        span: Span,
        name: Box<str>,
        depth: usize,
    },
    /// Macro invocation is parsed for diagnostics but macro execution is deferred.
    DeferredMacroInvocation { span: Span, name: Box<str> },
}

impl std::fmt::Display for ExpansionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpansionError::UnresolvedOperatorSection { operator, .. } => write!(
                f,
                "operator section `{operator}` requires notation resolution before Core lowering"
            ),
            ExpansionError::DuplicateNotationDeclaration { operator, .. } => {
                write!(f, "duplicate notation declaration for `{operator}`")
            }
            ExpansionError::ConflictingNotationDeclaration { operator, .. } => write!(
                f,
                "conflicting precedence or associativity for notation `{operator}`"
            ),
            ExpansionError::DuplicateMacroDeclaration { name, .. } => {
                write!(f, "duplicate macro declaration for `{name}`")
            }
            ExpansionError::UnknownMacroInvocation { name, .. } => {
                write!(f, "unknown local macro invocation `{name}!`")
            }
            ExpansionError::UnsupportedMacroInvocation { name, .. } => write!(
                f,
                "macro invocation `{name}!` uses unsupported Phase 172 MVP syntax"
            ),
            ExpansionError::MacroTokenTreeReparseFailed { name, reason, .. } => write!(
                f,
                "macro invocation `{name}!` token-tree input failed to reparse: {reason}"
            ),
            ExpansionError::MacroArityMismatch {
                name,
                expected,
                actual,
                ..
            } => write!(
                f,
                "macro invocation `{name}!` expected {expected} argument(s), got {actual}"
            ),
            ExpansionError::UnsupportedMacroTemplate { name, reason, .. } => {
                write!(
                    f,
                    "macro `{name}` uses unsupported template syntax: {reason}"
                )
            }
            ExpansionError::MacroTypeMismatch {
                name,
                expected,
                actual,
                position,
                ..
            } => write!(
                f,
                "macro `{name}` typed signature mismatch at {position}: expected {expected}, got {actual}"
            ),
            ExpansionError::MacroExpansionDepthExceeded { name, depth, .. } => write!(
                f,
                "macro expansion depth limit exceeded while expanding `{name}!` at depth {depth}"
            ),
            ExpansionError::DeferredMacroInvocation { name, .. } => write!(
                f,
                "unexpanded macro invocation carrier `{name}!` reached an expanded-surface boundary"
            ),
        }
    }
}

impl std::error::Error for ExpansionError {}

/// Local macro table built during parsed-surface expansion.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LocalMacroTable {
    entries: Vec<LocalMacroEntry>,
}

/// Resolved local macro declaration row.
#[derive(Debug, Clone, PartialEq)]
pub struct LocalMacroEntry {
    /// Local macro name.
    pub name: Name,
    /// Macro parameter names.
    pub params: Vec<Name>,
    /// Parsed expression template body.
    pub body: Expr,
    /// Optional syntax-phase typed macro signature carrier.
    pub typed_signature: Option<MacroTypeSignatureSummary>,
    /// Canonical syntax-phase identity for this macro row.
    pub identity: MacroDeclarationIdentity,
    /// Local callable identities available while checking this template.
    callable_env: Vec<CallableTypeSummary>,
    /// Source span of the declaration.
    pub span: Span,
}

impl LocalMacroTable {
    /// Resolve an unqualified macro invocation name against local declarations.
    pub fn resolve(&self, name: &str) -> Option<&LocalMacroEntry> {
        self.entries
            .iter()
            .find(|entry| entry.name.as_ref() == name)
    }

    /// Iterate entries in source order.
    pub fn entries(&self) -> impl Iterator<Item = &LocalMacroEntry> {
        self.entries.iter()
    }

    /// Insert an imported syntax-phase macro row into the expansion table.
    pub fn insert_imported(&mut self, entry: LocalMacroEntry) -> Result<(), ExpansionError> {
        if let Some(existing) = self.resolve(entry.name.as_ref()) {
            return Err(ExpansionError::DuplicateMacroDeclaration {
                name: entry.name.clone(),
                first_span: existing.span,
                second_span: entry.span,
            });
        }
        self.entries.push(entry);
        Ok(())
    }
}

/// Build the local macro table for a parsed module.
pub fn build_local_macro_table(module: &ModuleFile) -> Result<LocalMacroTable, ExpansionError> {
    build_local_macro_table_for_definitions(&module.definitions)
}

/// Collect explicit syntax-phase summaries for public macro declarations.
///
/// This is export metadata only. It does not activate imported macros and does
/// not expose macros as callables.
pub fn collect_public_macro_summaries(
    module: &ModuleFile,
    module_path: impl Into<Box<str>>,
) -> Result<Vec<MacroSummary>, ExpansionError> {
    collect_public_macro_summaries_for_definitions(&module.definitions, module_path.into())
}

/// Collect same-file syntax-phase macro declaration identities.
pub fn collect_local_macro_identities(
    module: &ModuleFile,
) -> Result<Vec<MacroDeclarationIdentity>, ExpansionError> {
    Ok(build_local_macro_table(module)?
        .entries()
        .map(|entry| entry.identity.clone())
        .collect())
}

/// Resolve a same-file syntax-phase macro declaration identity by local name.
pub fn resolve_local_macro_identity(
    module: &ModuleFile,
    name: &str,
) -> Result<Option<MacroDeclarationIdentity>, ExpansionError> {
    Ok(build_local_macro_table(module)?
        .resolve(name)
        .map(|entry| entry.identity.clone()))
}

/// Collect unique same-file callable declaration identities for public ordinary
/// functions and builtin functions.
///
/// This deliberately excludes macro declarations and ambiguous duplicate names.
#[must_use]
pub fn collect_local_callable_identities(module: &ModuleFile) -> Vec<CallableDeclarationIdentity> {
    collect_local_callable_type_summaries(&module.definitions)
        .into_iter()
        .filter(|summary| !summary.ambiguous)
        .map(|summary| summary.identity)
        .collect()
}

fn collect_public_macro_summaries_for_definitions(
    definitions: &[Definition],
    module_path: Box<str>,
) -> Result<Vec<MacroSummary>, ExpansionError> {
    let table = build_local_macro_table_for_definitions(definitions)?;
    let mut summaries = Vec::new();
    for definition in definitions {
        let Definition::Macro(decl) = definition else {
            continue;
        };
        if !matches!(decl.visibility, Visibility::Public) {
            continue;
        }
        let entry = table
            .resolve(decl.name.as_ref())
            .expect("macro table was built from the same definitions");
        ensure_macro_template_supported(&decl.body, entry)?;
        summaries.push(MacroSummary {
            module_path: module_path.clone(),
            name: decl.name.clone(),
            identity: MacroDeclarationIdentity {
                origin: MacroIdentityOrigin::Imported {
                    module_path: module_path.clone(),
                    exported_name: decl.name.clone(),
                },
                local_name: decl.name.clone(),
                origin_span: decl.span,
                param_count: decl.params.len(),
            },
            visibility: decl.visibility.clone(),
            params: decl.params.clone(),
            input_kind: MacroInputKind::ExprArgs,
            output_kind: MacroOutputKind::Expr,
            template_fingerprint: MacroTemplateFingerprint {
                param_count: decl.params.len(),
                body_span: decl.body.span(),
            },
            hygiene_policy: MacroHygienePolicy::BinderFreeExpression,
            typed_signature: entry.typed_signature.clone(),
            origin_span: decl.span,
        });
    }
    Ok(summaries)
}

fn build_local_macro_table_for_definitions(
    definitions: &[Definition],
) -> Result<LocalMacroTable, ExpansionError> {
    let mut table = LocalMacroTable::default();
    collect_macro_entries(definitions, &mut table)?;
    Ok(table)
}

fn collect_macro_entries(
    definitions: &[Definition],
    table: &mut LocalMacroTable,
) -> Result<(), ExpansionError> {
    for definition in definitions {
        let Definition::Macro(decl) = definition else {
            continue;
        };
        if let Some(existing) = table.resolve(decl.name.as_ref()) {
            return Err(ExpansionError::DuplicateMacroDeclaration {
                name: decl.name.clone(),
                first_span: existing.span,
                second_span: decl.span,
            });
        }
        let callable_env = collect_local_callable_type_summaries(definitions);
        let typed_signature = infer_macro_type_signature(
            decl.typed_signature.clone(),
            &decl.params,
            &decl.body,
            decl.span,
            &callable_env,
        );
        table.entries.push(LocalMacroEntry {
            name: decl.name.clone(),
            params: decl.params.clone(),
            body: decl.body.clone(),
            typed_signature,
            identity: MacroDeclarationIdentity::local(
                decl.name.clone(),
                decl.span,
                decl.params.len(),
            ),
            callable_env: callable_env.clone(),
            span: decl.span,
        });
    }
    Ok(())
}

/// Local notation table built during parsed-surface expansion.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LocalNotationTable {
    entries: Vec<LocalNotationEntry>,
}

/// Resolved local notation row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalNotationEntry {
    /// Declared operator/pattern key.
    pub operator: Box<str>,
    /// Fixity for the operator.
    pub fixity: NotationFixity,
    /// Callable target path.
    pub target: CallablePath,
    /// Source span of the declaration.
    pub span: Span,
}

impl LocalNotationTable {
    /// Resolve a binary infix operator spelling against local notation declarations.
    pub fn resolve_infix(&self, operator: &str) -> Option<&LocalNotationEntry> {
        self.entries.iter().find(|entry| {
            entry.operator.as_ref() == operator
                && matches!(entry.fixity, NotationFixity::Infix { .. })
        })
    }

    /// Iterate entries in source order.
    pub fn entries(&self) -> impl Iterator<Item = &LocalNotationEntry> {
        self.entries.iter()
    }
}

/// Build the local notation table for a parsed module.
pub fn build_local_notation_table(
    module: &ModuleFile,
) -> Result<LocalNotationTable, ExpansionError> {
    build_local_notation_table_for_definitions(&module.definitions)
}

fn build_local_notation_table_for_definitions(
    definitions: &[Definition],
) -> Result<LocalNotationTable, ExpansionError> {
    let mut table = LocalNotationTable::default();
    collect_notation_entries(definitions, &mut table)?;
    Ok(table)
}

fn collect_notation_entries(
    definitions: &[Definition],
    table: &mut LocalNotationTable,
) -> Result<(), ExpansionError> {
    for definition in definitions {
        let Definition::Notation(decl) = definition else {
            continue;
        };
        let operator = notation_decl_key(decl);
        for existing in &table.entries {
            if existing.operator == operator && existing.fixity == decl.fixity {
                return Err(ExpansionError::DuplicateNotationDeclaration {
                    operator,
                    first_span: existing.span,
                    second_span: decl.span,
                });
            }
            if existing.operator == operator && same_fixity_class(&existing.fixity, &decl.fixity) {
                return Err(ExpansionError::ConflictingNotationDeclaration {
                    operator,
                    first_span: existing.span,
                    second_span: decl.span,
                });
            }
        }
        table.entries.push(LocalNotationEntry {
            operator,
            fixity: decl.fixity.clone(),
            target: decl.target.clone(),
            span: decl.span,
        });
    }
    Ok(())
}

fn notation_decl_key(decl: &NotationDecl) -> Box<str> {
    decl.pattern
        .tokens
        .first()
        .map(|token| token.spelling.clone())
        .unwrap_or_else(|| decl.pattern.raw.clone())
}

fn same_fixity_class(left: &NotationFixity, right: &NotationFixity) -> bool {
    matches!(
        (left, right),
        (NotationFixity::Prefix { .. }, NotationFixity::Prefix { .. })
            | (NotationFixity::Suffix { .. }, NotationFixity::Suffix { .. })
            | (NotationFixity::Mixfix, NotationFixity::Mixfix)
            | (NotationFixity::Infix { .. }, NotationFixity::Infix { .. })
    )
}

/// Expand a parsed surface module to the explicit expanded-surface boundary.
///
/// Phase 169 keeps macro expansion deferred but resolves local/built-in binary operator sections
/// into ordinary callable surface expressions before the module can cross into Core lowering.
pub fn expand_surface_module(module: ModuleFile) -> Result<ExpandedSurfaceModule, ExpansionError> {
    expand_surface_module_with_imported_macros(module, Vec::new())
}

/// Expand a parsed surface module with additional imported macro entries.
///
/// Imported entries must already have been gated by explicit macro summaries in
/// the engine/module-loader layer. They are syntax-phase rows only and do not
/// create callables or runtime bindings.
pub fn expand_surface_module_with_imported_macros(
    mut module: ModuleFile,
    imported_macros: Vec<LocalMacroEntry>,
) -> Result<ExpandedSurfaceModule, ExpansionError> {
    let mut origins = Vec::new();
    expand_macros_in_module(&mut module, &mut origins, imported_macros)?;
    elaborate_operator_sections_in_module(&mut module, &mut origins)?;
    if let Some(section) = find_operator_section_in_module(&module) {
        return Err(ExpansionError::UnresolvedOperatorSection {
            span: section.span,
            operator: section.operator.spelling.clone(),
        });
    }
    if let Some(invocation) = find_macro_invocation_in_module(&module) {
        return Err(ExpansionError::DeferredMacroInvocation {
            span: invocation.span,
            name: invocation.name.clone(),
        });
    }
    let hygiene = collect_identifier_hygiene_metadata(&module, &origins);
    Ok(ExpandedSurfaceModule {
        module,
        diagnostics: Vec::new(),
        origins,
        hygiene,
    })
}

const MACRO_EXPANSION_DEPTH_LIMIT: usize = 16;

fn expand_macros_in_module(
    module: &mut ModuleFile,
    origins: &mut Vec<ExpandedSurfaceOrigin>,
    imported_macros: Vec<LocalMacroEntry>,
) -> Result<(), ExpansionError> {
    let mut table = build_local_macro_table_for_definitions(&module.definitions)?;
    for entry in imported_macros {
        table.insert_imported(entry)?;
    }
    let notation_table = build_local_notation_table_for_definitions(&module.definitions)?;
    for definition in &mut module.definitions {
        expand_macros_in_definition(definition, &table, &notation_table, origins, 0)?;
    }
    for decl in &mut module.module_decls {
        if let crate::module::ModuleSource::Inline(definitions) = &mut decl.source {
            let inline_table = build_local_macro_table_for_definitions(definitions)?;
            let inline_notation_table = build_local_notation_table_for_definitions(definitions)?;
            for definition in definitions {
                expand_macros_in_definition(
                    definition,
                    &inline_table,
                    &inline_notation_table,
                    origins,
                    0,
                )?;
            }
        }
    }
    if let Some(workflow) = &mut module.workflow {
        expand_macros_in_workflow_def(workflow, &table, &notation_table, origins, 0)?;
    }
    Ok(())
}

fn expand_macros_in_definition(
    definition: &mut Definition,
    table: &LocalMacroTable,
    notation_table: &LocalNotationTable,
    origins: &mut Vec<ExpandedSurfaceOrigin>,
    depth: usize,
) -> Result<(), ExpansionError> {
    match definition {
        Definition::Function(def) => {
            expand_macros_in_expr(&mut def.body, table, notation_table, origins, depth)
        }
        Definition::Macro(_) => Ok(()),
        Definition::Law(def) => {
            expand_macros_in_expr(&mut def.proposition, table, notation_table, origins, depth)
        }
        Definition::CapabilityImplementation(def) => {
            for operation in &mut def.operations {
                expand_macros_in_expr(&mut operation.body, table, notation_table, origins, depth)?;
            }
            Ok(())
        }
        Definition::Impl(def) => {
            for method in &mut def.methods {
                expand_macros_in_expr(&mut method.body, table, notation_table, origins, depth)?;
            }
            for proof in &mut def.proofs {
                expand_macros_in_proof(proof, table, notation_table, origins, depth)?;
            }
            Ok(())
        }
        Definition::Proof(def) => {
            expand_macros_in_proof(def, table, notation_table, origins, depth)
        }
        Definition::Policy(def) => {
            if let Some(expr) = &mut def.where_clause {
                expand_macros_in_expr(expr, table, notation_table, origins, depth)?;
            }
            for field in &mut def.fields {
                if let Some(expr) = &mut field.default {
                    expand_macros_in_expr(expr, table, notation_table, origins, depth)?;
                }
            }
            Ok(())
        }
        Definition::Capability(def) => {
            for constraint in &mut def.constraints {
                for arg in &mut constraint.predicate.args {
                    expand_macros_in_expr(arg, table, notation_table, origins, depth)?;
                }
            }
            Ok(())
        }
        Definition::Proxy(def) => {
            expand_macros_in_workflow(&mut def.body, table, notation_table, origins, depth)
        }
        Definition::Interface(def) => {
            for law in &mut def.laws {
                expand_macros_in_expr(&mut law.proposition, table, notation_table, origins, depth)?;
            }
            Ok(())
        }
        Definition::Notation(_)
        | Definition::CapabilityInterface(_)
        | Definition::ResourceType(_)
        | Definition::Type(_)
        | Definition::DataKind(_)
        | Definition::TypeFn(_)
        | Definition::PropositionPredicate(_)
        | Definition::Role(_)
        | Definition::BuiltinFn(_)
        | Definition::SealedDomain(_) => Ok(()),
    }
}

fn expand_macros_in_proof(
    proof: &mut ProofDef,
    table: &LocalMacroTable,
    notation_table: &LocalNotationTable,
    origins: &mut Vec<ExpandedSurfaceOrigin>,
    depth: usize,
) -> Result<(), ExpansionError> {
    match &mut proof.body {
        ProofBody::Expr(expr) => expand_macros_in_expr(expr, table, notation_table, origins, depth),
        ProofBody::ByTestProperty { strategies } => {
            for strategy in strategies {
                expand_macros_in_expr(
                    &mut strategy.strategy_expr,
                    table,
                    notation_table,
                    origins,
                    depth,
                )?;
            }
            Ok(())
        }
        ProofBody::ByDefinition | ProofBody::ByTest { .. } | ProofBody::ByTestSmallWorld => Ok(()),
    }
}

fn expand_macros_in_workflow_def(
    workflow: &mut WorkflowDef,
    table: &LocalMacroTable,
    notation_table: &LocalNotationTable,
    origins: &mut Vec<ExpandedSurfaceOrigin>,
    depth: usize,
) -> Result<(), ExpansionError> {
    for binding in &mut workflow.used_bindings {
        expand_macros_in_expr(
            &mut binding.implementation,
            table,
            notation_table,
            origins,
            depth,
        )?;
    }
    for event in &mut workflow.header_events {
        match event {
            WorkflowHeaderEvent::Uses(binding) => {
                expand_macros_in_expr(
                    &mut binding.implementation,
                    table,
                    notation_table,
                    origins,
                    depth,
                )?;
            }
            WorkflowHeaderEvent::Requires { expr, .. }
            | WorkflowHeaderEvent::Ensures { expr, .. } => {
                expand_macros_in_expr(expr, table, notation_table, origins, depth)?;
            }
            WorkflowHeaderEvent::PlaysRole(_)
            | WorkflowHeaderEvent::Capabilities(_)
            | WorkflowHeaderEvent::Owns(_) => {}
        }
    }
    expand_macros_in_workflow(&mut workflow.body, table, notation_table, origins, depth)
}

fn expand_macros_in_workflow(
    workflow: &mut Workflow,
    table: &LocalMacroTable,
    notation_table: &LocalNotationTable,
    origins: &mut Vec<ExpandedSurfaceOrigin>,
    depth: usize,
) -> Result<(), ExpansionError> {
    match workflow {
        Workflow::Observe { continuation, .. } | Workflow::Propose { continuation, .. } => {
            if let Some(continuation) = continuation {
                expand_macros_in_workflow(continuation, table, notation_table, origins, depth)?;
            }
        }
        Workflow::Check {
            target,
            continuation,
            ..
        } => {
            match target {
                CheckTarget::Obligation(obligation) => {
                    expand_macros_in_expr(
                        &mut obligation.condition,
                        table,
                        notation_table,
                        origins,
                        depth,
                    )?;
                }
                CheckTarget::Policy(policy) => {
                    for (_, expr) in &mut policy.fields {
                        expand_macros_in_expr(expr, table, notation_table, origins, depth)?;
                    }
                }
            }
            if let Some(continuation) = continuation {
                expand_macros_in_workflow(continuation, table, notation_table, origins, depth)?;
            }
        }
        Workflow::Oblige { .. } | Workflow::Done { .. } => {}
        Workflow::Orient {
            expr, continuation, ..
        } => {
            expand_macros_in_expr(expr, table, notation_table, origins, depth)?;
            if let Some(continuation) = continuation {
                expand_macros_in_workflow(continuation, table, notation_table, origins, depth)?;
            }
        }
        Workflow::Decide {
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            expand_macros_in_expr(expr, table, notation_table, origins, depth)?;
            expand_macros_in_workflow(then_branch, table, notation_table, origins, depth)?;
            if let Some(else_branch) = else_branch {
                expand_macros_in_workflow(else_branch, table, notation_table, origins, depth)?;
            }
        }
        Workflow::Act {
            action,
            guard,
            continuation,
            ..
        } => {
            for arg in &mut action.args {
                expand_macros_in_expr(arg, table, notation_table, origins, depth)?;
            }
            if let Some(guard) = guard {
                expand_macros_in_guard(guard, table, notation_table, origins, depth)?;
            }
            if let Some(continuation) = continuation {
                expand_macros_in_workflow(continuation, table, notation_table, origins, depth)?;
            }
        }
        Workflow::Let {
            expr, continuation, ..
        } => {
            expand_macros_in_expr(expr, table, notation_table, origins, depth)?;
            if let Some(continuation) = continuation {
                expand_macros_in_workflow(continuation, table, notation_table, origins, depth)?;
            }
        }
        Workflow::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            expand_macros_in_expr(condition, table, notation_table, origins, depth)?;
            expand_macros_in_workflow(then_branch, table, notation_table, origins, depth)?;
            if let Some(else_branch) = else_branch {
                expand_macros_in_workflow(else_branch, table, notation_table, origins, depth)?;
            }
        }
        Workflow::For {
            collection, body, ..
        } => {
            expand_macros_in_expr(collection, table, notation_table, origins, depth)?;
            expand_macros_in_workflow(body, table, notation_table, origins, depth)?;
        }
        Workflow::With { body, .. } | Workflow::Must { body, .. } => {
            expand_macros_in_workflow(body, table, notation_table, origins, depth)?;
        }
        Workflow::Maybe {
            primary, fallback, ..
        } => {
            expand_macros_in_workflow(primary, table, notation_table, origins, depth)?;
            expand_macros_in_workflow(fallback, table, notation_table, origins, depth)?;
        }
        Workflow::Seq { first, second, .. } => {
            expand_macros_in_workflow(first, table, notation_table, origins, depth)?;
            expand_macros_in_workflow(second, table, notation_table, origins, depth)?;
        }
        Workflow::Ret { expr, .. } | Workflow::Resume { expr, .. } => {
            expand_macros_in_expr(expr, table, notation_table, origins, depth)?;
        }
        Workflow::Set {
            value,
            continuation,
            ..
        }
        | Workflow::Send {
            value,
            continuation,
            ..
        } => {
            expand_macros_in_expr(value, table, notation_table, origins, depth)?;
            if let Some(continuation) = continuation {
                expand_macros_in_workflow(continuation, table, notation_table, origins, depth)?;
            }
        }
        Workflow::Receive { arms, .. } => {
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    expand_macros_in_expr(guard, table, notation_table, origins, depth)?;
                }
                expand_macros_in_workflow(&mut arm.body, table, notation_table, origins, depth)?;
            }
        }
        Workflow::Yield { expr, arms, .. } => {
            expand_macros_in_expr(expr, table, notation_table, origins, depth)?;
            for arm in arms {
                expand_macros_in_workflow(&mut arm.body, table, notation_table, origins, depth)?;
            }
        }
    }
    Ok(())
}

fn expand_macros_in_guard(
    guard: &mut Guard,
    table: &LocalMacroTable,
    notation_table: &LocalNotationTable,
    origins: &mut Vec<ExpandedSurfaceOrigin>,
    depth: usize,
) -> Result<(), ExpansionError> {
    match guard {
        Guard::Always | Guard::Never => {}
        Guard::Pred(predicate) => {
            for arg in &mut predicate.args {
                expand_macros_in_expr(arg, table, notation_table, origins, depth)?;
            }
        }
        Guard::And(left, right) | Guard::Or(left, right) => {
            expand_macros_in_guard(left, table, notation_table, origins, depth)?;
            expand_macros_in_guard(right, table, notation_table, origins, depth)?;
        }
        Guard::Not(inner) => {
            expand_macros_in_guard(inner, table, notation_table, origins, depth)?;
        }
    }
    Ok(())
}

fn expand_macros_in_expr(
    expr: &mut Expr,
    table: &LocalMacroTable,
    notation_table: &LocalNotationTable,
    origins: &mut Vec<ExpandedSurfaceOrigin>,
    depth: usize,
) -> Result<(), ExpansionError> {
    expand_macros_in_expr_with_parent(expr, table, notation_table, origins, depth, None)
}

fn expand_macros_in_expr_with_parent(
    expr: &mut Expr,
    table: &LocalMacroTable,
    notation_table: &LocalNotationTable,
    origins: &mut Vec<ExpandedSurfaceOrigin>,
    depth: usize,
    parent: Option<&SurfaceOrigin>,
) -> Result<(), ExpansionError> {
    match expr {
        Expr::MacroInvocation { invocation } => {
            let expanded =
                expand_macro_invocation(invocation, table, notation_table, origins, depth, parent)?;
            *expr = expanded;
            Ok(())
        }
        Expr::OperatorSection { section } => {
            if let Some(left) = &mut section.left {
                expand_macros_in_expr_with_parent(
                    left,
                    table,
                    notation_table,
                    origins,
                    depth,
                    parent,
                )?;
            }
            if let Some(right) = &mut section.right {
                expand_macros_in_expr_with_parent(
                    right,
                    table,
                    notation_table,
                    origins,
                    depth,
                    parent,
                )?;
            }
            Ok(())
        }
        Expr::FieldAccess { base, .. } => {
            expand_macros_in_expr_with_parent(base, table, notation_table, origins, depth, parent)
        }
        Expr::IndexAccess { base, index, .. } => {
            expand_macros_in_expr_with_parent(base, table, notation_table, origins, depth, parent)?;
            expand_macros_in_expr_with_parent(index, table, notation_table, origins, depth, parent)
        }
        Expr::Unary { operand, .. } => expand_macros_in_expr_with_parent(
            operand,
            table,
            notation_table,
            origins,
            depth,
            parent,
        ),
        Expr::Binary { left, right, .. } => {
            expand_macros_in_expr_with_parent(left, table, notation_table, origins, depth, parent)?;
            expand_macros_in_expr_with_parent(right, table, notation_table, origins, depth, parent)
        }
        Expr::Call { args, .. } | Expr::List { items: args, .. } => {
            for arg in args {
                expand_macros_in_expr_with_parent(
                    arg,
                    table,
                    notation_table,
                    origins,
                    depth,
                    parent,
                )?;
            }
            Ok(())
        }
        Expr::Constructor {
            fields, payload, ..
        } => {
            for (_, expr) in fields {
                expand_macros_in_expr_with_parent(
                    expr,
                    table,
                    notation_table,
                    origins,
                    depth,
                    parent,
                )?;
            }
            match payload {
                ConstructorPayload::Tuple(items) => {
                    for item in items {
                        expand_macros_in_expr_with_parent(
                            item,
                            table,
                            notation_table,
                            origins,
                            depth,
                            parent,
                        )?;
                    }
                }
                ConstructorPayload::Record(fields) => {
                    for (_, expr) in fields {
                        expand_macros_in_expr_with_parent(
                            expr,
                            table,
                            notation_table,
                            origins,
                            depth,
                            parent,
                        )?;
                    }
                }
                ConstructorPayload::Unit => {}
            }
            Ok(())
        }
        Expr::Record { fields, .. } => {
            for (_, expr) in fields {
                expand_macros_in_expr_with_parent(
                    expr,
                    table,
                    notation_table,
                    origins,
                    depth,
                    parent,
                )?;
            }
            Ok(())
        }
        Expr::FnApply { func, args, .. } => {
            expand_macros_in_expr_with_parent(func, table, notation_table, origins, depth, parent)?;
            for arg in args {
                expand_macros_in_expr_with_parent(
                    arg,
                    table,
                    notation_table,
                    origins,
                    depth,
                    parent,
                )?;
            }
            Ok(())
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            expand_macros_in_expr_with_parent(
                condition,
                table,
                notation_table,
                origins,
                depth,
                parent,
            )?;
            expand_macros_in_expr_with_parent(
                then_branch,
                table,
                notation_table,
                origins,
                depth,
                parent,
            )?;
            if let Some(else_branch) = else_branch {
                expand_macros_in_expr_with_parent(
                    else_branch,
                    table,
                    notation_table,
                    origins,
                    depth,
                    parent,
                )?;
            }
            Ok(())
        }
        Expr::Match {
            scrutinee, arms, ..
        } => {
            expand_macros_in_expr_with_parent(
                scrutinee,
                table,
                notation_table,
                origins,
                depth,
                parent,
            )?;
            for arm in arms {
                expand_macros_in_expr_with_parent(
                    &mut arm.body,
                    table,
                    notation_table,
                    origins,
                    depth,
                    parent,
                )?;
            }
            Ok(())
        }
        Expr::IfLet {
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            expand_macros_in_expr_with_parent(expr, table, notation_table, origins, depth, parent)?;
            expand_macros_in_expr_with_parent(
                then_branch,
                table,
                notation_table,
                origins,
                depth,
                parent,
            )?;
            expand_macros_in_expr_with_parent(
                else_branch,
                table,
                notation_table,
                origins,
                depth,
                parent,
            )
        }
        Expr::Fail { payload, .. } => expand_macros_in_expr_with_parent(
            payload,
            table,
            notation_table,
            origins,
            depth,
            parent,
        ),
        Expr::WithError { body, arms, .. } => {
            expand_macros_in_expr_with_parent(body, table, notation_table, origins, depth, parent)?;
            for arm in arms {
                expand_macros_in_expr_with_parent(
                    &mut arm.body,
                    table,
                    notation_table,
                    origins,
                    depth,
                    parent,
                )?;
            }
            Ok(())
        }
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            for stmt in statements {
                match stmt {
                    BlockStmt::Let { expr, .. } => {
                        expand_macros_in_expr_with_parent(
                            expr,
                            table,
                            notation_table,
                            origins,
                            depth,
                            parent,
                        )?;
                    }
                }
            }
            if let Some(tail_expr) = tail_expr {
                expand_macros_in_expr_with_parent(
                    tail_expr,
                    table,
                    notation_table,
                    origins,
                    depth,
                    parent,
                )?;
            }
            Ok(())
        }
        Expr::FnDef { body, .. } => {
            expand_macros_in_expr_with_parent(body, table, notation_table, origins, depth, parent)
        }
        Expr::ActBlock { stmts, .. } => {
            for stmt in stmts {
                match stmt {
                    ActStmt::Bind { value, .. } | ActStmt::Return { value, .. } => {
                        expand_macros_in_expr_with_parent(
                            value,
                            table,
                            notation_table,
                            origins,
                            depth,
                            parent,
                        )?;
                    }
                }
            }
            Ok(())
        }
        Expr::DoBlock { stmts, .. } => {
            for stmt in stmts {
                match stmt {
                    DoStmt::Let { value, .. }
                    | DoStmt::Bind { value, .. }
                    | DoStmt::WorkflowRequires { expr: value, .. }
                    | DoStmt::WorkflowEnsures { expr: value, .. }
                    | DoStmt::Return { value, .. } => {
                        expand_macros_in_expr_with_parent(
                            value,
                            table,
                            notation_table,
                            origins,
                            depth,
                            parent,
                        )?;
                    }
                }
            }
            Ok(())
        }
        Expr::Comprehension {
            result, qualifiers, ..
        } => {
            expand_macros_in_expr_with_parent(
                result,
                table,
                notation_table,
                origins,
                depth,
                parent,
            )?;
            for qualifier in qualifiers {
                match qualifier {
                    ComprehensionQualifier::Bind { value, .. }
                    | ComprehensionQualifier::DiscardBind { value, .. }
                    | ComprehensionQualifier::Let { value, .. } => {
                        expand_macros_in_expr_with_parent(
                            value,
                            table,
                            notation_table,
                            origins,
                            depth,
                            parent,
                        )?;
                    }
                }
            }
            Ok(())
        }
        Expr::Literal(_)
        | Expr::Variable { .. }
        | Expr::Policy(_)
        | Expr::CheckObligation { .. }
        | Expr::Panic { .. } => Ok(()),
    }
}

fn expand_macro_invocation(
    invocation: &MacroInvocation,
    table: &LocalMacroTable,
    notation_table: &LocalNotationTable,
    origins: &mut Vec<ExpandedSurfaceOrigin>,
    depth: usize,
    parent: Option<&SurfaceOrigin>,
) -> Result<Expr, ExpansionError> {
    if depth >= MACRO_EXPANSION_DEPTH_LIMIT {
        return Err(ExpansionError::MacroExpansionDepthExceeded {
            span: invocation.span,
            name: invocation.name.clone(),
            depth,
        });
    }
    let Some(entry) = table.resolve(invocation.name.as_ref()) else {
        return Err(ExpansionError::UnknownMacroInvocation {
            span: invocation.span,
            name: invocation.name.clone(),
        });
    };
    let reparsed_args;
    let args =
        match invocation.delimiter {
            MacroDelimiter::Paren => invocation.args.as_ref().ok_or_else(|| {
                ExpansionError::UnsupportedMacroInvocation {
                    span: invocation.span,
                    name: invocation.name.clone(),
                }
            })?,
            MacroDelimiter::Bracket | MacroDelimiter::Brace => {
                reparsed_args = reparse_macro_token_tree_args(invocation)?;
                &reparsed_args
            }
        };
    if args.len() != entry.params.len() {
        return Err(ExpansionError::MacroArityMismatch {
            span: invocation.span,
            name: invocation.name.clone(),
            expected: entry.params.len(),
            actual: args.len(),
        });
    }
    check_typed_macro_signature(entry, args, invocation.span)?;
    let expansion_id = ExpansionId(
        u32::try_from(origins.len() + 1).expect("surface expansion origin count exceeds u32"),
    );
    ensure_macro_template_supported(&entry.body, entry)?;
    let mut expanded = substitute_macro_template(&entry.body, &entry.params, args, expansion_id);
    let macro_origin = SurfaceOrigin::MacroExpansion {
        call_span: invocation.span,
        expansion_id: entry.name.clone(),
    };
    origins.push(ExpandedSurfaceOrigin {
        expansion_id,
        generated_span: invocation.span,
        origin: macro_origin.clone(),
        parent: parent.cloned().map(Box::new),
    });
    expand_macros_in_expr_with_parent(
        &mut expanded,
        table,
        notation_table,
        origins,
        depth + 1,
        Some(&macro_origin),
    )?;
    elaborate_operator_sections_in_expr_with_parent(
        &mut expanded,
        notation_table,
        origins,
        Some(&macro_origin),
    );
    Ok(expanded)
}

fn reparse_macro_token_tree_args(
    invocation: &MacroInvocation,
) -> Result<Vec<Expr>, ExpansionError> {
    let source = render_macro_token_trees(&invocation.token_trees);
    if source.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut input = crate::input::new_input(&source);
    let args = crate::parse_expr::parse_args(&mut input).map_err(|error| {
        ExpansionError::MacroTokenTreeReparseFailed {
            span: invocation.span,
            name: invocation.name.clone(),
            reason: format!("{error}").into_boxed_str(),
        }
    })?;
    crate::parse_utils::skip_whitespace_and_comments(&mut input);
    if !input.input.is_empty() {
        return Err(ExpansionError::MacroTokenTreeReparseFailed {
            span: invocation.span,
            name: invocation.name.clone(),
            reason: "trailing tokens after reparsed macro arguments".into(),
        });
    }
    Ok(args)
}

#[derive(Debug, Clone, PartialEq)]
struct CallableTypeSummary {
    identity: CallableDeclarationIdentity,
    name: Name,
    param_types: Vec<Type>,
    return_type: Type,
    ambiguous: bool,
}

fn collect_local_callable_type_summaries(definitions: &[Definition]) -> Vec<CallableTypeSummary> {
    let mut summaries = Vec::<CallableTypeSummary>::new();
    for definition in definitions {
        let summary = match definition {
            Definition::Function(def)
                if def.return_type.is_some() && matches!(def.visibility, Visibility::Public) =>
            {
                Some(CallableTypeSummary {
                    identity: CallableDeclarationIdentity {
                        name: def.name.clone(),
                        kind: CallableDeclarationKind::Function,
                        origin_span: def.span,
                        param_count: def.params.len(),
                    },
                    name: def.name.clone(),
                    param_types: def.params.iter().map(|param| param.ty.clone()).collect(),
                    return_type: def
                        .return_type
                        .clone()
                        .expect("guard above checked return type"),
                    ambiguous: false,
                })
            }
            Definition::BuiltinFn(def) if matches!(def.visibility, Visibility::Public) => {
                Some(CallableTypeSummary {
                    identity: CallableDeclarationIdentity {
                        name: def.name.clone(),
                        kind: CallableDeclarationKind::BuiltinFn,
                        origin_span: def.span,
                        param_count: def.params.len(),
                    },
                    name: def.name.clone(),
                    param_types: def.params.iter().map(|param| param.ty.clone()).collect(),
                    return_type: def.return_type.clone(),
                    ambiguous: false,
                })
            }
            _ => None,
        };
        let Some(summary) = summary else {
            continue;
        };
        if let Some(existing) = summaries
            .iter_mut()
            .find(|existing| existing.name == summary.name)
        {
            existing.ambiguous = true;
        } else {
            summaries.push(summary);
        }
    }
    summaries
}

fn infer_macro_type_signature(
    explicit: Option<MacroTypeSignatureSummary>,
    params: &[Name],
    body: &Expr,
    span: Span,
    callable_env: &[CallableTypeSummary],
) -> Option<MacroTypeSignatureSummary> {
    let mut signature = explicit.unwrap_or_else(|| MacroTypeSignatureSummary {
        param_types: vec![None; params.len()],
        return_type: None,
        span,
    });
    if signature.return_type.is_none() {
        let param_env: Vec<(&Name, &Type)> = params
            .iter()
            .zip(signature.param_types.iter())
            .filter_map(|(name, ty)| ty.as_ref().map(|ty| (name, ty)))
            .collect();
        signature.return_type = infer_bounded_macro_expr_type(body, &param_env, callable_env);
    }

    if explicit_signature_has_information(&signature) {
        Some(signature)
    } else {
        None
    }
}

fn explicit_signature_has_information(signature: &MacroTypeSignatureSummary) -> bool {
    signature.return_type.is_some() || signature.param_types.iter().any(Option::is_some)
}

fn check_typed_macro_signature(
    entry: &LocalMacroEntry,
    args: &[Expr],
    _call_span: Span,
) -> Result<(), ExpansionError> {
    let Some(signature) = &entry.typed_signature else {
        return Ok(());
    };
    if signature.param_types.len() != entry.params.len() {
        return Err(ExpansionError::UnsupportedMacroTemplate {
            span: signature.span,
            name: entry.name.clone(),
            reason: format!(
                "typed signature has {} parameter(s), but macro declares {} parameter(s)",
                signature.param_types.len(),
                entry.params.len()
            )
            .into_boxed_str(),
        });
    }

    let param_env: Vec<(&Name, &Type)> = entry
        .params
        .iter()
        .zip(signature.param_types.iter())
        .filter_map(|(name, ty)| ty.as_ref().map(|ty| (name, ty)))
        .collect();

    for (index, expected) in signature.param_types.iter().enumerate() {
        let Some(expected) = expected else {
            continue;
        };
        let Some(actual) = infer_bounded_macro_expr_type(&args[index], &[], &[]) else {
            return Err(ExpansionError::MacroTypeMismatch {
                span: args[index].span(),
                name: entry.name.clone(),
                expected: format_type(expected).into_boxed_str(),
                actual: "unknown argument type".into(),
                position: format!("argument {} at call site", index + 1).into_boxed_str(),
            });
        };
        if &actual != expected {
            return Err(ExpansionError::MacroTypeMismatch {
                span: args[index].span(),
                name: entry.name.clone(),
                expected: format_type(expected).into_boxed_str(),
                actual: format_type(&actual).into_boxed_str(),
                position: format!("argument {} at call site", index + 1).into_boxed_str(),
            });
        }
    }

    if let Some(expected) = &signature.return_type {
        let Some(actual) =
            infer_bounded_macro_expr_type(&entry.body, &param_env, &entry.callable_env)
        else {
            return Err(ExpansionError::MacroTypeMismatch {
                span: entry.body.span(),
                name: entry.name.clone(),
                expected: format_type(expected).into_boxed_str(),
                actual: "unknown template result type".into(),
                position: "template result at macro definition".into(),
            });
        };
        if &actual != expected {
            return Err(ExpansionError::MacroTypeMismatch {
                span: entry.body.span(),
                name: entry.name.clone(),
                expected: format_type(expected).into_boxed_str(),
                actual: format_type(&actual).into_boxed_str(),
                position: "template result at macro definition".into(),
            });
        }
    }

    Ok(())
}

fn infer_bounded_macro_expr_type(
    expr: &Expr,
    env: &[(&Name, &Type)],
    callable_env: &[CallableTypeSummary],
) -> Option<Type> {
    match expr {
        Expr::Literal(Literal::Int(_)) => Some(Type::Name("Int".into())),
        Expr::Literal(Literal::String(_)) => Some(Type::Name("String".into())),
        Expr::Literal(Literal::Bool(_)) => Some(Type::Name("Bool".into())),
        Expr::Literal(Literal::Null) => Some(Type::Name("Null".into())),
        Expr::Variable { name, .. } => env
            .iter()
            .rev()
            .find(|(param, _)| *param == name)
            .map(|(_, ty)| (*ty).clone()),
        Expr::Unary { op, operand, .. } => match op {
            UnaryOp::Neg => match infer_bounded_macro_expr_type(operand, env, callable_env) {
                Some(Type::Name(name)) if name.as_ref() == "Int" => Some(Type::Name("Int".into())),
                _ => None,
            },
            UnaryOp::Not => match infer_bounded_macro_expr_type(operand, env, callable_env) {
                Some(Type::Name(name)) if name.as_ref() == "Bool" => {
                    Some(Type::Name("Bool".into()))
                }
                _ => None,
            },
        },
        Expr::Binary {
            op, left, right, ..
        } => infer_bounded_macro_binary_type(op, left, right, env, callable_env),
        Expr::Call {
            func, module, args, ..
        } => infer_bounded_macro_call_type(func, module.as_ref(), args, env, callable_env),
        Expr::FnDef {
            params,
            return_type,
            ..
        } => Some(Type::Fn(
            params
                .iter()
                .map(|(_, ty)| ty.as_ref().map(|ty| Type::Name(ty.clone())))
                .collect::<Option<Vec<_>>>()?,
            None,
            Box::new(Type::Name(return_type.as_ref()?.clone())),
        )),
        _ => None,
    }
}

fn infer_bounded_macro_call_type(
    func: &Name,
    module: Option<&Name>,
    args: &[Expr],
    env: &[(&Name, &Type)],
    callable_env: &[CallableTypeSummary],
) -> Option<Type> {
    if module.is_some() {
        return None;
    }
    let summary = callable_env.iter().find(|summary| summary.name == *func)?;
    if summary.ambiguous || summary.param_types.len() != args.len() {
        return None;
    }
    for (arg, expected) in args.iter().zip(&summary.param_types) {
        let actual = infer_bounded_macro_expr_type(arg, env, callable_env)?;
        if &actual != expected {
            return None;
        }
    }
    Some(summary.return_type.clone())
}

fn infer_bounded_macro_binary_type(
    op: &BinaryOp,
    left: &Expr,
    right: &Expr,
    env: &[(&Name, &Type)],
    callable_env: &[CallableTypeSummary],
) -> Option<Type> {
    let left_ty = infer_bounded_macro_expr_type(left, env, callable_env)?;
    let right_ty = infer_bounded_macro_expr_type(right, env, callable_env)?;
    match op {
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod
            if is_named_type(&left_ty, "Int") && is_named_type(&right_ty, "Int") =>
        {
            Some(Type::Name("Int".into()))
        }
        BinaryOp::Eq
        | BinaryOp::Neq
        | BinaryOp::Lt
        | BinaryOp::Leq
        | BinaryOp::Gt
        | BinaryOp::Geq
            if left_ty == right_ty =>
        {
            Some(Type::Name("Bool".into()))
        }
        BinaryOp::And | BinaryOp::Or
            if is_named_type(&left_ty, "Bool") && is_named_type(&right_ty, "Bool") =>
        {
            Some(Type::Name("Bool".into()))
        }
        _ => None,
    }
}

fn is_named_type(ty: &Type, expected: &str) -> bool {
    matches!(ty, Type::Name(name) if name.as_ref() == expected)
}

fn format_type(ty: &Type) -> String {
    match ty {
        Type::Name(name) => name.to_string(),
        Type::Hole { .. } => "_".to_string(),
        Type::List(inner) => format!("[{}]", format_type(inner)),
        Type::Tuple(items) => format!(
            "({})",
            items.iter().map(format_type).collect::<Vec<_>>().join(", ")
        ),
        Type::Record(fields) => format!(
            "{{{}}}",
            fields
                .iter()
                .map(|(name, ty)| format!("{name}: {}", format_type(ty)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Type::Capability(name) => format!("Capability<{name}>"),
        Type::Constructor { name, args } => format!(
            "{}<{}>",
            name,
            args.iter().map(format_type).collect::<Vec<_>>().join(", ")
        ),
        Type::Associated { base, name } => format!("{}::{name}", format_type(base)),
        Type::AssociatedFamilyProjection {
            interface,
            args,
            member,
            ..
        } => format!(
            "<{}<{}>>::{}",
            interface,
            args.iter().map(format_type).collect::<Vec<_>>().join(", "),
            member
        ),
        Type::Fn(params, row, ret) => {
            let row_text = row.as_ref().map_or_else(String::new, |row| {
                format!(
                    " {}",
                    row.items
                        .iter()
                        .map(format_row_item)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            });
            format!(
                "Fn({}) ->{} {}",
                params
                    .iter()
                    .map(format_type)
                    .collect::<Vec<_>>()
                    .join(", "),
                row_text,
                format_type(ret)
            )
        }
    }
}

fn format_row_path(path: &[Name]) -> String {
    path.iter()
        .map(|part| part.as_ref())
        .collect::<Vec<_>>()
        .join("::")
}

fn format_operation_row_path(path: &[Name], separator: Option<RowPathSeparator>) -> String {
    let Some((last, prefix)) = path.split_last() else {
        return String::new();
    };
    if prefix.is_empty() {
        return last.to_string();
    }
    let separator = match separator.unwrap_or(RowPathSeparator::DoubleColon) {
        RowPathSeparator::Dot => ".",
        RowPathSeparator::DoubleColon => "::",
    };
    format!("{}{separator}{last}", format_row_path(prefix))
}

fn format_row_item(item: &ComputationRowItem) -> String {
    match item {
        ComputationRowItem::Operation {
            path, separator, ..
        } => format_operation_row_path(path, *separator),
        ComputationRowItem::WholeRow { variable, .. } => variable.to_string(),
        ComputationRowItem::Resource { path, mode, .. } => {
            let path = format_row_path(path);
            match mode {
                Some(mode) => format!("resource {mode} {path}"),
                None => format!("resource {path}"),
            }
        }
        ComputationRowItem::Role { path, .. } => {
            format!("role {}", format_row_path(path))
        }
        ComputationRowItem::Policy { path, .. } => {
            format!("policy {}", format_row_path(path))
        }
        ComputationRowItem::Channel { path, mode, .. } => {
            let path = format_row_path(path);
            match mode {
                Some(mode) => format!("channel {mode} {path}"),
                None => format!("channel {path}"),
            }
        }
        ComputationRowItem::Process {
            keyword, operation, ..
        } => match operation {
            Some(operation) => format!("{keyword} {operation}"),
            None => keyword.to_string(),
        },
        ComputationRowItem::Fail { path, .. } => match path {
            Some(path) => format!("fail {}", format_row_path(path)),
            None => "fail".to_string(),
        },
        ComputationRowItem::Evidence { path, .. } => {
            format!("evidence {}", format_row_path(path))
        }
        ComputationRowItem::Group { path, .. } => {
            format!("group {}", format_row_path(path))
        }
        ComputationRowItem::Tail { variable, .. } => {
            format!("| {variable}")
        }
    }
}
fn render_macro_token_trees(trees: &[MacroTokenTree]) -> String {
    let mut out = String::new();
    for tree in trees {
        if !out.is_empty() {
            out.push(' ');
        }
        render_macro_token_tree(tree, &mut out);
    }
    out
}

fn render_macro_token_tree(tree: &MacroTokenTree, out: &mut String) {
    match tree {
        MacroTokenTree::Token { spelling, .. } => out.push_str(spelling),
        MacroTokenTree::Group {
            delimiter, tokens, ..
        } => {
            let (open, close) = match delimiter {
                MacroDelimiter::Paren => ('(', ')'),
                MacroDelimiter::Bracket => ('[', ']'),
                MacroDelimiter::Brace => ('{', '}'),
            };
            out.push(open);
            out.push_str(&render_macro_token_trees(tokens));
            out.push(close);
        }
    }
}

fn substitute_macro_template(
    template: &Expr,
    params: &[Name],
    args: &[Expr],
    expansion_id: ExpansionId,
) -> Expr {
    substitute_macro_template_scoped(template, params, args, &[], expansion_id, &mut 0)
}

fn substitute_macro_template_scoped(
    template: &Expr,
    params: &[Name],
    args: &[Expr],
    binders: &[(Name, Name)],
    expansion_id: ExpansionId,
    generated_index: &mut usize,
) -> Expr {
    if let Expr::Variable { name, span } = template {
        if let Some((_, generated)) = binders.iter().rev().find(|(source, _)| source == name) {
            return Expr::Variable {
                name: generated.clone(),
                span: *span,
            };
        }
        if let Some(index) = params.iter().position(|param| param == name) {
            return args[index].clone();
        }
    }

    let mut subst = |expr: &Expr| {
        substitute_macro_template_scoped(expr, params, args, binders, expansion_id, generated_index)
    };

    match template {
        Expr::Unary { op, operand, span } => Expr::Unary {
            op: *op,
            operand: Box::new(subst(operand)),
            span: *span,
        },
        Expr::Binary {
            op,
            raw_operator,
            left,
            right,
            span,
        } => Expr::Binary {
            op: *op,
            raw_operator: raw_operator.clone(),
            left: Box::new(subst(left)),
            right: Box::new(subst(right)),
            span: *span,
        },
        Expr::Call {
            func,
            module,
            args: call_args,
            span,
        } => Expr::Call {
            func: func.clone(),
            module: module.clone(),
            args: call_args.iter().map(subst).collect(),
            span: *span,
        },
        Expr::FnApply {
            func,
            args: apply_args,
            span,
        } => Expr::FnApply {
            func: Box::new(subst(func)),
            args: apply_args.iter().map(subst).collect(),
            span: *span,
        },
        Expr::FieldAccess { base, field, span } => Expr::FieldAccess {
            base: Box::new(subst(base)),
            field: field.clone(),
            span: *span,
        },
        Expr::IndexAccess { base, index, span } => Expr::IndexAccess {
            base: Box::new(subst(base)),
            index: Box::new(subst(index)),
            span: *span,
        },
        Expr::Constructor {
            name,
            fields,
            payload,
            span,
        } => Expr::Constructor {
            name: name.clone(),
            fields: fields
                .iter()
                .map(|(name, expr)| (name.clone(), subst(expr)))
                .collect(),
            payload: substitute_constructor_payload(
                payload,
                params,
                args,
                binders,
                expansion_id,
                generated_index,
            ),
            span: *span,
        },
        Expr::OperatorSection { section } => Expr::OperatorSection {
            section: substitute_operator_section(
                section,
                params,
                args,
                binders,
                expansion_id,
                generated_index,
            ),
        },
        Expr::MacroInvocation { invocation } => Expr::MacroInvocation {
            invocation: MacroInvocation {
                name: invocation.name.clone(),
                delimiter: invocation.delimiter,
                raw_body: invocation.raw_body.clone(),
                body: invocation.body.clone(),
                token_trees: invocation.token_trees.clone(),
                args: invocation
                    .args
                    .as_ref()
                    .map(|macro_args| macro_args.iter().map(subst).collect()),
                span: invocation.span,
            },
        },
        Expr::If {
            condition,
            then_branch,
            else_branch,
            span,
        } => Expr::If {
            condition: Box::new(subst(condition)),
            then_branch: Box::new(subst(then_branch)),
            else_branch: else_branch.as_ref().map(|expr| Box::new(subst(expr))),
            span: *span,
        },
        Expr::Fail { payload, span } => Expr::Fail {
            payload: Box::new(subst(payload)),
            span: *span,
        },
        Expr::List { items, span } => Expr::List {
            items: items.iter().map(subst).collect(),
            span: *span,
        },
        Expr::Block {
            statements,
            tail_expr,
            span,
        } if statements.is_empty() => Expr::Block {
            statements: Vec::new(),
            tail_expr: tail_expr.as_ref().map(|expr| Box::new(subst(expr))),
            span: *span,
        },
        Expr::FnDef {
            params: fn_params,
            return_type,
            body,
            span,
        } => {
            let mut scoped_binders = binders.to_vec();
            let mut generated_params = Vec::with_capacity(fn_params.len());
            for (name, ty) in fn_params {
                let generated: Name = format!(
                    "$ash_generated_macro_{}_{}_{}",
                    expansion_id.0,
                    name.as_ref(),
                    *generated_index
                )
                .into();
                *generated_index += 1;
                scoped_binders.push((name.clone(), generated.clone()));
                generated_params.push((generated, ty.clone()));
            }
            Expr::FnDef {
                params: generated_params,
                return_type: return_type.clone(),
                body: Box::new(substitute_macro_template_scoped(
                    body,
                    params,
                    args,
                    &scoped_binders,
                    expansion_id,
                    generated_index,
                )),
                span: *span,
            }
        }
        other => other.clone(),
    }
}

fn substitute_constructor_payload(
    payload: &ConstructorPayload,
    params: &[Name],
    args: &[Expr],
    binders: &[(Name, Name)],
    expansion_id: ExpansionId,
    generated_index: &mut usize,
) -> ConstructorPayload {
    let mut subst = |expr: &Expr| {
        substitute_macro_template_scoped(expr, params, args, binders, expansion_id, generated_index)
    };
    match payload {
        ConstructorPayload::Unit => ConstructorPayload::Unit,
        ConstructorPayload::Tuple(items) => {
            ConstructorPayload::Tuple(items.iter().map(subst).collect())
        }
        ConstructorPayload::Record(fields) => ConstructorPayload::Record(
            fields
                .iter()
                .map(|(name, expr)| (name.clone(), subst(expr)))
                .collect(),
        ),
    }
}

fn substitute_operator_section(
    section: &OperatorSection,
    params: &[Name],
    args: &[Expr],
    binders: &[(Name, Name)],
    expansion_id: ExpansionId,
    generated_index: &mut usize,
) -> OperatorSection {
    let mut subst = |expr: &Expr| {
        substitute_macro_template_scoped(expr, params, args, binders, expansion_id, generated_index)
    };
    OperatorSection {
        kind: section.kind.clone(),
        operator: section.operator.clone(),
        left: section.left.as_ref().map(|expr| Box::new(subst(expr))),
        right: section.right.as_ref().map(|expr| Box::new(subst(expr))),
        span: section.span,
    }
}

fn ensure_macro_template_supported(
    template: &Expr,
    entry: &LocalMacroEntry,
) -> Result<(), ExpansionError> {
    ensure_macro_template_supported_scoped(template, entry, &[])
}

fn ensure_macro_template_supported_scoped(
    template: &Expr,
    entry: &LocalMacroEntry,
    binders: &[Name],
) -> Result<(), ExpansionError> {
    match template {
        Expr::Match { span, .. } => unsupported_macro_template(entry, *span, "match"),
        Expr::IfLet { span, .. } => unsupported_macro_template(entry, *span, "if-let"),
        Expr::If { span, .. } => unsupported_macro_template(entry, *span, "if"),
        Expr::WithError { span, .. } => unsupported_macro_template(entry, *span, "with_error"),
        Expr::Fail { span, .. } => unsupported_macro_template(entry, *span, "fail"),
        Expr::Block { span, .. } => unsupported_macro_template(entry, *span, "block"),
        Expr::ActBlock { span, .. } => unsupported_macro_template(entry, *span, "act block"),
        Expr::DoBlock { span, .. } => unsupported_macro_template(entry, *span, "do block"),
        Expr::Comprehension { span, .. } => {
            unsupported_macro_template(entry, *span, "comprehension")
        }
        Expr::FnDef { params, body, .. } => {
            let mut scoped_binders = binders.to_vec();
            scoped_binders.extend(params.iter().map(|(name, _)| name.clone()));
            match body.as_ref() {
                Expr::Block {
                    statements,
                    tail_expr: Some(tail),
                    span,
                } if statements.is_empty() => {
                    ensure_macro_template_supported_scoped(tail, entry, &scoped_binders)
                }
                Expr::Block { span, .. } => unsupported_macro_template(entry, *span, "block"),
                body => ensure_macro_template_supported_scoped(body, entry, &scoped_binders),
            }
        }
        Expr::Variable { span, name } => {
            if entry.params.iter().any(|param| param == name)
                || binders.iter().any(|binder| binder == name)
            {
                Ok(())
            } else {
                unsupported_macro_template(entry, *span, "free variable")
            }
        }
        Expr::MacroInvocation { invocation } => {
            if let Some(args) = &invocation.args {
                for arg in args {
                    ensure_macro_template_supported_scoped(arg, entry, binders)?;
                }
            }
            Ok(())
        }
        _ => {
            let mut error = None;
            visit_expr(template, &mut |expr| {
                if error.is_none() && !std::ptr::eq(expr, template) {
                    error = ensure_macro_template_supported_scoped(expr, entry, binders).err();
                }
            });
            error.map_or(Ok(()), Err)
        }
    }
}

fn unsupported_macro_template(
    entry: &LocalMacroEntry,
    span: Span,
    reason: &'static str,
) -> Result<(), ExpansionError> {
    Err(ExpansionError::UnsupportedMacroTemplate {
        span,
        name: entry.name.clone(),
        reason: reason.into(),
    })
}

fn elaborate_operator_sections_in_module(
    module: &mut ModuleFile,
    origins: &mut Vec<ExpandedSurfaceOrigin>,
) -> Result<(), ExpansionError> {
    let table = build_local_notation_table_for_definitions(&module.definitions)?;
    for definition in &mut module.definitions {
        elaborate_operator_sections_in_definition(definition, &table, origins);
    }
    for decl in &mut module.module_decls {
        if let crate::module::ModuleSource::Inline(definitions) = &mut decl.source {
            let inline_table = build_local_notation_table_for_definitions(definitions)?;
            for definition in definitions {
                elaborate_operator_sections_in_definition(definition, &inline_table, origins);
            }
        }
    }
    if let Some(workflow) = &mut module.workflow {
        elaborate_operator_sections_in_workflow_def(workflow, &table, origins);
    }
    Ok(())
}

fn elaborate_operator_sections_in_definition(
    definition: &mut Definition,
    table: &LocalNotationTable,
    origins: &mut Vec<ExpandedSurfaceOrigin>,
) {
    match definition {
        Definition::Capability(def) => {
            for constraint in &mut def.constraints {
                for arg in &mut constraint.predicate.args {
                    elaborate_operator_sections_in_expr(arg, table, origins);
                }
            }
        }
        Definition::CapabilityImplementation(def) => {
            for operation in &mut def.operations {
                elaborate_operator_sections_in_expr(&mut operation.body, table, origins);
            }
        }
        Definition::Policy(def) => {
            if let Some(expr) = &mut def.where_clause {
                elaborate_operator_sections_in_expr(expr, table, origins);
            }
            for field in &mut def.fields {
                if let Some(expr) = &mut field.default {
                    elaborate_operator_sections_in_expr(expr, table, origins);
                }
            }
        }
        Definition::Proxy(def) => {
            elaborate_operator_sections_in_workflow(&mut def.body, table, origins)
        }
        Definition::Interface(def) => {
            for law in &mut def.laws {
                elaborate_operator_sections_in_expr(&mut law.proposition, table, origins);
            }
        }
        Definition::Impl(def) => {
            for method in &mut def.methods {
                elaborate_operator_sections_in_expr(&mut method.body, table, origins);
            }
            for proof in &mut def.proofs {
                elaborate_operator_sections_in_proof(proof, table, origins);
            }
        }
        Definition::Function(def) => {
            elaborate_operator_sections_in_contract(def.contract.as_mut(), table, origins);
            elaborate_operator_sections_in_expr(&mut def.body, table, origins);
        }
        Definition::Law(def) => {
            elaborate_operator_sections_in_expr(&mut def.proposition, table, origins)
        }
        Definition::Proof(def) => elaborate_operator_sections_in_proof(def, table, origins),
        Definition::Macro(_) => {}
        Definition::Notation(_)
        | Definition::CapabilityInterface(_)
        | Definition::ResourceType(_)
        | Definition::Type(_)
        | Definition::DataKind(_)
        | Definition::TypeFn(_)
        | Definition::PropositionPredicate(_)
        | Definition::Role(_)
        | Definition::BuiltinFn(_)
        | Definition::SealedDomain(_) => {}
    }
}

fn elaborate_operator_sections_in_contract(
    contract: Option<&mut Contract>,
    table: &LocalNotationTable,
    origins: &mut Vec<ExpandedSurfaceOrigin>,
) {
    let Some(contract) = contract else {
        return;
    };
    for requirement in &mut contract.requires {
        if let Requirement::Arithmetic { expr } = requirement {
            elaborate_operator_sections_in_expr(expr, table, origins);
        }
    }
    for ensures in &mut contract.ensures {
        elaborate_operator_sections_in_expr(&mut ensures.expr, table, origins);
    }
}

fn elaborate_operator_sections_in_proof(
    proof: &mut ProofDef,
    table: &LocalNotationTable,
    origins: &mut Vec<ExpandedSurfaceOrigin>,
) {
    match &mut proof.body {
        ProofBody::Expr(expr) => elaborate_operator_sections_in_expr(expr, table, origins),
        ProofBody::ByTestProperty { strategies } => {
            for strategy in strategies {
                elaborate_operator_sections_in_expr(&mut strategy.strategy_expr, table, origins);
            }
        }
        ProofBody::ByDefinition | ProofBody::ByTest { .. } | ProofBody::ByTestSmallWorld => {}
    }
}

fn elaborate_operator_sections_in_workflow_def(
    workflow: &mut WorkflowDef,
    table: &LocalNotationTable,
    origins: &mut Vec<ExpandedSurfaceOrigin>,
) {
    for binding in &mut workflow.used_bindings {
        elaborate_operator_sections_in_expr(&mut binding.implementation, table, origins);
    }
    for event in &mut workflow.header_events {
        match event {
            WorkflowHeaderEvent::Uses(binding) => {
                elaborate_operator_sections_in_expr(&mut binding.implementation, table, origins)
            }
            WorkflowHeaderEvent::Requires { expr, .. }
            | WorkflowHeaderEvent::Ensures { expr, .. } => {
                elaborate_operator_sections_in_expr(expr, table, origins)
            }
            WorkflowHeaderEvent::PlaysRole(_)
            | WorkflowHeaderEvent::Capabilities(_)
            | WorkflowHeaderEvent::Owns(_) => {}
        }
    }
    elaborate_operator_sections_in_contract(workflow.contract.as_mut(), table, origins);
    elaborate_operator_sections_in_workflow(&mut workflow.body, table, origins);
}

fn elaborate_operator_sections_in_workflow(
    workflow: &mut Workflow,
    table: &LocalNotationTable,
    origins: &mut Vec<ExpandedSurfaceOrigin>,
) {
    match workflow {
        Workflow::Observe { continuation, .. } | Workflow::Propose { continuation, .. } => {
            if let Some(continuation) = continuation {
                elaborate_operator_sections_in_workflow(continuation, table, origins);
            }
        }
        Workflow::Check {
            target,
            continuation,
            ..
        } => {
            match target {
                CheckTarget::Obligation(obligation) => {
                    elaborate_operator_sections_in_expr(&mut obligation.condition, table, origins)
                }
                CheckTarget::Policy(policy) => {
                    for (_, expr) in &mut policy.fields {
                        elaborate_operator_sections_in_expr(expr, table, origins);
                    }
                }
            }
            if let Some(continuation) = continuation {
                elaborate_operator_sections_in_workflow(continuation, table, origins);
            }
        }
        Workflow::Oblige { .. } | Workflow::Done { .. } => {}
        Workflow::Orient {
            expr, continuation, ..
        } => {
            elaborate_operator_sections_in_expr(expr, table, origins);
            if let Some(continuation) = continuation {
                elaborate_operator_sections_in_workflow(continuation, table, origins);
            }
        }
        Workflow::Decide {
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            elaborate_operator_sections_in_expr(expr, table, origins);
            elaborate_operator_sections_in_workflow(then_branch, table, origins);
            if let Some(else_branch) = else_branch {
                elaborate_operator_sections_in_workflow(else_branch, table, origins);
            }
        }
        Workflow::Act {
            action,
            guard,
            continuation,
            ..
        } => {
            for arg in &mut action.args {
                elaborate_operator_sections_in_expr(arg, table, origins);
            }
            if let Some(guard) = guard {
                elaborate_operator_sections_in_guard(guard, table, origins);
            }
            if let Some(continuation) = continuation {
                elaborate_operator_sections_in_workflow(continuation, table, origins);
            }
        }
        Workflow::Let {
            expr, continuation, ..
        } => {
            elaborate_operator_sections_in_expr(expr, table, origins);
            if let Some(continuation) = continuation {
                elaborate_operator_sections_in_workflow(continuation, table, origins);
            }
        }
        Workflow::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            elaborate_operator_sections_in_expr(condition, table, origins);
            elaborate_operator_sections_in_workflow(then_branch, table, origins);
            if let Some(else_branch) = else_branch {
                elaborate_operator_sections_in_workflow(else_branch, table, origins);
            }
        }
        Workflow::For {
            collection, body, ..
        } => {
            elaborate_operator_sections_in_expr(collection, table, origins);
            elaborate_operator_sections_in_workflow(body, table, origins);
        }
        Workflow::With { body, .. } | Workflow::Must { body, .. } => {
            elaborate_operator_sections_in_workflow(body, table, origins)
        }
        Workflow::Maybe {
            primary, fallback, ..
        } => {
            elaborate_operator_sections_in_workflow(primary, table, origins);
            elaborate_operator_sections_in_workflow(fallback, table, origins);
        }
        Workflow::Seq { first, second, .. } => {
            elaborate_operator_sections_in_workflow(first, table, origins);
            elaborate_operator_sections_in_workflow(second, table, origins);
        }
        Workflow::Ret { expr, .. } | Workflow::Resume { expr, .. } => {
            elaborate_operator_sections_in_expr(expr, table, origins)
        }
        Workflow::Set {
            value,
            continuation,
            ..
        }
        | Workflow::Send {
            value,
            continuation,
            ..
        } => {
            elaborate_operator_sections_in_expr(value, table, origins);
            if let Some(continuation) = continuation {
                elaborate_operator_sections_in_workflow(continuation, table, origins);
            }
        }
        Workflow::Receive { arms, .. } => {
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    elaborate_operator_sections_in_expr(guard, table, origins);
                }
                elaborate_operator_sections_in_workflow(&mut arm.body, table, origins);
            }
        }
        Workflow::Yield { expr, arms, .. } => {
            elaborate_operator_sections_in_expr(expr, table, origins);
            for arm in arms {
                elaborate_operator_sections_in_workflow(&mut arm.body, table, origins);
            }
        }
    }
}

fn elaborate_operator_sections_in_guard(
    guard: &mut Guard,
    table: &LocalNotationTable,
    origins: &mut Vec<ExpandedSurfaceOrigin>,
) {
    match guard {
        Guard::Pred(predicate) => {
            for arg in &mut predicate.args {
                elaborate_operator_sections_in_expr(arg, table, origins);
            }
        }
        Guard::And(left, right) | Guard::Or(left, right) => {
            elaborate_operator_sections_in_guard(left, table, origins);
            elaborate_operator_sections_in_guard(right, table, origins);
        }
        Guard::Not(inner) => elaborate_operator_sections_in_guard(inner, table, origins),
        Guard::Always | Guard::Never => {}
    }
}

fn elaborate_operator_sections_in_expr(
    expr: &mut Expr,
    table: &LocalNotationTable,
    origins: &mut Vec<ExpandedSurfaceOrigin>,
) {
    elaborate_operator_sections_in_expr_with_parent(expr, table, origins, None);
}

fn elaborate_operator_sections_in_expr_with_parent(
    expr: &mut Expr,
    table: &LocalNotationTable,
    origins: &mut Vec<ExpandedSurfaceOrigin>,
    parent_origin: Option<&SurfaceOrigin>,
) {
    match expr {
        Expr::OperatorSection { section } => {
            let expansion_id = next_expansion_id(origins);
            let (elaborated, origin) =
                elaborate_operator_section(section.clone(), table, expansion_id);
            *expr = elaborated;
            if let Some(origin) = origin {
                let nested_parent = origin.origin.clone();
                push_expanded_origin(origins, origin, parent_origin.cloned());
                if !matches!(expr, Expr::OperatorSection { .. }) {
                    elaborate_operator_sections_in_expr_with_parent(
                        expr,
                        table,
                        origins,
                        Some(&nested_parent),
                    );
                }
            } else if !matches!(expr, Expr::OperatorSection { .. }) {
                elaborate_operator_sections_in_expr_with_parent(
                    expr,
                    table,
                    origins,
                    parent_origin,
                );
            }
        }
        Expr::FieldAccess { base, .. } => {
            elaborate_operator_sections_in_expr_with_parent(base, table, origins, parent_origin)
        }
        Expr::IndexAccess { base, index, .. } => {
            elaborate_operator_sections_in_expr_with_parent(base, table, origins, parent_origin);
            elaborate_operator_sections_in_expr_with_parent(index, table, origins, parent_origin);
        }
        Expr::Unary { operand, .. } => {
            elaborate_operator_sections_in_expr_with_parent(operand, table, origins, parent_origin)
        }
        Expr::Binary { left, right, .. } => {
            elaborate_operator_sections_in_expr_with_parent(left, table, origins, parent_origin);
            elaborate_operator_sections_in_expr_with_parent(right, table, origins, parent_origin);
        }
        Expr::Call { args, .. } => {
            for arg in args {
                elaborate_operator_sections_in_expr_with_parent(arg, table, origins, parent_origin);
            }
        }
        Expr::MacroInvocation { .. } => {}
        Expr::Match {
            scrutinee, arms, ..
        } => {
            elaborate_operator_sections_in_expr_with_parent(
                scrutinee,
                table,
                origins,
                parent_origin,
            );
            for arm in arms {
                elaborate_operator_sections_in_expr_with_parent(
                    &mut arm.body,
                    table,
                    origins,
                    parent_origin,
                );
            }
        }
        Expr::IfLet {
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            elaborate_operator_sections_in_expr_with_parent(expr, table, origins, parent_origin);
            elaborate_operator_sections_in_expr_with_parent(
                then_branch,
                table,
                origins,
                parent_origin,
            );
            elaborate_operator_sections_in_expr_with_parent(
                else_branch,
                table,
                origins,
                parent_origin,
            );
        }
        Expr::Constructor {
            fields, payload, ..
        } => {
            for (_, expr) in fields {
                elaborate_operator_sections_in_expr_with_parent(
                    expr,
                    table,
                    origins,
                    parent_origin,
                );
            }
            match payload {
                ConstructorPayload::Tuple(items) => {
                    for item in items {
                        elaborate_operator_sections_in_expr_with_parent(
                            item,
                            table,
                            origins,
                            parent_origin,
                        );
                    }
                }
                ConstructorPayload::Record(fields) => {
                    for (_, expr) in fields {
                        elaborate_operator_sections_in_expr_with_parent(
                            expr,
                            table,
                            origins,
                            parent_origin,
                        );
                    }
                }
                ConstructorPayload::Unit => {}
            }
        }
        Expr::Record { fields, .. } => {
            for (_, expr) in fields {
                elaborate_operator_sections_in_expr_with_parent(
                    expr,
                    table,
                    origins,
                    parent_origin,
                );
            }
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            elaborate_operator_sections_in_expr_with_parent(
                condition,
                table,
                origins,
                parent_origin,
            );
            elaborate_operator_sections_in_expr_with_parent(
                then_branch,
                table,
                origins,
                parent_origin,
            );
            if let Some(else_branch) = else_branch {
                elaborate_operator_sections_in_expr_with_parent(
                    else_branch,
                    table,
                    origins,
                    parent_origin,
                );
            }
        }
        Expr::Fail { payload, .. } => {
            elaborate_operator_sections_in_expr_with_parent(payload, table, origins, parent_origin)
        }
        Expr::WithError { body, arms, .. } => {
            elaborate_operator_sections_in_expr_with_parent(body, table, origins, parent_origin);
            for arm in arms {
                elaborate_operator_sections_in_expr_with_parent(
                    &mut arm.body,
                    table,
                    origins,
                    parent_origin,
                );
            }
        }
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            for stmt in statements {
                match stmt {
                    BlockStmt::Let { expr, .. } => elaborate_operator_sections_in_expr_with_parent(
                        expr,
                        table,
                        origins,
                        parent_origin,
                    ),
                }
            }
            if let Some(tail_expr) = tail_expr {
                elaborate_operator_sections_in_expr_with_parent(
                    tail_expr,
                    table,
                    origins,
                    parent_origin,
                );
            }
        }
        Expr::FnDef { body, .. } => {
            elaborate_operator_sections_in_expr_with_parent(body, table, origins, parent_origin)
        }
        Expr::FnApply { func, args, .. } => {
            elaborate_operator_sections_in_expr_with_parent(func, table, origins, parent_origin);
            for arg in args {
                elaborate_operator_sections_in_expr_with_parent(arg, table, origins, parent_origin);
            }
        }
        Expr::ActBlock { stmts, .. } => {
            for stmt in stmts {
                match stmt {
                    ActStmt::Bind { value, .. } | ActStmt::Return { value, .. } => {
                        elaborate_operator_sections_in_expr_with_parent(
                            value,
                            table,
                            origins,
                            parent_origin,
                        )
                    }
                }
            }
        }
        Expr::DoBlock { stmts, .. } => {
            for stmt in stmts {
                match stmt {
                    DoStmt::Let { value, .. }
                    | DoStmt::Bind { value, .. }
                    | DoStmt::WorkflowRequires { expr: value, .. }
                    | DoStmt::WorkflowEnsures { expr: value, .. }
                    | DoStmt::Return { value, .. } => {
                        elaborate_operator_sections_in_expr_with_parent(
                            value,
                            table,
                            origins,
                            parent_origin,
                        )
                    }
                }
            }
        }
        Expr::Comprehension {
            result, qualifiers, ..
        } => {
            elaborate_operator_sections_in_expr_with_parent(result, table, origins, parent_origin);
            for qualifier in qualifiers {
                match qualifier {
                    ComprehensionQualifier::Bind { value, .. }
                    | ComprehensionQualifier::DiscardBind { value, .. }
                    | ComprehensionQualifier::Let { value, .. } => {
                        elaborate_operator_sections_in_expr_with_parent(
                            value,
                            table,
                            origins,
                            parent_origin,
                        )
                    }
                }
            }
        }
        Expr::List { items, .. } => {
            for item in items {
                elaborate_operator_sections_in_expr_with_parent(
                    item,
                    table,
                    origins,
                    parent_origin,
                );
            }
        }
        Expr::Literal(_)
        | Expr::Variable { .. }
        | Expr::Policy(_)
        | Expr::CheckObligation { .. }
        | Expr::Panic { .. } => {}
    }
}

fn push_expanded_origin(
    origins: &mut Vec<ExpandedSurfaceOrigin>,
    mut origin: ExpandedSurfaceOrigin,
    parent: Option<SurfaceOrigin>,
) {
    origin.expansion_id = next_expansion_id(origins);
    origin.parent = parent.map(Box::new);
    origins.push(origin);
}

fn next_expansion_id(origins: &[ExpandedSurfaceOrigin]) -> ExpansionId {
    ExpansionId(
        origins
            .len()
            .try_into()
            .expect("expansion origin count exceeded u32::MAX"),
    )
}

fn collect_identifier_hygiene_metadata(
    module: &ModuleFile,
    _origins: &[ExpandedSurfaceOrigin],
) -> Vec<IdentifierHygieneMetadata> {
    let mut metadata = Vec::new();
    for definition in &module.definitions {
        collect_definition_hygiene_metadata(definition, &mut metadata);
    }
    for decl in &module.module_decls {
        if let crate::module::ModuleSource::Inline(definitions) = &decl.source {
            for definition in definitions {
                collect_definition_hygiene_metadata(definition, &mut metadata);
            }
        }
    }
    if let Some(workflow) = &module.workflow {
        collect_workflow_hygiene_metadata(workflow, &mut metadata);
    }
    metadata
}

fn collect_definition_hygiene_metadata(
    definition: &Definition,
    metadata: &mut Vec<IdentifierHygieneMetadata>,
) {
    match definition {
        Definition::Function(def) => {
            for param in &def.params {
                push_binder_hygiene(metadata, param.name.clone(), def.span);
            }
            collect_expr_hygiene_metadata(&def.body, metadata);
        }
        Definition::BuiltinFn(def) => {
            for param in &def.params {
                push_binder_hygiene(metadata, param.name.clone(), def.span);
            }
        }
        Definition::Capability(def) => {
            for param in &def.params {
                push_binder_hygiene(metadata, param.name.clone(), def.span);
            }
        }
        Definition::Impl(def) => {
            for method in &def.methods {
                for param in &method.params {
                    push_binder_hygiene(metadata, param.clone(), method.span);
                }
                collect_expr_hygiene_metadata(&method.body, metadata);
            }
            for proof in &def.proofs {
                collect_proof_hygiene_metadata(proof, metadata);
            }
        }
        Definition::Law(law) => collect_expr_hygiene_metadata(&law.proposition, metadata),
        Definition::Proof(proof) => collect_proof_hygiene_metadata(proof, metadata),
        Definition::Type(_)
        | Definition::CapabilityInterface(_)
        | Definition::CapabilityImplementation(_)
        | Definition::ResourceType(_)
        | Definition::SealedDomain(_)
        | Definition::Interface(_)
        | Definition::PropositionPredicate(_)
        | Definition::Role(_)
        | Definition::Policy(_)
        | Definition::Proxy(_)
        | Definition::Notation(_)
        | Definition::Macro(_)
        | Definition::TypeFn(_)
        | Definition::DataKind(_) => {}
    }
}

fn collect_workflow_hygiene_metadata(
    workflow: &WorkflowDef,
    metadata: &mut Vec<IdentifierHygieneMetadata>,
) {
    for param in &workflow.params {
        push_binder_hygiene(metadata, param.name.clone(), workflow.span);
    }
    visit_exprs_in_workflow(&workflow.body, &mut |expr| {
        collect_expr_hygiene_metadata(expr, metadata);
    });
}

fn collect_proof_hygiene_metadata(proof: &ProofDef, metadata: &mut Vec<IdentifierHygieneMetadata>) {
    match &proof.body {
        ProofBody::Expr(expr) => collect_expr_hygiene_metadata(expr, metadata),
        ProofBody::ByTestProperty { strategies } => {
            for strategy in strategies {
                collect_expr_hygiene_metadata(&strategy.strategy_expr, metadata);
            }
        }
        ProofBody::ByDefinition | ProofBody::ByTest { .. } | ProofBody::ByTestSmallWorld => {}
    }
}

fn collect_expr_hygiene_metadata(expr: &Expr, metadata: &mut Vec<IdentifierHygieneMetadata>) {
    visit_expr(expr, &mut |expr| match expr {
        Expr::Variable { name, span } => metadata.push(IdentifierHygieneMetadata {
            name: name.clone(),
            span: *span,
            context: IdentifierHygieneContext::CallSite,
            expansion_id: generated_identifier_expansion_id(name),
        }),
        Expr::FnDef { params, span, .. } => {
            for (name, _) in params {
                push_binder_hygiene(metadata, name.clone(), *span);
            }
        }
        _ => {}
    });
}

fn push_binder_hygiene(metadata: &mut Vec<IdentifierHygieneMetadata>, name: Name, span: Span) {
    if let Some(expansion_id) = generated_identifier_expansion_id(&name) {
        metadata.push(IdentifierHygieneMetadata {
            name,
            span,
            context: IdentifierHygieneContext::Generated,
            expansion_id: Some(expansion_id),
        });
    } else {
        metadata.push(IdentifierHygieneMetadata {
            name,
            span,
            context: IdentifierHygieneContext::DefinitionSite,
            expansion_id: None,
        });
    }
}

fn generated_identifier_expansion_id(name: &str) -> Option<ExpansionId> {
    for prefix in ["$ash_generated_section_", "$ash_generated_macro_"] {
        let Some(rest) = name.strip_prefix(prefix) else {
            continue;
        };
        let (id, _) = rest.split_once('_')?;
        return id.parse::<u32>().ok().map(ExpansionId);
    }
    None
}

fn elaborate_operator_section(
    section: OperatorSection,
    table: &LocalNotationTable,
    expansion_id: ExpansionId,
) -> (Expr, Option<ExpandedSurfaceOrigin>) {
    let target = table.resolve_infix(section.operator.spelling.as_ref());
    match section.kind {
        OperatorSectionKind::Bare => match target {
            Some(entry) => {
                notation_section_expansion(section.span, entry, None, None, expansion_id)
            }
            None => match builtin_binary_op(section.operator.spelling.as_ref()) {
                Some(op) => builtin_section_expansion(
                    section.span,
                    section.operator.clone(),
                    op,
                    None,
                    None,
                    expansion_id,
                ),
                None => (Expr::OperatorSection { section }, None),
            },
        },
        OperatorSectionKind::Left => {
            let Some(left) = section.left.clone().map(|expr| *expr) else {
                return (Expr::OperatorSection { section }, None);
            };
            match target {
                Some(entry) => {
                    notation_section_expansion(section.span, entry, Some(left), None, expansion_id)
                }
                None => match builtin_binary_op(section.operator.spelling.as_ref()) {
                    Some(op) => builtin_section_expansion(
                        section.span,
                        section.operator.clone(),
                        op,
                        Some(left),
                        None,
                        expansion_id,
                    ),
                    None => (Expr::OperatorSection { section }, None),
                },
            }
        }
        OperatorSectionKind::Right => {
            let Some(right) = section.right.clone().map(|expr| *expr) else {
                return (Expr::OperatorSection { section }, None);
            };
            match target {
                Some(entry) => {
                    notation_section_expansion(section.span, entry, None, Some(right), expansion_id)
                }
                None => match builtin_binary_op(section.operator.spelling.as_ref()) {
                    Some(op) => builtin_section_expansion(
                        section.span,
                        section.operator.clone(),
                        op,
                        None,
                        Some(right),
                        expansion_id,
                    ),
                    None => (Expr::OperatorSection { section }, None),
                },
            }
        }
    }
}

fn builtin_section_expansion(
    span: Span,
    raw_operator: RawOperatorToken,
    op: BinaryOp,
    left: Option<Expr>,
    right: Option<Expr>,
    expansion_id: ExpansionId,
) -> (Expr, Option<ExpandedSurfaceOrigin>) {
    let operator_span = raw_operator.span;
    let expr = eta_binary_section(span, raw_operator, op, left, right, expansion_id);
    (
        expr,
        Some(ExpandedSurfaceOrigin {
            expansion_id: ExpansionId(0),
            generated_span: span,
            origin: SurfaceOrigin::OperatorSection {
                section_span: span,
                operator_span,
            },
            parent: None,
        }),
    )
}

fn notation_section_expansion(
    span: Span,
    entry: &LocalNotationEntry,
    left: Option<Expr>,
    right: Option<Expr>,
    expansion_id: ExpansionId,
) -> (Expr, Option<ExpandedSurfaceOrigin>) {
    let expr = eta_local_section(span, entry.target.clone(), left, right, expansion_id);
    (
        expr,
        Some(ExpandedSurfaceOrigin {
            expansion_id: ExpansionId(0),
            generated_span: span,
            origin: SurfaceOrigin::NotationExpansion {
                notation_span: entry.span,
                target: render_callable_path(&entry.target).into_boxed_str(),
            },
            parent: None,
        }),
    )
}

fn render_callable_path(path: &CallablePath) -> String {
    match &path.module {
        Some(module) => format!("{}::{}", module.as_ref(), path.name.as_ref()),
        None => path.name.to_string(),
    }
}

fn builtin_binary_op(operator: &str) -> Option<BinaryOp> {
    match operator {
        "+" => Some(BinaryOp::Add),
        "-" => Some(BinaryOp::Sub),
        "*" => Some(BinaryOp::Mul),
        "/" => Some(BinaryOp::Div),
        "%" => Some(BinaryOp::Mod),
        "==" => Some(BinaryOp::Eq),
        "!=" => Some(BinaryOp::Neq),
        "<" => Some(BinaryOp::Lt),
        ">" => Some(BinaryOp::Gt),
        "<=" => Some(BinaryOp::Leq),
        ">=" => Some(BinaryOp::Geq),
        "&&" => Some(BinaryOp::And),
        "||" => Some(BinaryOp::Or),
        _ => None,
    }
}

fn eta_binary_section(
    span: Span,
    raw_operator: RawOperatorToken,
    op: BinaryOp,
    left: Option<Expr>,
    right: Option<Expr>,
    expansion_id: ExpansionId,
) -> Expr {
    let lhs_name: Name = generated_section_name(expansion_id, "lhs").into();
    let rhs_name: Name = generated_section_name(expansion_id, "rhs").into();
    let left_missing = left.is_none();
    let right_missing = right.is_none();
    let left_expr = left.unwrap_or_else(|| Expr::Variable {
        name: lhs_name.clone(),
        span,
    });
    let right_expr = right.unwrap_or_else(|| Expr::Variable {
        name: rhs_name.clone(),
        span,
    });
    let mut params = Vec::new();
    if left_missing && right_missing {
        params.push((lhs_name, None));
        params.push((rhs_name, None));
    } else if left_missing {
        params.push((lhs_name, None));
    } else if right_missing {
        params.push((rhs_name, None));
    }
    Expr::FnDef {
        params,
        return_type: None,
        body: Box::new(Expr::Binary {
            op,
            raw_operator: Some(raw_operator),
            left: Box::new(left_expr),
            right: Box::new(right_expr),
            span,
        }),
        span,
    }
}

fn eta_local_section(
    span: Span,
    target: CallablePath,
    left: Option<Expr>,
    right: Option<Expr>,
    expansion_id: ExpansionId,
) -> Expr {
    let lhs_name: Name = generated_section_name(expansion_id, "lhs").into();
    let rhs_name: Name = generated_section_name(expansion_id, "rhs").into();
    let left_expr = left.unwrap_or_else(|| Expr::Variable {
        name: lhs_name.clone(),
        span,
    });
    let right_expr = right.unwrap_or_else(|| Expr::Variable {
        name: rhs_name.clone(),
        span,
    });
    let mut params = Vec::new();
    if !matches!(left_expr, Expr::Variable { ref name, .. } if name == &lhs_name) {
        params.push((rhs_name, None));
    } else if !matches!(right_expr, Expr::Variable { ref name, .. } if name == &rhs_name) {
        params.push((lhs_name, None));
    } else {
        params.push((lhs_name, None));
        params.push((rhs_name, None));
    }
    Expr::FnDef {
        params,
        return_type: None,
        body: Box::new(Expr::Call {
            func: target.name,
            module: target.module,
            args: vec![left_expr, right_expr],
            span,
        }),
        span,
    }
}

fn generated_section_name(expansion_id: ExpansionId, role: &str) -> String {
    format!("$ash_generated_section_{}_{}", expansion_id.0, role)
}

fn find_operator_section_in_module(module: &ModuleFile) -> Option<&OperatorSection> {
    let mut found = None;
    visit_expanded_boundary_exprs_in_module(module, &mut |expr| match expr {
        Expr::OperatorSection { section } if found.is_none() => found = Some(section),
        _ => {}
    });
    found
}

fn find_macro_invocation_in_module(module: &ModuleFile) -> Option<&MacroInvocation> {
    let mut found = None;
    visit_expanded_boundary_exprs_in_module(module, &mut |expr| match expr {
        Expr::MacroInvocation { invocation } if found.is_none() => found = Some(invocation),
        _ => {}
    });
    found
}

fn visit_expanded_boundary_exprs_in_module<'a, F>(module: &'a ModuleFile, visitor: &mut F)
where
    F: FnMut(&'a Expr),
{
    for definition in &module.definitions {
        if !matches!(definition, Definition::Macro(_)) {
            visit_exprs_in_definition(definition, visitor);
        }
    }
    for decl in &module.module_decls {
        if let crate::module::ModuleSource::Inline(definitions) = &decl.source {
            for definition in definitions {
                if !matches!(definition, Definition::Macro(_)) {
                    visit_exprs_in_definition(definition, visitor);
                }
            }
        }
    }
    if let Some(workflow) = &module.workflow {
        visit_exprs_in_workflow_def(workflow, visitor);
    }
}

/// Visit every expression-bearing surface reachable from a module file.
///
/// This is the read-only expansion traversal used by notation diagnostics and by the
/// expanded-surface fail-closed boundary. It intentionally enumerates surface variants
/// rather than using catch-all wildcard arms so new expression-bearing variants require
/// an explicit traversal decision.
pub fn visit_exprs_in_module<'a, F>(module: &'a ModuleFile, visitor: &mut F)
where
    F: FnMut(&'a Expr),
{
    for definition in &module.definitions {
        visit_exprs_in_definition(definition, visitor);
    }
    for decl in &module.module_decls {
        if let crate::module::ModuleSource::Inline(definitions) = &decl.source {
            for definition in definitions {
                visit_exprs_in_definition(definition, visitor);
            }
        }
    }
    if let Some(workflow) = &module.workflow {
        visit_exprs_in_workflow_def(workflow, visitor);
    }
}

/// Visit every expression-bearing surface reachable from a definition.
pub fn visit_exprs_in_definition<'a, F>(definition: &'a Definition, visitor: &mut F)
where
    F: FnMut(&'a Expr),
{
    match definition {
        Definition::Capability(def) => {
            for constraint in &def.constraints {
                visit_exprs_in_predicate(&constraint.predicate, visitor);
            }
        }
        Definition::CapabilityImplementation(def) => {
            for operation in &def.operations {
                visit_expr(&operation.body, visitor);
            }
        }
        Definition::Policy(def) => {
            if let Some(expr) = &def.where_clause {
                visit_expr(expr, visitor);
            }
            for field in &def.fields {
                if let Some(expr) = &field.default {
                    visit_expr(expr, visitor);
                }
            }
        }
        Definition::Proxy(def) => visit_exprs_in_workflow(&def.body, visitor),
        Definition::Interface(def) => {
            for law in &def.laws {
                visit_expr(&law.proposition, visitor);
            }
        }
        Definition::Impl(def) => {
            for method in &def.methods {
                visit_expr(&method.body, visitor);
            }
            for proof in &def.proofs {
                visit_exprs_in_proof(proof, visitor);
            }
        }
        Definition::Function(def) => {
            visit_exprs_in_contract(def.contract.as_ref(), visitor);
            visit_expr(&def.body, visitor);
        }
        Definition::Law(def) => visit_expr(&def.proposition, visitor),
        Definition::Proof(def) => visit_exprs_in_proof(def, visitor),
        Definition::Macro(def) => visit_expr(&def.body, visitor),
        Definition::Notation(_)
        | Definition::CapabilityInterface(_)
        | Definition::ResourceType(_)
        | Definition::Type(_)
        | Definition::DataKind(_)
        | Definition::TypeFn(_)
        | Definition::PropositionPredicate(_)
        | Definition::Role(_)
        | Definition::BuiltinFn(_)
        | Definition::SealedDomain(_) => {}
    }
}

fn visit_exprs_in_workflow_def<'a, F>(workflow: &'a WorkflowDef, visitor: &mut F)
where
    F: FnMut(&'a Expr),
{
    for binding in &workflow.used_bindings {
        visit_expr(&binding.implementation, visitor);
    }
    for event in &workflow.header_events {
        match event {
            WorkflowHeaderEvent::Uses(binding) => visit_expr(&binding.implementation, visitor),
            WorkflowHeaderEvent::Requires { expr, .. }
            | WorkflowHeaderEvent::Ensures { expr, .. } => visit_expr(expr, visitor),
            WorkflowHeaderEvent::PlaysRole(_)
            | WorkflowHeaderEvent::Capabilities(_)
            | WorkflowHeaderEvent::Owns(_) => {}
        }
    }
    visit_exprs_in_contract(workflow.contract.as_ref(), visitor);
    visit_exprs_in_workflow(&workflow.body, visitor);
}

fn visit_exprs_in_contract<'a, F>(contract: Option<&'a Contract>, visitor: &mut F)
where
    F: FnMut(&'a Expr),
{
    let Some(contract) = contract else {
        return;
    };
    for requirement in &contract.requires {
        match requirement {
            Requirement::Arithmetic { expr } => visit_expr(expr, visitor),
            Requirement::HasCapability { .. } | Requirement::HasRole(_) => {}
        }
    }
    for ensures in &contract.ensures {
        visit_expr(&ensures.expr, visitor);
    }
}

fn visit_exprs_in_proof<'a, F>(proof: &'a ProofDef, visitor: &mut F)
where
    F: FnMut(&'a Expr),
{
    match &proof.body {
        ProofBody::Expr(expr) => visit_expr(expr, visitor),
        ProofBody::ByTestProperty { strategies } => {
            for strategy in strategies {
                visit_expr(&strategy.strategy_expr, visitor);
            }
        }
        ProofBody::ByDefinition | ProofBody::ByTest { .. } | ProofBody::ByTestSmallWorld => {}
    }
}

fn visit_exprs_in_predicate<'a, F>(predicate: &'a Predicate, visitor: &mut F)
where
    F: FnMut(&'a Expr),
{
    for arg in &predicate.args {
        visit_expr(arg, visitor);
    }
}

fn visit_exprs_in_guard<'a, F>(guard: &'a Guard, visitor: &mut F)
where
    F: FnMut(&'a Expr),
{
    match guard {
        Guard::Pred(predicate) => visit_exprs_in_predicate(predicate, visitor),
        Guard::And(left, right) | Guard::Or(left, right) => {
            visit_exprs_in_guard(left, visitor);
            visit_exprs_in_guard(right, visitor);
        }
        Guard::Not(inner) => visit_exprs_in_guard(inner, visitor),
        Guard::Always | Guard::Never => {}
    }
}

fn visit_exprs_in_action<'a, F>(action: &'a ActionRef, visitor: &mut F)
where
    F: FnMut(&'a Expr),
{
    for arg in &action.args {
        visit_expr(arg, visitor);
    }
}

fn visit_exprs_in_check_target<'a, F>(target: &'a CheckTarget, visitor: &mut F)
where
    F: FnMut(&'a Expr),
{
    match target {
        CheckTarget::Obligation(obligation) => visit_expr(&obligation.condition, visitor),
        CheckTarget::Policy(policy) => {
            for (_, expr) in &policy.fields {
                visit_expr(expr, visitor);
            }
        }
    }
}

fn visit_exprs_in_workflow<'a, F>(workflow: &'a Workflow, visitor: &mut F)
where
    F: FnMut(&'a Expr),
{
    match workflow {
        Workflow::Observe { continuation, .. } | Workflow::Propose { continuation, .. } => {
            if let Some(continuation) = continuation {
                visit_exprs_in_workflow(continuation, visitor);
            }
        }
        Workflow::Check {
            target,
            continuation,
            ..
        } => {
            visit_exprs_in_check_target(target, visitor);
            if let Some(continuation) = continuation {
                visit_exprs_in_workflow(continuation, visitor);
            }
        }
        Workflow::Oblige { .. } | Workflow::Done { .. } => {}
        Workflow::Orient {
            expr, continuation, ..
        } => {
            visit_expr(expr, visitor);
            if let Some(continuation) = continuation {
                visit_exprs_in_workflow(continuation, visitor);
            }
        }
        Workflow::Decide {
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            visit_expr(expr, visitor);
            visit_exprs_in_workflow(then_branch, visitor);
            if let Some(else_branch) = else_branch {
                visit_exprs_in_workflow(else_branch, visitor);
            }
        }
        Workflow::Act {
            action,
            guard,
            continuation,
            ..
        } => {
            visit_exprs_in_action(action, visitor);
            if let Some(guard) = guard {
                visit_exprs_in_guard(guard, visitor);
            }
            if let Some(continuation) = continuation {
                visit_exprs_in_workflow(continuation, visitor);
            }
        }
        Workflow::Let {
            expr, continuation, ..
        } => {
            visit_expr(expr, visitor);
            if let Some(continuation) = continuation {
                visit_exprs_in_workflow(continuation, visitor);
            }
        }
        Workflow::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            visit_expr(condition, visitor);
            visit_exprs_in_workflow(then_branch, visitor);
            if let Some(else_branch) = else_branch {
                visit_exprs_in_workflow(else_branch, visitor);
            }
        }
        Workflow::For {
            collection, body, ..
        } => {
            visit_expr(collection, visitor);
            visit_exprs_in_workflow(body, visitor);
        }
        Workflow::With { body, .. } | Workflow::Must { body, .. } => {
            visit_exprs_in_workflow(body, visitor);
        }
        Workflow::Maybe {
            primary, fallback, ..
        } => {
            visit_exprs_in_workflow(primary, visitor);
            visit_exprs_in_workflow(fallback, visitor);
        }
        Workflow::Seq { first, second, .. } => {
            visit_exprs_in_workflow(first, visitor);
            visit_exprs_in_workflow(second, visitor);
        }
        Workflow::Ret { expr, .. } | Workflow::Resume { expr, .. } => visit_expr(expr, visitor),
        Workflow::Set {
            value,
            continuation,
            ..
        }
        | Workflow::Send {
            value,
            continuation,
            ..
        } => {
            visit_expr(value, visitor);
            if let Some(continuation) = continuation {
                visit_exprs_in_workflow(continuation, visitor);
            }
        }
        Workflow::Receive { arms, .. } => {
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    visit_expr(guard, visitor);
                }
                visit_exprs_in_workflow(&arm.body, visitor);
            }
        }
        Workflow::Yield { expr, arms, .. } => {
            visit_expr(expr, visitor);
            for arm in arms {
                visit_exprs_in_workflow(&arm.body, visitor);
            }
        }
    }
}

/// Visit an expression and all child expressions in source order.
pub fn visit_expr<'a, F>(expr: &'a Expr, visitor: &mut F)
where
    F: FnMut(&'a Expr),
{
    visitor(expr);
    match expr {
        Expr::OperatorSection { section } => {
            if let Some(left) = &section.left {
                visit_expr(left, visitor);
            }
            if let Some(right) = &section.right {
                visit_expr(right, visitor);
            }
        }
        Expr::FieldAccess { base, .. } => visit_expr(base, visitor),
        Expr::IndexAccess { base, index, .. } => {
            visit_expr(base, visitor);
            visit_expr(index, visitor);
        }
        Expr::Unary { operand, .. } => visit_expr(operand, visitor),
        Expr::Binary { left, right, .. } => {
            visit_expr(left, visitor);
            visit_expr(right, visitor);
        }
        Expr::Call { args, .. } => {
            for arg in args {
                visit_expr(arg, visitor);
            }
        }
        Expr::MacroInvocation { .. } => {}
        Expr::Match {
            scrutinee, arms, ..
        } => {
            visit_expr(scrutinee, visitor);
            for arm in arms {
                visit_expr(&arm.body, visitor);
            }
        }
        Expr::IfLet {
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            visit_expr(expr, visitor);
            visit_expr(then_branch, visitor);
            visit_expr(else_branch, visitor);
        }
        Expr::Constructor {
            fields, payload, ..
        } => {
            for (_, expr) in fields {
                visit_expr(expr, visitor);
            }
            match payload {
                ConstructorPayload::Tuple(items) => {
                    for item in items {
                        visit_expr(item, visitor);
                    }
                }
                ConstructorPayload::Record(fields) => {
                    for (_, expr) in fields {
                        visit_expr(expr, visitor);
                    }
                }
                ConstructorPayload::Unit => {}
            }
        }
        Expr::Record { fields, .. } => {
            for (_, expr) in fields {
                visit_expr(expr, visitor);
            }
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            visit_expr(condition, visitor);
            visit_expr(then_branch, visitor);
            if let Some(else_branch) = else_branch {
                visit_expr(else_branch, visitor);
            }
        }
        Expr::Fail { payload, .. } => visit_expr(payload, visitor),
        Expr::WithError { body, arms, .. } => {
            visit_expr(body, visitor);
            for arm in arms {
                visit_expr(&arm.body, visitor);
            }
        }
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            for stmt in statements {
                match stmt {
                    BlockStmt::Let { expr, .. } => visit_expr(expr, visitor),
                }
            }
            if let Some(tail_expr) = tail_expr {
                visit_expr(tail_expr, visitor);
            }
        }
        Expr::FnDef { body, .. } => visit_expr(body, visitor),
        Expr::FnApply { func, args, .. } => {
            visit_expr(func, visitor);
            for arg in args {
                visit_expr(arg, visitor);
            }
        }
        Expr::ActBlock { stmts, .. } => {
            for stmt in stmts {
                match stmt {
                    ActStmt::Bind { value, .. } | ActStmt::Return { value, .. } => {
                        visit_expr(value, visitor)
                    }
                }
            }
        }
        Expr::DoBlock { stmts, .. } => {
            for stmt in stmts {
                match stmt {
                    DoStmt::Let { value, .. }
                    | DoStmt::Bind { value, .. }
                    | DoStmt::WorkflowRequires { expr: value, .. }
                    | DoStmt::WorkflowEnsures { expr: value, .. }
                    | DoStmt::Return { value, .. } => visit_expr(value, visitor),
                }
            }
        }
        Expr::Comprehension {
            result, qualifiers, ..
        } => {
            visit_expr(result, visitor);
            for qualifier in qualifiers {
                match qualifier {
                    ComprehensionQualifier::Bind { value, .. }
                    | ComprehensionQualifier::DiscardBind { value, .. }
                    | ComprehensionQualifier::Let { value, .. } => visit_expr(value, visitor),
                }
            }
        }
        Expr::List { items, .. } => {
            for item in items {
                visit_expr(item, visitor);
            }
        }
        Expr::Literal(_)
        | Expr::Variable { .. }
        | Expr::Policy(_)
        | Expr::CheckObligation { .. }
        | Expr::Panic { .. } => {}
    }
}

/// Expression types.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Source-preserving binary infix operator section.
    ///
    /// This is parsed surface syntax only. It must be resolved to an ordinary
    /// callable value before Core lowering.
    OperatorSection {
        /// Operator-section payload preserving section kind, operator token, and spans.
        section: OperatorSection,
    },
    /// Literal value
    Literal(Literal),
    /// Variable reference
    Variable { name: Name, span: Span },
    /// Field access: base.field
    FieldAccess {
        /// Base expression
        base: Box<Expr>,
        /// Field name
        field: Name,
        /// Source span
        span: Span,
    },
    /// Index access: base\[index\]
    IndexAccess {
        /// Base expression
        base: Box<Expr>,
        /// Index expression
        index: Box<Expr>,
        /// Source span
        span: Span,
    },
    /// Unary operation
    Unary {
        /// Unary operator
        op: UnaryOp,
        /// Operand
        operand: Box<Expr>,
        /// Source span
        span: Span,
    },
    /// Binary operation
    Binary {
        /// Binary operator
        op: BinaryOp,
        /// Raw source operator token, when available from parsed syntax.
        raw_operator: Option<RawOperatorToken>,
        /// Left operand
        left: Box<Expr>,
        /// Right operand
        right: Box<Expr>,
        /// Source span
        span: Span,
    },
    /// Function call
    Call {
        /// Function name
        func: Name,
        /// Optional module qualifier (e.g., `module` in `module::name(args)`)
        module: Option<Name>,
        /// Arguments
        args: Vec<Expr>,
        /// Source span
        span: Span,
    },
    /// Macro invocation parsed for future macro work but rejected before Core lowering.
    MacroInvocation {
        /// Macro invocation payload.
        invocation: MacroInvocation,
    },

    /// Match expression: match scrutinee { arms... }
    Match {
        /// Expression to match on
        scrutinee: Box<Expr>,
        /// Match arms
        arms: Vec<MatchArm>,
        /// Source span
        span: Span,
    },
    /// Policy expression
    Policy(PolicyExpr),
    /// If-let expression: if let pattern = expr then expr else expr
    IfLet {
        /// Pattern to match against
        pattern: Pattern,
        /// Expression to match
        expr: Box<Expr>,
        /// Branch taken when pattern matches
        then_branch: Box<Expr>,
        /// Branch taken when pattern doesn't match
        else_branch: Box<Expr>,
        /// Source span
        span: Span,
    },
    /// Check obligation expression: check obligation_name
    CheckObligation {
        /// Name of the obligation to check
        obligation: Name,
        /// Source span
        span: Span,
    },

    /// Constructor expression: Some { value: 42 } or RuntimeError(2, "boom")
    Constructor {
        /// Constructor name
        name: Name,
        /// Record field expressions preserved for record constructors
        fields: Vec<(Name, Expr)>,
        /// Explicit constructor payload shape
        payload: ConstructorPayload,
        /// Source span
        span: Span,
    },
    /// Structural record expression: `{ name: "Ada", age: 41 }`
    Record {
        /// Field expressions.
        fields: Vec<(Name, Expr)>,
        /// Source span
        span: Span,
    },
    /// Value-producing if expression: if expr then { body } else { body }
    If {
        /// Condition expression
        condition: Box<Expr>,
        /// Then branch expression
        then_branch: Box<Expr>,
        /// Optional else branch expression (defaults to null if absent)
        else_branch: Option<Box<Expr>>,
        /// Source span
        span: Span,
    },
    /// Panic expression: panic "message"
    Panic {
        /// Panic message
        message: Box<str>,
        /// Source span
        span: Span,
    },
    /// Operational bottom expression: `fail payload`.
    Fail {
        /// Failure payload value
        payload: Box<Expr>,
        /// Source span
        span: Span,
    },
    /// Scoped operational failure handler: `with_error { body } handle { arms... }`.
    WithError {
        /// Protected body expression
        body: Box<Expr>,
        /// Failure handler arms
        arms: Vec<MatchArm>,
        /// Source span
        span: Span,
    },
    /// Block expression: { stmt1; stmt2; tail_expr }
    Block {
        /// Statements (let-bindings)
        statements: Vec<BlockStmt>,
        /// Tail expression (return value)
        tail_expr: Option<Box<Expr>>,
        /// Source span
        span: Span,
    },

    /// Anonymous function definition (closure). SPEC-031 §5.1
    FnDef {
        /// Parameters as (name, optional type annotation) pairs
        params: Vec<(Name, Option<Name>)>,
        /// Optional return type annotation
        return_type: Option<Name>,
        /// Function body
        body: Box<Expr>,
        /// Source span
        span: Span,
    },

    /// Function application. SPEC-031 §5.4
    FnApply {
        /// Expression evaluating to a function value
        func: Box<Expr>,
        /// Arguments to apply
        args: Vec<Expr>,
        /// Source span
        span: Span,
    },

    /// Act block expression: `act { stmt; stmt; ... }`. SPEC-047 §4.1
    ///
    /// Surface-only: lowers to nested `bind`/`unit` calls via the lowerer.
    /// Does not appear in core IR.
    ActBlock {
        /// Statements inside the act block
        stmts: Vec<ActStmt>,
        /// Source span
        span: Span,
    },
    /// Generalized typed do-block: `do:K { stmt; ...; return expr }`.
    ///
    /// Surface-only substrate for SPEC-054. This must not lower until typed
    /// elaboration is implemented in later tasks.
    DoBlock {
        /// Target kind after `do:`.
        target: DoTarget,
        /// Statements inside the do block.
        stmts: Vec<DoStmt>,
        /// Source span covering the whole block.
        span: Span,
    },
    /// Bracket comprehension expression: `[result | qualifiers]: K`.
    ///
    /// Surface-only substrate for SPEC-055. This must not lower until typed
    /// elaboration normalizes it through generalized typed do-notation.
    Comprehension {
        /// Result expression before `|`.
        result: Box<Expr>,
        /// Comma-separated qualifier list after `|`; parser requires non-empty.
        qualifiers: Vec<ComprehensionQualifier>,
        /// Optional comprehension-specific target annotation after `]`.
        target: Option<DoTarget>,
        /// Source span covering the whole comprehension and target annotation.
        span: Span,
    },
    /// List expression, primarily used to preserve raw contract syntax such as `any_role([a, b])`.
    List { items: Vec<Expr>, span: Span },
}

/// Preserved constructor payload shape at the parser surface.
#[derive(Debug, Clone, PartialEq)]
pub enum ConstructorPayload {
    /// Unit constructor with no payload
    Unit,
    /// Record constructor with named fields
    Record(Vec<(Name, Expr)>),
    /// Tuple constructor with positional items
    Tuple(Vec<Expr>),
}

/// A single arm in a match expression.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    /// Pattern to match against
    pub pattern: Pattern,
    /// Expression to evaluate if pattern matches
    pub body: Box<Expr>,
    /// Source span
    pub span: Span,
}

/// A statement inside a block expression.
#[derive(Debug, Clone, PartialEq)]
pub enum BlockStmt {
    /// Let binding: `let pattern = expr;`
    Let {
        /// Pattern to bind
        pattern: Pattern,
        /// Expression to evaluate
        expr: Expr,
        /// Source span
        span: Span,
    },
}

/// Policy expression for combinators.
///
/// Policy expressions allow building complex policies from simple primitives
/// using logical, arithmetic, and higher-order combinators (SPEC-007).
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyExpr {
    /// Variable reference to a policy
    Var { name: Name, span: Span },
    /// Conjunction: all policies must hold
    And(Vec<PolicyExpr>),
    /// Disjunction: at least one policy must hold
    Or(Vec<PolicyExpr>),
    /// Negation: policy must not hold
    Not(Box<PolicyExpr>),
    /// Implication: if antecedent then consequent
    Implies(Box<PolicyExpr>, Box<PolicyExpr>),
    /// Sequential composition: policies apply in order
    Sequential(Vec<PolicyExpr>),
    /// Concurrent composition: policies apply simultaneously
    Concurrent(Vec<PolicyExpr>),
    /// Universal quantifier: all items satisfy the policy
    ForAll {
        /// Variable name for each item
        var: Name,
        /// Collection expression
        items: Box<Expr>,
        /// Policy body
        body: Box<PolicyExpr>,
        /// Source span
        span: Span,
    },
    /// Existential quantifier: at least one item satisfies the policy
    Exists {
        /// Variable name for each item
        var: Name,
        /// Collection expression
        items: Box<Expr>,
        /// Policy body
        body: Box<PolicyExpr>,
        /// Source span
        span: Span,
    },
    /// Method call on a policy: receiver.method(args)
    MethodCall {
        /// Receiver policy expression
        receiver: Box<PolicyExpr>,
        /// Method name
        method: Name,
        /// Method arguments
        args: Vec<Expr>,
        /// Source span
        span: Span,
    },
    /// Function call returning a policy
    Call {
        /// Function name
        func: Name,
        /// Function arguments
        args: Vec<Expr>,
        /// Source span
        span: Span,
    },
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    /// Logical negation: !
    Not,
    /// Arithmetic negation: -
    Neg,
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    /// Addition: +
    Add,
    /// Subtraction: -
    Sub,
    /// Multiplication: *
    Mul,
    /// Division: /
    Div,
    /// Modulo: %
    Mod,
    /// Logical AND: &&
    And,
    /// Logical OR: ||
    Or,
    /// Equality: ==
    Eq,
    /// Inequality: !=
    Neq,
    /// Less than: <
    Lt,
    /// Greater than: >
    Gt,
    /// Less than or equal: <=
    Leq,
    /// Greater than or equal: >=
    Geq,
    /// Membership test: in
    In,
    /// Pipe operator: |>
    Pipe,
}

/// Pattern types for destructuring.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Variable binding
    Variable { name: Name, span: Span },
    /// Wildcard pattern: _
    Wildcard,
    /// Tuple pattern: (a, b, c)
    Tuple(Vec<Pattern>),
    /// Record pattern: { field: pat, ... }
    Record(Vec<(Name, Pattern)>),
    /// List pattern: [a, b, ..rest]
    List {
        /// Element patterns
        elements: Vec<Pattern>,
        /// Optional rest binding
        rest: Option<Name>,
    },
    /// Literal pattern
    Literal(Literal),
    /// Variant pattern: Some { value: x }, RuntimeError(code, msg), or None
    Variant {
        /// Variant name (e.g., "Some", "None")
        name: Name,
        /// Optional record fields with patterns preserved for record variants
        fields: Option<Vec<(Name, Pattern)>>,
        /// Explicit payload shape for unit/record/tuple variant patterns
        payload: VariantPatternPayload,
    },
}

/// Preserved payload shape for parsed variant patterns.
#[derive(Debug, Clone, PartialEq)]
pub enum VariantPatternPayload {
    /// Unit variant pattern without payload
    Unit,
    /// Record variant pattern with named fields
    Record(Vec<(Name, Pattern)>),
    /// Tuple variant pattern with positional items
    Tuple(Vec<Pattern>),
}

use ordered_float::OrderedFloat;

/// Literal values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Literal {
    /// Integer literal
    Int(i64),
    /// Floating-point literal (total ordering for Eq/Hash)
    Float(OrderedFloat<f64>),
    /// String literal
    String(Box<str>),
    /// Boolean literal
    Bool(bool),
    /// Null literal
    Null,
    /// List literal: [1, 2, 3]
    List(Vec<Literal>),
}

/// Effect type levels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectType {
    /// Read-only observation
    Observe,
    /// Reading data
    Read,
    /// Analyzing data
    Analyze,
    /// Making decisions
    Decide,
    /// Taking actions
    Act,
    /// Writing/modifying data
    Write,
    /// External effects
    External,
    /// Knowledge/observation effect (lattice variant)
    Epistemic,
    /// Deliberation/analysis effect (lattice variant)
    Deliberative,
    /// Decision/evaluation effect (lattice variant)
    Evaluative,
    /// Action/operation effect (lattice variant)
    Operational,
}

/// Policy decisions.
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    /// Permit the action
    Permit,
    /// Deny the action
    Deny,
    /// Require approval from a role
    RequireApproval {
        /// Role required for approval
        role: Name,
    },
    /// Escalate to supervisor
    Escalate,
}

/// Target of an operational call - symbolic, qualified, or explicit provider:action.
#[derive(Debug, Clone, PartialEq)]
pub enum OperationalTarget {
    /// Symbolic capability call: `capability(args)` - resolved via resolver metadata
    Symbolic {
        /// The capability name to resolve (e.g., "fs_read")
        capability_name: Name,
    },
    /// Module-qualified symbolic call: `module::capability(args)` - resolved via resolver
    Qualified {
        /// Module path (e.g., "io" in "io::fs_read")
        module: Name,
        /// Capability name within the module (e.g., "fs_read" in "io::fs_read")
        capability_name: Name,
    },
    /// Explicit provider:action call: `provider:action(args)`
    Explicit {
        /// Provider name
        provider: Name,
        /// Action name
        action: Name,
    },
}

/// Reference to an action invocation.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionRef {
    /// Target of the action (symbolic or explicit)
    pub target: OperationalTarget,
    /// Arguments to the action
    pub args: Vec<Expr>,
}

/// Reference to an obligation.
#[derive(Debug, Clone, PartialEq)]
pub struct ObligationRef {
    /// Role with the obligation
    pub role: Name,
    /// Condition that must hold
    pub condition: Expr,
}

/// Target of a check statement - either an obligation or a policy instance.
#[derive(Debug, Clone, PartialEq)]
pub enum CheckTarget {
    /// Legacy obligation reference
    Obligation(ObligationRef),
    /// Policy instance check
    Policy(PolicyInstance),
}

/// Parameter with name and type.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    /// Parameter name
    pub name: Name,
    /// Parameter type
    pub ty: Type,
}

/// Type annotations.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Named type
    Name(Name),
    /// Explicit source type hole `_` in an audited type-expression position.
    Hole {
        /// Source span covering the `_` token.
        span: Span,
    },
    /// List type: \[T\]
    List(Box<Type>),
    /// Tuple type: `(T, U, ...)`
    Tuple(Vec<Type>),
    /// Record type: { field: T, ... }
    Record(Vec<(Name, Type)>),
    /// Capability type
    Capability(Name),
    /// Generic type constructor: `List<Int>`, `Option<String>`
    Constructor { name: Name, args: Vec<Type> },
    /// Associated type projection: `S::Ok`, `Map<K,V>::Entry`
    Associated { base: Box<Type>, name: Name },
    /// Explicit associated-family projection: `<Interface<Args...>>::Assoc`
    AssociatedFamilyProjection {
        /// Source-visible unqualified interface name.
        interface: Name,
        /// Raw interface argument spine.
        args: Vec<Type>,
        /// Associated member name.
        member: Name,
        /// Source span.
        span: Span,
    },
    /// Function type: Fn(params..., [row], result)
    Fn(Vec<Type>, Option<ComputationRow>, Box<Type>),
}

/// Guard expressions for actions.
#[derive(Debug, Clone, PartialEq)]
pub enum Guard {
    /// Always allow
    Always,
    /// Never allow
    Never,
    /// Predicate guard
    Pred(Predicate),
    /// Conjunction: left AND right
    And(Box<Guard>, Box<Guard>),
    /// Disjunction: left OR right
    Or(Box<Guard>, Box<Guard>),
    /// Negation: NOT guard
    Not(Box<Guard>),
}

/// A predicate expression.
#[derive(Debug, Clone, PartialEq)]
pub struct Predicate {
    /// Predicate name
    pub name: Name,
    /// Predicate arguments
    pub args: Vec<Expr>,
}

/// A constraint on a capability.
#[derive(Debug, Clone, PartialEq)]
pub struct Constraint {
    /// The constraint predicate
    pub predicate: Predicate,
}

/// Trait for types that have a source span.
pub trait Spanned {
    /// Returns the source span of this node.
    fn span(&self) -> Span;
}

impl Spanned for Workflow {
    fn span(&self) -> Span {
        match self {
            Workflow::Observe { span, .. } => *span,
            Workflow::Orient { span, .. } => *span,
            Workflow::Propose { span, .. } => *span,
            Workflow::Decide { span, .. } => *span,
            Workflow::Check { span, .. } => *span,
            Workflow::Oblige { span, .. } => *span,
            Workflow::Act { span, .. } => *span,
            Workflow::Let { span, .. } => *span,
            Workflow::If { span, .. } => *span,
            Workflow::For { span, .. } => *span,
            Workflow::With { span, .. } => *span,
            Workflow::Maybe { span, .. } => *span,
            Workflow::Must { span, .. } => *span,
            Workflow::Seq { span, .. } => *span,
            Workflow::Done { span, .. } => *span,
            Workflow::Ret { span, .. } => *span,
            Workflow::Set { span, .. } => *span,
            Workflow::Send { span, .. } => *span,
            Workflow::Receive { span, .. } => *span,
            Workflow::Yield { span, .. } => *span,
            Workflow::Resume { span, .. } => *span,
        }
    }
}

impl Spanned for Expr {
    fn span(&self) -> Span {
        match self {
            Expr::OperatorSection { section } => section.span,
            Expr::Literal(_) => Span::default(),
            Expr::Variable { span, .. } => *span,
            Expr::FieldAccess { span, .. } => *span,
            Expr::IndexAccess { span, .. } => *span,
            Expr::Unary { span, .. } => *span,
            Expr::Binary { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::MacroInvocation { invocation } => invocation.span,
            Expr::Match { span, .. } => *span,
            Expr::Policy(policy_expr) => policy_expr.span(),
            Expr::IfLet { span, .. } => *span,
            Expr::CheckObligation { span, .. } => *span,
            Expr::Constructor { span, .. } => *span,
            Expr::Record { span, .. } => *span,
            Expr::If { span, .. } => *span,
            Expr::Panic { span, .. } => *span,
            Expr::Fail { span, .. } => *span,
            Expr::WithError { span, .. } => *span,
            Expr::Block { span, .. } => *span,
            Expr::FnDef { span, .. } => *span,
            Expr::FnApply { span, .. } => *span,
            Expr::ActBlock { span, .. } => *span,
            Expr::DoBlock { span, .. } => *span,
            Expr::Comprehension { span, .. } => *span,
            Expr::List { span, .. } => *span,
        }
    }
}

impl Spanned for PolicyExpr {
    fn span(&self) -> Span {
        match self {
            PolicyExpr::Var { span, .. } => *span,
            PolicyExpr::And(exprs) => {
                // Return span of first expression, or default if empty
                exprs.first().map(Spanned::span).unwrap_or_default()
            }
            PolicyExpr::Or(exprs) => exprs.first().map(Spanned::span).unwrap_or_default(),
            PolicyExpr::Not(expr) => expr.span(),
            PolicyExpr::Implies(left, _) => left.span(),
            PolicyExpr::Sequential(exprs) => exprs.first().map(Spanned::span).unwrap_or_default(),
            PolicyExpr::Concurrent(exprs) => exprs.first().map(Spanned::span).unwrap_or_default(),
            PolicyExpr::ForAll { span, .. } => *span,
            PolicyExpr::Exists { span, .. } => *span,
            PolicyExpr::MethodCall { span, .. } => *span,
            PolicyExpr::Call { span, .. } => *span,
        }
    }
}

impl Spanned for PolicyInstance {
    fn span(&self) -> Span {
        self.span
    }
}

impl Workflow {
    /// Compute the total effect of this workflow.
    ///
    /// Effects form a lattice: Epistemic < Deliberative < Evaluative < Operational
    /// This method computes the join (⊔) of all effects in the workflow.
    pub fn effect(&self) -> ash_core::Effect {
        use ash_core::Effect;

        match self {
            // Read-only observation - pure reads
            Workflow::Observe { continuation, .. } => {
                if let Some(cont) = continuation {
                    Effect::Epistemic.join(cont.effect())
                } else {
                    Effect::Epistemic
                }
            }

            // Pure expression evaluation
            Workflow::Orient { continuation, .. } => {
                if let Some(cont) = continuation {
                    Effect::Epistemic.join(cont.effect())
                } else {
                    Effect::Epistemic
                }
            }

            // Proposing actions requires deliberation
            Workflow::Propose { continuation, .. } => {
                if let Some(cont) = continuation {
                    Effect::Deliberative.join(cont.effect())
                } else {
                    Effect::Deliberative
                }
            }

            // Decision branches - join of both branches
            Workflow::Decide {
                then_branch,
                else_branch,
                ..
            } => {
                let then_effect = then_branch.effect();
                match else_branch {
                    Some(else_b) => then_effect.join(else_b.effect()),
                    None => then_effect,
                }
            }

            // Checking obligations/policies is evaluative
            Workflow::Check { continuation, .. } => {
                if let Some(cont) = continuation {
                    Effect::Evaluative.join(cont.effect())
                } else {
                    Effect::Evaluative
                }
            }

            // Creating obligations is evaluative (affects type checking)
            Workflow::Oblige { .. } => Effect::Evaluative,

            // Executing actions has side effects
            Workflow::Act { .. } => Effect::Operational,

            // Let binding - effect of the continuation
            Workflow::Let { continuation, .. } => {
                if let Some(cont) = continuation {
                    cont.effect()
                } else {
                    Effect::Epistemic
                }
            }

            // Conditional - join of branches
            Workflow::If {
                then_branch,
                else_branch,
                ..
            } => {
                let then_effect = then_branch.effect();
                match else_branch {
                    Some(else_b) => then_effect.join(else_b.effect()),
                    None => then_effect,
                }
            }

            // For loop - effect of body
            Workflow::For { body, .. } => body.effect(),

            // With clause - effect of body
            Workflow::With { body, .. } => body.effect(),

            // Maybe - join of primary and fallback
            Workflow::Maybe {
                primary, fallback, ..
            } => primary.effect().join(fallback.effect()),

            // Must - effect of body
            Workflow::Must { body, .. } => body.effect(),

            // Sequential composition - join of both
            Workflow::Seq { first, second, .. } => first.effect().join(second.effect()),

            // Done - no effect
            Workflow::Done { .. } => Effect::Epistemic,

            // Return - no effect
            Workflow::Ret { .. } => Effect::Epistemic,

            // Set - operational effect with continuation
            Workflow::Set { continuation, .. } => {
                if let Some(cont) = continuation {
                    Effect::Operational.join(cont.effect())
                } else {
                    Effect::Operational
                }
            }

            // Send - operational effect with continuation
            Workflow::Send { continuation, .. } => {
                if let Some(cont) = continuation {
                    Effect::Operational.join(cont.effect())
                } else {
                    Effect::Operational
                }
            }

            // Receive - epistemic (read-only) with join of all arm body effects
            Workflow::Receive { arms, .. } => {
                // Receive is Epistemic (reading from mailbox)
                // Join with effects of all arm bodies
                arms.iter()
                    .map(|arm| arm.body.effect())
                    .fold(Effect::Epistemic, |a, b| a.join(b))
            }

            // Yield - deliberative (delegating to a role) with join of all arm body effects
            Workflow::Yield { arms, .. } => {
                // Yield is Deliberative (interacting with roles)
                // Join with effects of all arm bodies
                arms.iter()
                    .map(|arm| arm.body.effect())
                    .fold(Effect::Deliberative, |a, b| a.join(b))
            }

            // Resume - epistemic (returning a value)
            Workflow::Resume { .. } => Effect::Epistemic,
        }
    }
}

#[cfg(test)]
mod effect_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod visibility_tests;
