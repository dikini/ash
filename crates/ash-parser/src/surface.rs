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
#[derive(Debug, Clone, PartialEq)]
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
    /// Source span covering the `where` keyword.
    pub where_span: Span,
    /// Source span covering the complete tail.
    pub span: Span,
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
    /// `by test "test_name"`
    ByTest { test_name: String },
    /// Explicit proof term (future)
    Expr(Expr),
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

/// Expression types.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
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

/// Literal values.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Integer literal
    Int(i64),
    /// Floating-point literal
    Float(f64),
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
    /// Function type: Fn(T, U) -> V
    Fn(Vec<Type>, Box<Type>),
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
            Expr::Literal(_) => Span::default(),
            Expr::Variable { span, .. } => *span,
            Expr::FieldAccess { span, .. } => *span,
            Expr::IndexAccess { span, .. } => *span,
            Expr::Unary { span, .. } => *span,
            Expr::Binary { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::Match { span, .. } => *span,
            Expr::Policy(policy_expr) => policy_expr.span(),
            Expr::IfLet { span, .. } => *span,
            Expr::CheckObligation { span, .. } => *span,
            Expr::Constructor { span, .. } => *span,
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
