# TASK-022: Name Resolution Pass

## Status: ✅ Complete

## Description

Implement the name resolution pass that binds identifiers to their definitions and checks for undefined names.

## Specification Reference

- SPEC-003: Type System - Section 4. Validation
- SPEC-001: IR

## Requirements

### Scope Types

```rust
/// Types of scopes in the name resolution hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// Global scope (top-level definitions)
    Global,
    /// Workflow scope (workflow-local bindings)
    Workflow,
    /// Block scope (let bindings, pattern bindings)
    Block,
    /// Capability scope (with-do)
    Capability,
}

/// A scope in the name resolution hierarchy
#[derive(Debug, Clone)]
pub struct Scope {
    pub kind: ScopeKind,
    pub bindings: HashMap<Box<str>, Binding>,
    pub parent: Option<usize>, // Index into scopes vector
}

/// Information about a binding
#[derive(Debug, Clone)]
pub struct Binding {
    pub name: Box<str>,
    pub kind: BindingKind,
    pub span: Span,
    pub ty: Option<Type>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Variable,
    Capability,
    Policy,
    Role,
    Workflow,
    Function,
}
```

### Name Resolver

```rust
/// Name resolver state
#[derive(Debug, Default)]
pub struct NameResolver {
    pub scopes: Vec<Scope>,
    pub current_scope: usize,
    pub errors: Vec<NameError>,
    pub warnings: Vec<NameWarning>,
}

impl NameResolver {
    pub fn new() -> Self {
        let global_scope = Scope {
            kind: ScopeKind::Global,
            bindings: HashMap::new(),
            parent: None,
        };
        
        Self {
            scopes: vec![global_scope],
            current_scope: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    /// Enter a new scope
    pub fn enter_scope(&mut self, kind: ScopeKind) {
        let new_scope = Scope {
            kind,
            bindings: HashMap::new(),
            parent: Some(self.current_scope),
        };
        
        self.scopes.push(new_scope);
        self.current_scope = self.scopes.len() - 1;
    }
    
    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current_scope].parent {
            self.current_scope = parent;
        }
    }
    
    /// Declare a binding in the current scope
    pub fn declare(&mut self, name: impl Into<Box<str>>, kind: BindingKind, span: Span) {
        let name = name.into();
        let scope = &mut self.scopes[self.current_scope];
        
        // Check for duplicate declarations
        if let Some(existing) = scope.bindings.get(&name) {
            self.errors.push(NameError::DuplicateDeclaration {
                name: name.to_string(),
                first_span: existing.span,
                second_span: span,
            });
            return;
        }
        
        scope.bindings.insert(name.clone(), Binding {
            name,
            kind,
            span,
            ty: None,
        });
    }
    
    /// Look up a name in the current scope chain
    pub fn lookup(&self, name: &str) -> Option<&Binding> {
        let mut scope_idx = Some(self.current_scope);
        
        while let Some(idx) = scope_idx {
            let scope = &self.scopes[idx];
            if let Some(binding) = scope.bindings.get(name) {
                return Some(binding);
            }
            scope_idx = scope.parent;
        }
        
        None
    }
    
    /// Resolve a name, reporting error if not found
    pub fn resolve(&mut self, name: &str, span: Span, expected: &[BindingKind]) -> Option<&Binding> {
        match self.lookup(name) {
            Some(binding) => {
                // Check kind matches expectation
                if !expected.is_empty() && !expected.contains(&binding.kind) {
                    self.errors.push(NameError::WrongKind {
                        name: name.to_string(),
                        found: binding.kind,
                        expected: expected.to_vec(),
                        span,
                    });
                }
                Some(binding)
            }
            None => {
                self.errors.push(NameError::UndefinedName {
                    name: name.to_string(),
                    span,
                });
                None
            }
        }
    }
}
```

### Resolution Errors

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum NameError {
    #[error("Undefined name: {name}")]
    UndefinedName { name: String, span: Span },
    
    #[error("Duplicate declaration of: {name}")]
    DuplicateDeclaration { name: String, first_span: Span, second_span: Span },
    
    #[error("Expected {expected:?}, found {found:?}: {name}")]
    WrongKind { name: String, found: BindingKind, expected: Vec<BindingKind>, span: Span },
    
    #[error("Cannot shadow {name} in this scope")]
    IllegalShadowing { name: String, span: Span },
}

#[derive(Debug, Clone)]
pub enum NameWarning {
    #[allow(dead_code)]
    UnusedBinding { name: String, span: Span },
}
```

### Program Resolution

```rust
/// Resolve names in a complete program
pub fn resolve_program(program: &mut Program) -> Result<(), Vec<NameError>> {
    let mut resolver = NameResolver::new();
    
    // First pass: collect top-level definitions
    for def in &program.definitions {
        collect_definition(&mut resolver, def);
    }
    
    // Second pass: resolve within definitions
    for def in &mut program.definitions {
        resolve_definition(&mut resolver, def)?;
    }
    
    // Resolve main workflow
    resolver.enter_scope(ScopeKind::Workflow);
    resolve_workflow(&mut resolver, &mut program.workflow)?;
    resolver.exit_scope();
    
    if resolver.errors.is_empty() {
        Ok(())
    } else {
        Err(resolver.errors)
    }
}

fn collect_definition(resolver: &mut NameResolver, def: &Definition) {
    match def {
        Definition::Capability(cap) => {
            resolver.declare(&cap.name, BindingKind::Capability, cap.span);
        }
        Definition::Policy(policy) => {
            resolver.declare(&policy.name, BindingKind::Policy, policy.span);
        }
        Definition::Role(role) => {
            resolver.declare(&role.name, BindingKind::Role, role.span);
        }
        Definition::Workflow(workflow) => {
            resolver.declare(&workflow.name, BindingKind::Workflow, workflow.span);
        }
        _ => {}
    }
}

fn resolve_workflow(
    resolver: &mut NameResolver,
    workflow: &mut Workflow,
) -> Result<(), Vec<NameError>> {
    match workflow {
        Workflow::Observe { capability, pattern, continuation } => {
            // Resolve capability reference
            resolver.resolve(&capability.name, Span::default(), &[BindingKind::Capability]);
            
            // Bind pattern variables
            bind_pattern(resolver, pattern)?;
            
            // Resolve continuation
            resolve_workflow(resolver, continuation)?;
        }
        
        Workflow::Let { pattern, expr, continuation } => {
            // First resolve the expression (in current scope)
            resolve_expr(resolver, expr)?;
            
            // Then bind the pattern
            bind_pattern(resolver, pattern)?;
            
            // Resolve continuation (with new bindings)
            if let Some(cont) = continuation {
                resolve_workflow(resolver, cont)?;
            }
        }
        
        Workflow::Act { action, .. } => {
            // Resolve action reference
            resolver.resolve(&action.name, Span::default(), &[BindingKind::Capability]);
        }
        
        Workflow::Seq { first, second } => {
            resolve_workflow(resolver, first)?;
            resolve_workflow(resolver, second)?;
        }
        
        Workflow::If { condition, then_branch, else_branch } => {
            resolve_expr(resolver, condition)?;
            resolve_workflow(resolver, then_branch)?;
            if let Some(else_) = else_branch {
                resolve_workflow(resolver, else_)?;
            }
        }
        
        Workflow::Par { workflows } => {
            // Each branch gets its own scope
            for wf in workflows {
                resolver.enter_scope(ScopeKind::Block);
                resolve_workflow(resolver, wf)?;
                resolver.exit_scope();
            }
        }
        
        Workflow::With { capability, body } => {
            resolver.enter_scope(ScopeKind::Capability);
            resolver.resolve(&capability.name, Span::default(), &[BindingKind::Capability]);
            resolve_workflow(resolver, body)?;
            resolver.exit_scope();
        }
        
        _ => {}
    }
    
    Ok(())
}

fn resolve_expr(resolver: &mut NameResolver, expr: &mut Expr) -> Result<(), Vec<NameError>> {
    match expr {
        Expr::Var(name) => {
            resolver.resolve(name, Span::default(), &[BindingKind::Variable]);
        }
        Expr::Field { base, .. } => {
            resolve_expr(resolver, base)?;
        }
        Expr::BinOp { left, right, .. } => {
            resolve_expr(resolver, left)?;
            resolve_expr(resolver, right)?;
        }
        Expr::UnOp { operand, .. } => {
            resolve_expr(resolver, operand)?;
        }
        Expr::Call { func, args } => {
            resolver.resolve(func, Span::default(), &[BindingKind::Function, BindingKind::Capability]);
            for arg in args {
                resolve_expr(resolver, arg)?;
            }
        }
        _ => {}
    }
    
    Ok(())
}

fn bind_pattern(resolver: &mut NameResolver, pat: &Pattern) -> Result<(), Vec<NameError>> {
    match pat {
        Pattern::Variable(name) => {
            resolver.declare(name.clone(), BindingKind::Variable, Span::default());
        }
        Pattern::Tuple(pats) | Pattern::List(pats, _) => {
            for p in pats {
                bind_pattern(resolver, p)?;
            }
        }
        Pattern::Record(fields) => {
            for (_, p) in fields {
                bind_pattern(resolver, p)?;
            }
        }
        _ => {}
    }
    
    Ok(())
}
```

## TDD Steps

### Step 1: Implement Scope Management

Create `crates/ash-typeck/src/names.rs` with Scope and NameResolver.

### Step 2: Implement Declaration and Lookup

Add declare and lookup methods.

### Step 3: Implement Program Resolution

Add resolve_program entry point.

### Step 4: Implement Workflow Resolution

Add resolve_workflow for all workflow types.

### Step 5: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_defined_name() {
        let mut resolver = NameResolver::new();
        resolver.declare("x", BindingKind::Variable, Span::default());
        
        let binding = resolver.lookup("x");
        assert!(binding.is_some());
        assert_eq!(binding.unwrap().kind, BindingKind::Variable);
    }

    #[test]
    fn test_resolve_undefined_name() {
        let mut resolver = NameResolver::new();
        
        let result = resolver.resolve("undefined", Span::default(), &[]);
        assert!(result.is_none());
        assert!(!resolver.errors.is_empty());
    }

    #[test]
    fn test_scope_nesting() {
        let mut resolver = NameResolver::new();
        resolver.declare("outer", BindingKind::Variable, Span::default());
        
        resolver.enter_scope(ScopeKind::Block);
        // Can still see outer
        assert!(resolver.lookup("outer").is_some());
        
        resolver.declare("inner", BindingKind::Variable, Span::default());
        // Can see inner
        assert!(resolver.lookup("inner").is_some());
        
        resolver.exit_scope();
        // Can no longer see inner
        assert!(resolver.lookup("inner").is_none());
        // But can still see outer
        assert!(resolver.lookup("outer").is_some());
    }

    #[test]
    fn test_duplicate_declaration() {
        let mut resolver = NameResolver::new();
        resolver.declare("x", BindingKind::Variable, Span::default());
        resolver.declare("x", BindingKind::Variable, Span::default());
        
        assert!(!resolver.errors.is_empty());
    }
}
```

## Completion Checklist

- [ ] Scope types and hierarchy
- [ ] NameResolver with enter/exit scope
- [ ] Declaration tracking
- [ ] Name lookup with scope chain
- [ ] Error types for undefined names, duplicates, wrong kinds
- [ ] Program resolution
- [ ] Workflow resolution for all constructs
- [ ] Expression resolution
- [ ] Pattern binding
- [ ] Unit tests for all resolution cases
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Completeness**: Are all name references resolved?
2. **Scope rules**: Do scope boundaries match language semantics?
3. **Error quality**: Are undefined name errors helpful?

## Estimated Effort

6 hours

## Dependencies

- ash-core: Core types (Workflow, Expr, Pattern)

## Blocked By

- TASK-003: Workflow AST

## Blocks

- TASK-019: Type constraints (needs resolved names)
- TASK-025: Type errors (name errors)
