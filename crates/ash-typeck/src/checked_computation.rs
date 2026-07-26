//! Immutable AST-directed computation evidence for source handlers.
//!
//! This is deliberately a typechecker-only seam.  It records requirements and
//! source anchors; it neither creates a thunk nor installs a runtime frame.

use ash_parser::{
    Spanned,
    surface::{
        BlockStmt, ConstructorPayload, Definition, Expr, FnDef, Program, Type as SurfaceType,
    },
};

use crate::{
    TypeCheckError, TypeEnv,
    check_expr::check_expr,
    handler_rows::{
        NormalizedHandlerRow, normalize_handler_row_in_env, normalized_declared_operation,
        normalized_handler_rows_semantically_equal, row_normalization_env,
        union_normalized_handler_rows,
    },
    surface_type_lowering::{bind_surface_type_parameters, workflow_surface_type_to_type},
    types::Type,
};

/// Immutable evidence for the implicit `Unit -> {row} result` computation
/// accepted by canonical source handlers.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckedComputation {
    result_type: Type,
    normalized_row: NormalizedHandlerRow,
    expression_anchor: ash_parser::token::Span,
}

impl CheckedComputation {
    #[must_use]
    pub const fn result_type(&self) -> &Type {
        &self.result_type
    }

    #[must_use]
    pub const fn normalized_row(&self) -> &NormalizedHandlerRow {
        &self.normalized_row
    }

    #[must_use]
    pub const fn expression_anchor(&self) -> ash_parser::token::Span {
        self.expression_anchor
    }
}

fn unsupported(anchor: ash_parser::token::Span) -> TypeCheckError {
    TypeCheckError::UnsupportedHandlerComputation {
        message: "unsupported-handler-computation-expression".to_string(),
        span: anchor,
    }
}

fn checked_pure(
    env: &TypeEnv,
    expr: &Expr,
    diagnostic_anchor: ash_parser::token::Span,
) -> Result<CheckedComputation, TypeCheckError> {
    let result = check_expr(env, expr);
    if !result.is_ok() {
        return Err(unsupported(diagnostic_anchor));
    }
    Ok(CheckedComputation {
        result_type: result.substitution.apply(&result.ty),
        normalized_row: union_normalized_handler_rows(&[]).expect("empty rows cannot conflict"),
        expression_anchor: diagnostic_anchor,
    })
}

fn union_with_result(
    env: &TypeEnv,
    expr: &Expr,
    diagnostic_anchor: ash_parser::token::Span,
    children: impl IntoIterator<Item = Result<CheckedComputation, TypeCheckError>>,
) -> Result<CheckedComputation, TypeCheckError> {
    let children = children.into_iter().collect::<Result<Vec<_>, _>>()?;
    let result = check_expr(env, expr);
    if !result.is_ok() {
        return Err(unsupported(diagnostic_anchor));
    }
    let rows = children
        .iter()
        .map(|child| child.normalized_row.clone())
        .collect::<Vec<_>>();
    Ok(CheckedComputation {
        result_type: result.substitution.apply(&result.ty),
        normalized_row: union_normalized_handler_rows(&rows)
            .map_err(|_| unsupported(diagnostic_anchor))?,
        expression_anchor: diagnostic_anchor,
    })
}

fn branch_pattern_environment(
    env: &TypeEnv,
    pattern: &ash_parser::surface::Pattern,
    scrutinee: &Expr,
    pattern_fallback_anchor: ash_parser::token::Span,
) -> Result<TypeEnv, TypeCheckError> {
    let checked_scrutinee = check_expr(env, scrutinee);
    if !checked_scrutinee.is_ok() {
        return Err(unsupported(scrutinee.span()));
    }
    let scrutinee_type = checked_scrutinee.substitution.apply(&checked_scrutinee.ty);
    let pattern_env = crate::check_expr::pattern_type_env_from_type_env(env);
    let bindings = match env.canonicalize_type_for_pattern(&scrutinee_type) {
        crate::PatternCanonicalization::Matchable(canonical) => {
            crate::check_pattern::check_pattern_with_canonical_type(
                &pattern_env,
                pattern,
                &canonical,
            )
        }
        crate::PatternCanonicalization::Blocked { .. } => {
            crate::check_pattern::check_pattern(&pattern_env, pattern, &scrutinee_type)
        }
    }
    .map_err(|_| {
        unsupported(crate::check_expr::surface_pattern_span(
            pattern,
            pattern_fallback_anchor,
        ))
    })?;

    let mut branch_env = env.clone();
    for (name, ty) in bindings {
        branch_env.bind_variable(&name, ty);
    }
    Ok(branch_env)
}

fn infer(
    env: &TypeEnv,
    expr: &Expr,
    computation_variable: Option<(&str, &CheckedComputation)>,
    diagnostic_anchor: ash_parser::token::Span,
) -> Result<CheckedComputation, TypeCheckError> {
    match expr {
        Expr::Call {
            module: Some(impl_type),
            func,
            args,
            span,
        } => {
            let declared = env
                .resolve_declared_concrete_operation(impl_type, func)
                .map_err(|_| unsupported(diagnostic_anchor))?;
            let result = check_expr(env, expr);
            if !result.is_ok() {
                return Err(unsupported(diagnostic_anchor));
            }
            let mut rows = args
                .iter()
                .map(|arg| infer(env, arg, computation_variable, arg.span()))
                .collect::<Result<Vec<_>, _>>()?;
            rows.push(CheckedComputation {
                result_type: declared.result_type.clone(),
                normalized_row: normalized_declared_operation(&declared, *span),
                expression_anchor: diagnostic_anchor,
            });
            Ok(CheckedComputation {
                result_type: result.substitution.apply(&result.ty),
                normalized_row: union_normalized_handler_rows(
                    &rows
                        .iter()
                        .map(|row| row.normalized_row.clone())
                        .collect::<Vec<_>>(),
                )
                .map_err(|_| unsupported(diagnostic_anchor))?,
                expression_anchor: diagnostic_anchor,
            })
        }
        Expr::Literal(_) => checked_pure(env, expr, diagnostic_anchor),
        Expr::Variable { name, .. }
            if computation_variable.is_some_and(|(variable, _)| name.as_ref() == variable) =>
        {
            let (_, fact) = computation_variable.expect("guard proves computation variable exists");
            Ok(CheckedComputation {
                result_type: fact.result_type.clone(),
                normalized_row: fact.normalized_row.clone(),
                expression_anchor: diagnostic_anchor,
            })
        }
        Expr::Variable { .. } => checked_pure(env, expr, diagnostic_anchor),
        Expr::Unary { operand, .. } => union_with_result(
            env,
            expr,
            diagnostic_anchor,
            [infer(env, operand, computation_variable, operand.span())],
        ),
        Expr::Binary { left, right, .. } => union_with_result(
            env,
            expr,
            diagnostic_anchor,
            [
                infer(env, left, computation_variable, left.span()),
                infer(env, right, computation_variable, right.span()),
            ],
        ),
        Expr::List { items, .. } => union_with_result(
            env,
            expr,
            diagnostic_anchor,
            items
                .iter()
                .map(|item| infer(env, item, computation_variable, item.span())),
        ),
        Expr::Record { fields, .. } => union_with_result(
            env,
            expr,
            diagnostic_anchor,
            fields
                .iter()
                .map(|(_, value)| infer(env, value, computation_variable, value.span())),
        ),
        Expr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => union_with_result(
            env,
            expr,
            diagnostic_anchor,
            std::iter::once(infer(
                env,
                condition,
                computation_variable,
                condition.span(),
            ))
            .chain(std::iter::once(infer(
                env,
                then_branch,
                computation_variable,
                then_branch.span(),
            )))
            .chain(
                else_branch
                    .iter()
                    .map(|branch| infer(env, branch, computation_variable, branch.span())),
            ),
        ),
        Expr::Match {
            scrutinee, arms, ..
        } => union_with_result(
            env,
            expr,
            diagnostic_anchor,
            std::iter::once(infer(
                env,
                scrutinee,
                computation_variable,
                scrutinee.span(),
            ))
            .chain(arms.iter().map(|arm| {
                let arm_env = branch_pattern_environment(env, &arm.pattern, scrutinee, arm.span)?;
                infer(&arm_env, &arm.body, computation_variable, arm.body.span())
            })),
        ),
        Expr::IfLet {
            pattern,
            expr: matched,
            then_branch,
            else_branch,
            span,
        } => {
            let then_env = branch_pattern_environment(env, pattern, matched, *span)?;
            union_with_result(
                env,
                expr,
                diagnostic_anchor,
                [
                    infer(env, matched, computation_variable, matched.span()),
                    infer(
                        &then_env,
                        then_branch,
                        computation_variable,
                        then_branch.span(),
                    ),
                    infer(env, else_branch, computation_variable, else_branch.span()),
                ],
            )
        }
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            let direct_root_child_anchor =
                (diagnostic_anchor == expr.span()).then_some(diagnostic_anchor);
            let mut block_env = env.clone();
            let mut children = Vec::new();
            for statement in statements {
                match statement {
                    BlockStmt::Expr { expr, .. } => children.push(infer(
                        &block_env,
                        expr,
                        computation_variable,
                        direct_root_child_anchor.unwrap_or_else(|| expr.span()),
                    )),
                    BlockStmt::Let {
                        pattern,
                        expr,
                        span,
                    } => {
                        children.push(infer(
                            &block_env,
                            expr,
                            computation_variable,
                            direct_root_child_anchor.unwrap_or_else(|| expr.span()),
                        ));
                        let checked = check_expr(&block_env, expr);
                        if !checked.is_ok() {
                            return Err(unsupported(diagnostic_anchor));
                        }
                        let value_type = checked.substitution.apply(&checked.ty);
                        let pattern_span = crate::check_expr::surface_pattern_span(pattern, *span);
                        let bindings = crate::check_expr::check_irrefutable_let_pattern(
                            &block_env,
                            "let",
                            pattern,
                            &value_type,
                            pattern_span,
                        )
                        .map_err(|_| unsupported(diagnostic_anchor))?;
                        crate::check_expr::bind_irrefutable_pattern_bindings(
                            &mut block_env,
                            bindings,
                        );
                    }
                }
            }
            if let Some(tail) = tail_expr {
                children.push(infer(
                    &block_env,
                    tail,
                    computation_variable,
                    direct_root_child_anchor.unwrap_or_else(|| tail.span()),
                ));
            }
            union_with_result(env, expr, diagnostic_anchor, children)
        }
        Expr::Constructor {
            fields, payload, ..
        } => {
            let field_children = fields
                .iter()
                .map(|(_, value)| infer(env, value, computation_variable, value.span()));
            let payload_children: Box<
                dyn Iterator<Item = Result<CheckedComputation, TypeCheckError>>,
            > = match payload {
                ConstructorPayload::Tuple(values) => Box::new(
                    values
                        .iter()
                        .map(|value| infer(env, value, computation_variable, value.span())),
                ),
                ConstructorPayload::Unit | ConstructorPayload::Record(_) => {
                    Box::new(std::iter::empty())
                }
            };
            union_with_result(
                env,
                expr,
                diagnostic_anchor,
                field_children.chain(payload_children),
            )
        }
        _ => Err(unsupported(diagnostic_anchor)),
    }
}

/// Infer immutable computation evidence using an already-scoped environment.
///
/// Handler clause checking uses this after it has checked and installed the
/// operation payload pattern bindings.  The evidence remains typechecker-only:
/// it is not a runtime thunk or an authority grant.
pub(crate) fn infer_checked_computation_in_env(
    env: &TypeEnv,
    expr: &Expr,
) -> Result<CheckedComputation, TypeCheckError> {
    infer(env, expr, None, expr.span())
}

/// Infer a source computation while retaining an explicitly annotated
/// zero-argument computation parameter's row. Ordinary type lowering erases
/// rows, so this recognizes only that lexical call form and remains
/// fail-closed for all other expression shapes.
pub(crate) fn infer_checked_computation_in_env_with_parameter_facts(
    env: &TypeEnv,
    expr: &Expr,
) -> Result<CheckedComputation, TypeCheckError> {
    if let Expr::Call {
        module: None,
        func,
        args,
        ..
    } = expr
        && args.is_empty()
    {
        if let Some(fact) = env.source_computation_fact(func.as_ref()) {
            let result = check_expr(env, expr);
            if !result.is_ok() {
                return Err(unsupported(expr.span()));
            }
            return Ok(CheckedComputation {
                result_type: result.substitution.apply(&result.ty),
                normalized_row: fact.normalized_row.clone(),
                expression_anchor: expr.span(),
            });
        }
        return Err(TypeCheckError::TypeError(
            "unsupported computation boundary".to_string(),
        ));
    }
    infer_checked_computation_in_env(env, expr)
}

/// Extract immutable computation facts from explicitly annotated
/// zero-argument function parameters. These facts are scoped to source
/// handler application inference and never create `TypeEnv` bindings.
pub(crate) fn function_computation_parameter_facts(
    env: &TypeEnv,
    program: &Program,
    function: &FnDef,
) -> Result<std::collections::HashMap<String, CheckedComputation>, TypeCheckError> {
    let (signature_env, bindings) = bind_surface_type_parameters(env, &function.type_params)?;
    let mut facts = std::collections::HashMap::new();
    for parameter in &function.params {
        let SurfaceType::Fn(parameters, Some(row), result) = &parameter.ty else {
            continue;
        };
        if !parameters.is_empty() {
            continue;
        }
        facts.insert(
            parameter.name.to_string(),
            CheckedComputation {
                result_type: workflow_surface_type_to_type(&signature_env, result, &bindings)?,
                normalized_row: normalize_handler_row_in_env(env, program, row)
                    .map_err(|error| TypeCheckError::TypeError(error.to_string()))?,
                expression_anchor: function.span,
            },
        );
    }
    Ok(facts)
}

/// Infer the requirements evaluated as arguments to a scoped direct resume.
/// The resume itself is deliberately not interpreted as an ordinary callable.
pub(crate) fn infer_direct_resume_arguments_in_env(
    env: &TypeEnv,
    arguments: &[&Expr],
    result_type: Type,
    anchor: ash_parser::token::Span,
) -> Result<CheckedComputation, TypeCheckError> {
    let arguments = arguments
        .iter()
        .map(|argument| infer(env, argument, None, argument.span()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CheckedComputation {
        result_type,
        normalized_row: union_normalized_handler_rows(
            &arguments
                .iter()
                .map(|argument| argument.normalized_row.clone())
                .collect::<Vec<_>>(),
        )
        .map_err(|_| unsupported(anchor))?,
        expression_anchor: anchor,
    })
}

fn lookup_function<'a>(program: &'a Program, name: &str) -> Option<&'a ash_parser::surface::FnDef> {
    program
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == name => Some(function),
            _ => None,
        })
}

/// Infer the annotated operand fact for a canonical handler declaration.
///
/// This is crate-visible so declaration checking and the doc-hidden test seam
/// share one AST-directed inference boundary.
pub(crate) fn infer_checked_handler_computation(
    env: &TypeEnv,
    program: &Program,
    handler: &ash_parser::surface::HandlerDef,
) -> Result<CheckedComputation, TypeCheckError> {
    let Expr::On { computation, .. } = &handler.body else {
        return Err(unsupported(handler.body.span()));
    };
    let parameter = handler
        .params
        .first()
        .ok_or_else(|| unsupported(computation.span()))?;
    let (signature_env, bindings) = bind_surface_type_parameters(env, &handler.type_params)?;
    let SurfaceType::Fn(_, row, result) = &parameter.ty else {
        return Err(unsupported(computation.span()));
    };
    let (result_type, annotated_row) = (
        workflow_surface_type_to_type(&signature_env, result, &bindings)?,
        match row {
            Some(row) => normalize_handler_row_in_env(env, program, row)
                .map_err(|error| TypeCheckError::TypeError(error.to_string()))?,
            None => union_normalized_handler_rows(&[])
                .expect("empty handler-computation rows cannot conflict"),
        },
    );
    let annotation = CheckedComputation {
        result_type,
        normalized_row: annotated_row,
        expression_anchor: computation.span(),
    };
    let inferred = infer(
        env,
        computation,
        Some((parameter.name.as_ref(), &annotation)),
        computation.span(),
    )?;
    if env
        .unify_types(&annotation.result_type, &inferred.result_type)
        .is_err()
        || !normalized_handler_rows_semantically_equal(
            &annotation.normalized_row,
            &inferred.normalized_row,
        )
    {
        return Err(unsupported(computation.span()));
    }
    Ok(CheckedComputation {
        result_type: inferred.result_type,
        normalized_row: union_normalized_handler_rows(&[
            annotation.normalized_row,
            inferred.normalized_row,
        ])
        .map_err(|_| unsupported(computation.span()))?,
        expression_anchor: inferred.expression_anchor,
    })
}

/// Doc-hidden test seam for immutable source computation inference.
#[doc(hidden)]
pub fn infer_checked_computation_for_test(
    program: &Program,
    function: &str,
) -> Result<CheckedComputation, TypeCheckError> {
    let env = row_normalization_env(program, &[])
        .map_err(|error| TypeCheckError::TypeError(error.to_string()))?;
    let body = lookup_function(program, function)
        .ok_or_else(|| TypeCheckError::ResolutionError(format!("unknown function '{function}'")))?;
    infer(&env, &body.body, None, body.body.span())
}

/// Doc-hidden test seam for the normalized annotated operand of a canonical
/// `on computation` handler declaration.
#[doc(hidden)]
pub fn infer_checked_handler_computation_for_test(
    program: &Program,
    handler_name: &str,
) -> Result<CheckedComputation, TypeCheckError> {
    let env = row_normalization_env(program, &[])
        .map_err(|error| TypeCheckError::TypeError(error.to_string()))?;
    let handler = program
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Handler(handler) if handler.name.as_ref() == handler_name => Some(handler),
            _ => None,
        })
        .ok_or_else(|| {
            TypeCheckError::ResolutionError(format!("unknown handler '{handler_name}'"))
        })?;
    infer_checked_handler_computation(&env, program, handler)
}

/// Doc-hidden test seam for deterministic structural row union.
#[doc(hidden)]
pub fn union_checked_computations_for_test(
    computations: &[CheckedComputation],
) -> Result<CheckedComputation, TypeCheckError> {
    let Some(first) = computations.first() else {
        return Err(TypeCheckError::TypeError(
            "handler-computation union requires at least one input".to_string(),
        ));
    };
    Ok(CheckedComputation {
        result_type: first.result_type.clone(),
        normalized_row: union_normalized_handler_rows(
            &computations
                .iter()
                .map(|item| item.normalized_row.clone())
                .collect::<Vec<_>>(),
        )
        .map_err(|error| TypeCheckError::TypeError(error.to_string()))?,
        expression_anchor: first.expression_anchor,
    })
}
