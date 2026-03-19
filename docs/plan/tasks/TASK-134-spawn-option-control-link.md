# TASK-134: Spawn Returns Option<ControlLink>

## Status: 🟡 Ready to Start

## Description

Update spawn expression to return a composite type that can be split into InstanceAddr and Option<ControlLink>.

## Specification Reference

- SPEC-020: ADT Types - Section 8.1, 8.2
- docs/design/CONTROL_LINK_TRANSFER.md

## Requirements

Update spawn semantics:
```ash
spawn worker with { init: args } as w;
let (w_addr, w_ctrl) = split w;

-- w_addr: InstanceAddr<Worker> - opaque handle for communication
-- w_ctrl: Option<ControlLink<Worker>> - initially Some { value: link }
```

## TDD Steps

### Step 1: Add Instance Types

**File**: `crates/ash-core/src/ast.rs`

```rust
/// Workflow instance reference (result of spawn)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instance {
    /// Address for sending signals
    pub addr: InstanceAddr,
    /// Control link for supervision (initially Some)
    pub control: Option<ControlLink>,
}

/// Opaque instance address
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstanceAddr {
    pub workflow_type: Name,
    pub instance_id: WorkflowId,
}

/// Control link for supervision
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlLink {
    pub instance_id: WorkflowId,
}

/// Split operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Split {
    pub instance: Box<Expr>,
}

/// Add to Expr enum
pub enum Expr {
    // Existing...
    
    /// Spawn a workflow instance
    Spawn {
        workflow_type: Name,
        init: Vec<(Name, Expr)>,
    },
    
    /// Split instance into (addr, control)
    Split(Box<Expr>),
}

/// Add to Workflow enum
pub enum Workflow {
    // Existing...
    
    /// Spawn a workflow and bind it
    Spawn {
        workflow_type: Name,
        init: Vec<(Name, Expr)>,
        binding: Name,
        continuation: Box<Workflow>,
    },
    
    /// Split instance into address and control link
    Split {
        instance: Name,
        addr_binding: Name,
        control_binding: Name,
        continuation: Box<Workflow>,
    },
}
```

### Step 2: Add Instance Types to Type System

**File**: `crates/ash-typeck/src/types.rs`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // Existing...
    
    /// Instance type (composite of addr + control link)
    Instance {
        workflow_type: Box<str>,
    },
    
    /// Opaque instance address
    InstanceAddr {
        workflow_type: Box<str>,
    },
    
    /// Control link (affine - must be used exactly once)
    ControlLink {
        workflow_type: Box<str>,
    },
}
```

### Step 3: Add Built-in Instance Types

**File**: `crates/ash-typeck/src/type_env.rs`

```rust
impl TypeEnv {
    pub fn spawn_instance_type(&self, workflow_type: &str) -> Type {
        Type::Instance {
            workflow_type: workflow_type.into(),
        }
    }
    
    pub fn split_types(&self, instance_type: &Type) -> Result<(Type, Type), TypeError> {
        match instance_type {
            Type::Instance { workflow_type } => {
                let addr = Type::InstanceAddr {
                    workflow_type: workflow_type.clone(),
                };
                let control = Type::Constructor {
                    name: "Option".into(),
                    args: vec![Type::ControlLink {
                        workflow_type: workflow_type.clone(),
                    }],
                };
                Ok((addr, control))
            }
            _ => Err(TypeError::InvalidSplit(instance_type.to_string())),
        }
    }
}
```

### Step 4: Type Check Spawn and Split

**File**: `crates/ash-typeck/src/check_expr.rs`

```rust
fn check_expr(env: &TypeEnv, var_env: &mut VarEnv, expr: &Expr) -> Result<(Type, Effect), TypeError> {
    match expr {
        // Existing...
        
        Expr::Spawn { workflow_type, init } => {
            // Check init arguments
            let mut total_effect = Effect::Epistemic;
            for (_, arg) in init {
                let (_, effect) = check_expr(env, var_env, arg)?;
                total_effect = total_effect.join(effect);
            }
            
            // Return instance type
            let ty = env.spawn_instance_type(workflow_type);
            Ok((ty, total_effect.join(Effect::Operational))) // Spawn is operational
        }
        
        Expr::Split(instance_expr) => {
            let (instance_ty, effect) = check_expr(env, var_env, instance_expr)?;
            let (addr_ty, control_ty) = env.split_types(&instance_ty)?;
            
            // Return tuple of (addr, control)
            Ok((Type::Tuple(vec![addr_ty, control_ty]), effect))
        }
    }
}
```

**File**: `crates/ash-typeck/src/check_workflow.rs` (or extend check_expr.rs)

```rust
fn check_workflow(env: &TypeEnv, var_env: &mut VarEnv, workflow: &Workflow) -> Result<(Type, Effect), TypeError> {
    match workflow {
        // Existing...
        
        Workflow::Spawn { workflow_type, init, binding, continuation } => {
            // Check init
            let mut total_effect = Effect::Operational;
            for (_, arg) in init {
                let (_, effect) = check_expr(env, var_env, arg)?;
                total_effect = total_effect.join(effect);
            }
            
            // Bind instance
            let instance_ty = env.spawn_instance_type(workflow_type);
            var_env.bind(binding.clone(), instance_ty);
            
            // Check continuation
            let (cont_ty, cont_effect) = check_workflow(env, var_env, continuation)?;
            Ok((cont_ty, total_effect.join(cont_effect)))
        }
        
        Workflow::Split { instance, addr_binding, control_binding, continuation } => {
            let instance_ty = var_env.lookup(instance)
                .ok_or_else(|| TypeError::UnboundVariable(instance.clone()))?;
            
            let (addr_ty, control_ty) = env.split_types(&instance_ty)?;
            
            // Bind addr (always available)
            var_env.bind(addr_binding.clone(), addr_ty);
            
            // Bind control as Option<ControlLink>
            var_env.bind(control_binding.clone(), control_ty);
            
            // Check continuation
            check_workflow(env, var_env, continuation)
        }
    }
}
```

### Step 5: Parse Spawn and Split

**File**: `crates/ash-parser/src/parse_workflow.rs` (extend)

```rust
/// Parse spawn: `spawn Worker with { init: args } as w;`
fn parse_spawn(input: &mut Input) -> PResult<Workflow> {
    literal("spawn").parse_next(input)?;
    winnow::combinator::space1.parse_next(input)?;
    
    let workflow_type = parse_ident_uppercase.parse_next(input)?;
    
    winnow::combinator::space1.parse_next(input)?;
    literal("with").parse_next(input)?;
    winnow::combinator::space0.parse_next(input)?;
    
    let init = parse_record_expr.parse_next(input)?;
    
    winnow::combinator::space1.parse_next(input)?;
    literal("as").parse_next(input)?;
    winnow::combinator::space1.parse_next(input)?;
    
    let binding = parse_ident_lowercase.parse_next(input)?;
    
    literal(";").parse_next(input)?;
    
    let continuation = parse_workflow.parse_next(input)?;
    
    Ok(Workflow::Spawn {
        workflow_type: workflow_type.to_string(),
        init: init_to_fields(init),
        binding: binding.to_string(),
        continuation: Box::new(continuation),
    })
}

/// Parse split: `let (addr, ctrl) = split instance;`
fn parse_split_let(input: &mut Input) -> PResult<Workflow> {
    literal("let").parse_next(input)?;
    winnow::combinator::space1.parse_next(input)?;
    literal("(").parse_next(input)?;
    
    let addr_binding = parse_ident_lowercase.parse_next(input)?;
    literal(",").parse_next(input)?;
    winnow::combinator::space0.parse_next(input)?;
    let control_binding = parse_ident_lowercase.parse_next(input)?;
    
    literal(")").parse_next(input)?;
    winnow::combinator::space0.parse_next(input)?;
    literal("=").parse_next(input)?;
    winnow::combinator::space0.parse_next(input)?;
    literal("split").parse_next(input)?;
    winnow::combinator::space1.parse_next(input)?;
    
    let instance = parse_ident_lowercase.parse_next(input)?;
    literal(";").parse_next(input)?;
    
    let continuation = parse_workflow.parse_next(input)?;
    
    Ok(Workflow::Split {
        instance: instance.to_string(),
        addr_binding: addr_binding.to_string(),
        control_binding: control_binding.to_string(),
        continuation: Box::new(continuation),
    })
}
```

### Step 6: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_type() {
        let env = TypeEnv::new();
        let mut var_env = VarEnv::new();
        
        let expr = Expr::Spawn {
            workflow_type: "Worker".into(),
            init: vec![],
        };
        
        let (ty, _) = check_expr(&env, &mut var_env, &expr).unwrap();
        
        assert!(matches!(ty, Type::Instance { workflow_type } 
            if workflow_type == "Worker"));
    }

    #[test]
    fn test_split_type() {
        let env = TypeEnv::new();
        let mut var_env = VarEnv::new();
        
        // First spawn
        let spawn = Expr::Spawn {
            workflow_type: "Worker".into(),
            init: vec![],
        };
        let (instance_ty, _) = check_expr(&env, &mut var_env, &spawn).unwrap();
        
        // Then split
        let split = Expr::Split(Box::new(spawn));
        let (ty, _) = check_expr(&env, &mut var_env, &split).unwrap();
        
        // Should be tuple (InstanceAddr<Worker>, Option<ControlLink<Worker>>)
        match ty {
            Type::Tuple(types) => {
                assert_eq!(types.len(), 2);
                assert!(matches!(&types[0], Type::InstanceAddr { .. }));
                assert!(matches!(&types[1], Type::Constructor { name, .. } if name == "Option"));
            }
            _ => panic!("Expected tuple type"),
        }
    }
}
```

### Step 7: Run Tests

```bash
cargo test -p ash-typeck spawn -- --nocapture
cargo test -p ash-parser spawn -- --nocapture
```

## Completion Checklist

- [ ] Instance, InstanceAddr, ControlLink types added to AST
- [ ] Instance types added to Type enum
- [ ] Built-in Option<ControlLink> type
- [ ] Spawn type checking
- [ ] Split type checking
- [ ] Parser for spawn expression
- [ ] Parser for split workflow
- [ ] Type environment for instance types
- [ ] Error handling for invalid split
- [ ] Unit tests for spawn/split types
- [ ] Integration tests
- [ ] `cargo fmt` and `cargo clippy` pass

## Estimated Effort

6 hours

## Dependencies

- TASK-121 (ADT Core Types)
- TASK-127 (Type Check Constructors)

## Blocked By

- TASK-121
- TASK-127

## Blocks

- TASK-135 (Control Link Transfer)
- TASK-136 (Standard Library)
