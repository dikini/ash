# TASK-017: Desugaring Transformations

## Status: 🟢 Complete

## Description

Implement additional desugaring transformations that simplify complex surface constructs into simpler core forms.

## Specification Reference

- SPEC-002: Surface Language - Section 4. Semantic Sugar
- SPEC-001: IR

## Requirements

### Desugaring Transformations

1. **Optional Binding Desugaring**
   - `observe cap` → `observe cap as _`
   - `orient { expr }` → `orient { expr } as _`

2. **Implicit Done**
   - Workflow without explicit `done` → Add `done` at end

3. **Sequential Composition**
   - `stmt1; stmt2; stmt3` → `Seq(stmt1, Seq(stmt2, stmt3))`

4. **Maybe Desugaring**
   - `maybe primary else fallback` → Error handling construct

5. **Must Desugaring**
   - `must workflow` → With obligation tracking

6. **Attempt/Catch Desugaring**
   - `attempt try catch catch` → Error handling

7. **Retry Desugaring**
   - `retry wf up to n times` → Loop with counter

8. **Timeout Desugaring**
   - `timeout wf after duration` → With timeout wrapper

### Desugaring Pass

```rust
/// Apply all desugaring transformations
pub fn desugar(workflow: surface::Workflow) -> surface::Workflow {
    let workflow = desugar_optional_bindings(workflow);
    let workflow = desugar_implicit_done(workflow);
    let workflow = desugar_maybe(workflow);
    let workflow = desugar_must(workflow);
    let workflow = desugar_attempt(workflow);
    let workflow = desugar_retry(workflow);
    let workflow = desugar_timeout(workflow);
    workflow
}

/// Desugar optional binding patterns
fn desugar_optional_bindings(workflow: surface::Workflow) -> surface::Workflow {
    match workflow {
        surface::Workflow::Observe { capability, binding: None, continuation, span } => {
            surface::Workflow::Observe {
                capability,
                binding: Some(surface::Pattern::Wildcard),
                continuation,
                span,
            }
        }
        
        surface::Workflow::Orient { expr, binding: None, continuation, span } => {
            surface::Workflow::Orient {
                expr,
                binding: Some(surface::Pattern::Wildcard),
                continuation,
                span,
            }
        }
        
        surface::Workflow::Propose { action, binding: None, continuation, span } => {
            surface::Workflow::Propose {
                action,
                binding: Some(surface::Pattern::Wildcard),
                continuation,
                span,
            }
        }
        
        // Recursively desugar children
        surface::Workflow::Seq { first, second, span } => {
            surface::Workflow::Seq {
                first: Box::new(desugar_optional_bindings(*first)),
                second: Box::new(desugar_optional_bindings(*second)),
                span,
            }
        }
        
        surface::Workflow::If { condition, then_branch, else_branch, span } => {
            surface::Workflow::If {
                condition,
                then_branch: Box::new(desugar_optional_bindings(*then_branch)),
                else_branch: else_branch.map(|e| Box::new(desugar_optional_bindings(*e))),
                span,
            }
        }
        
        // ... more recursive cases
        
        _ => workflow,
    }
}

/// Add implicit done to workflows
fn desugar_implicit_done(workflow: surface::Workflow) -> surface::Workflow {
    match workflow {
        // If the workflow ends with an action, add done after it
        surface::Workflow::Act { .. } => {
            // This would need span info
            surface::Workflow::Seq {
                first: Box::new(workflow),
                second: Box::new(surface::Workflow::Done { span: Span::default() }),
                span: Span::default(),
            }
        }
        
        // Recursively process children
        surface::Workflow::Observe { capability, binding, continuation, span } => {
            surface::Workflow::Observe {
                capability,
                binding,
                continuation: continuation.map(|c| Box::new(desugar_implicit_done(*c))),
                span,
            }
        }
        
        // ... more cases
        
        _ => workflow,
    }
}

/// Desugar maybe to try/catch pattern
fn desugar_maybe(workflow: surface::Workflow) -> surface::Workflow {
    match workflow {
        surface::Workflow::Maybe { primary, fallback, span } => {
            // maybe primary else fallback
            // =>
            // attempt primary catch fallback
            surface::Workflow::Attempt {
                try_body: primary,
                catch_body: fallback,
                span,
            }
        }
        
        // Recursively desugar children
        surface::Workflow::Seq { first, second, span } => {
            surface::Workflow::Seq {
                first: Box::new(desugar_maybe(*first)),
                second: Box::new(desugar_maybe(*second)),
                span,
            }
        }
        
        // ... more recursive cases
        
        _ => workflow,
    }
}

/// Desugar must to obligation check
fn desugar_must(workflow: surface::Workflow) -> surface::Workflow {
    match workflow {
        surface::Workflow::Must { body, span } => {
            // must workflow
            // =>
            // workflow; check obligation_satisfied
            // (simplified - actual implementation more complex)
            surface::Workflow::Seq {
                first: body,
                second: Box::new(surface::Workflow::Done { span }), // Placeholder
                span,
            }
        }
        
        // ... recursive cases
        
        _ => workflow,
    }
}

/// Desugar retry to loop with counter
fn desugar_retry(workflow: surface::Workflow) -> surface::Workflow {
    match workflow {
        surface::Workflow::Retry { body, max_attempts, span } => {
            // retry wf up to n times
            // =>
            // let attempt = 0 in
            //   loop {
            //     attempt attempt = attempt + 1;
            //     if attempt > n then fail else wf
            //   }
            
            // Simplified: just unwrap for now
            *body
        }
        
        // ... recursive cases
        
        _ => workflow,
    }
}

/// Desugar timeout to async with timeout
fn desugar_timeout(workflow: surface::Workflow) -> surface::Workflow {
    match workflow {
        surface::Workflow::Timeout { body, duration, span } => {
            // timeout wf after duration
            // =>
            // async with timeout(duration) { wf }
            
            // Simplified: just unwrap for now
            *body
        }
        
        // ... recursive cases
        
        _ => workflow,
    }
}
```

### Desugaring with Context

```rust
/// Desugaring context for tracking transformations
pub struct DesugarContext {
    /// Generated variable counter
    var_counter: usize,
    /// Applied transformations
    transformations: Vec<String>,
}

impl DesugarContext {
    pub fn new() -> Self {
        Self {
            var_counter: 0,
            transformations: Vec::new(),
        }
    }
    
    pub fn fresh_var(&mut self, prefix: &str) -> String {
        self.var_counter += 1;
        format!("{}_{}", prefix, self.var_counter)
    }
    
    pub fn record(&mut self, transformation: impl Into<String>) {
        self.transformations.push(transformation.into());
    }
}
```

## TDD Steps

### Step 1: Implement Optional Binding Desugaring

Start with simplest transformation.

### Step 2: Implement Implicit Done

Add done to workflow endings.

### Step 3: Implement Maybe/Must Desugaring

Add more complex transformations.

### Step 4: Implement Retry/Timeout

Add async-related desugaring.

### Step 5: Write Transformation Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desugar_optional_binding() {
        let input = surface::Workflow::Observe {
            capability: surface::CapabilityRef { name: "read".into(), args: vec![] },
            binding: None,
            continuation: None,
            span: Span::default(),
        };
        
        let result = desugar_optional_bindings(input);
        
        match result {
            surface::Workflow::Observe { binding: Some(surface::Pattern::Wildcard), .. } => {
                // Success
            }
            _ => panic!("Expected wildcard binding"),
        }
    }

    #[test]
    fn test_desugar_maybe_to_attempt() {
        let input = surface::Workflow::Maybe {
            primary: Box::new(surface::Workflow::Done { span: Span::default() }),
            fallback: Box::new(surface::Workflow::Done { span: Span::default() }),
            span: Span::default(),
        };
        
        let result = desugar_maybe(input);
        
        assert!(matches!(result, surface::Workflow::Attempt { .. }));
    }
}
```

## Completion Checklist

- [ ] Optional binding desugaring
- [ ] Implicit done desugaring
- [ ] Maybe desugaring
- [ ] Must desugaring
- [ ] Attempt/Catch desugaring
- [ ] Retry desugaring
- [ ] Timeout desugaring
- [ ] Recursive desugaring for nested workflows
- [ ] Transformation tests
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Completeness**: Are all syntactic sugars desugared?
2. **Correctness**: Do desugared forms preserve semantics?
3. **Composability**: Do desugaring passes compose correctly?

## Estimated Effort

4 hours

## Dependencies

- TASK-011: Surface AST (transforms these types)

## Blocked By

- TASK-011: Surface AST

## Blocks

- TASK-016: Lowering (desugaring happens before lowering)
