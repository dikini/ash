//! Parser for the Core Ash `.core` fixture/debug text format.
//!
//! This module intentionally stays close to the raw Core AST. TASK-1622 covers
//! atoms, types, rows, row items, and values; TASK-1623 adds expression forms.

use crate::core_ash::{
    CoreAtom, CoreCaptureSet, CoreContRef, CoreContractDischarge, CoreDischargeMode, CoreEffectOp,
    CoreEvalMode, CoreExpr, CoreHandlerClause, CoreMultiplicity, CoreParam, CorePrimOp, CoreRow,
    CoreRowItem, CoreSourceSpan, CoreThunkMode, CoreTrapReason, CoreType, CoreValue,
};
use serde::Serialize;
use std::fmt;
use std::fmt::Write as _;
use std::path::Path;

/// Error returned by the Core text parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreTextError {
    message: String,
    position: usize,
}

impl CoreTextError {
    fn new(position: usize, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            position,
        }
    }

    /// Byte position where parsing failed.
    #[must_use]
    pub fn position(&self) -> usize {
        self.position
    }

    /// Human-readable parser message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for CoreTextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "core text parse error at {}: {}",
            self.position, self.message
        )
    }
}

impl std::error::Error for CoreTextError {}

type ParseResult<T> = Result<T, CoreTextError>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    LParen,
    RParen,
    LBrace,
    RBrace,
    Colon,
    Comma,
    Arrow,
    Symbol(String),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    position: usize,
}

/// Parses a Core atom.
///
/// # Errors
///
/// Returns [`CoreTextError`] when the source is not a single valid atom.
pub fn parse_atom(source: &str) -> ParseResult<CoreAtom> {
    Parser::new(source)?.parse_complete(Parser::parse_atom_inner)
}

/// Parses a Core type.
///
/// # Errors
///
/// Returns [`CoreTextError`] when the source is not a single valid type.
pub fn parse_type(source: &str) -> ParseResult<CoreType> {
    Parser::new(source)?.parse_complete(Parser::parse_type_inner)
}

/// Parses a Core row.
///
/// # Errors
///
/// Returns [`CoreTextError`] when the source is not a single valid row.
pub fn parse_row(source: &str) -> ParseResult<CoreRow> {
    Parser::new(source)?.parse_complete(Parser::parse_row_inner)
}

/// Parses a Core row item without surrounding braces.
///
/// # Errors
///
/// Returns [`CoreTextError`] when the source is not a single valid row item.
pub fn parse_row_item(source: &str) -> ParseResult<CoreRowItem> {
    Parser::new(source)?.parse_complete(Parser::parse_row_item_inner)
}

/// Parses a Core value.
///
/// # Errors
///
/// Returns [`CoreTextError`] when the source is not a single valid value.
pub fn parse_value(source: &str) -> ParseResult<CoreValue> {
    Parser::new(source)?.parse_complete(Parser::parse_value_inner)
}

/// Parses a complete Core expression.
///
/// # Errors
///
/// Returns [`CoreTextError`] when the source is not a single valid expression.
pub fn parse_core_expr(source: &str) -> ParseResult<CoreExpr> {
    Parser::new(source)?.parse_complete(Parser::parse_expr_inner)
}

/// Reads and parses a complete `.core` file.
///
/// # Errors
///
/// Returns [`CoreTextError`] when the file cannot be read or the contents are
/// not a single valid expression.
pub fn parse_core_file(path: impl AsRef<Path>) -> ParseResult<CoreExpr> {
    let path = path.as_ref();
    let source = std::fs::read_to_string(path).map_err(|error| {
        CoreTextError::new(0, format!("failed to read {}: {error}", path.display()))
    })?;
    parse_core_expr(&source)
}

/// Serializes a Core expression to canonical `.core` fixture text.
#[must_use]
pub fn core_expr_to_string(expr: &CoreExpr) -> String {
    format_expr(expr)
}

/// Writes a Core expression to a `.core` file with a trailing newline.
///
/// # Errors
///
/// Returns [`std::io::Error`] when the destination cannot be written.
pub fn write_core_expr_to_file(path: impl AsRef<Path>, expr: &CoreExpr) -> std::io::Result<()> {
    let mut text = core_expr_to_string(expr);
    text.push('\n');
    std::fs::write(path, text)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(source: &str) -> ParseResult<Self> {
        Ok(Self {
            tokens: lex(source)?,
            pos: 0,
        })
    }

    fn parse_complete<T>(
        mut self,
        parse: impl FnOnce(&mut Self) -> ParseResult<T>,
    ) -> ParseResult<T> {
        let value = parse(&mut self)?;
        if self.is_eof() {
            Ok(value)
        } else {
            Err(self.error_here("unexpected trailing tokens"))
        }
    }

    fn parse_expr_inner(&mut self) -> ParseResult<CoreExpr> {
        if self.peek_symbol().is_some() {
            return Ok(CoreExpr::Atom(self.parse_atom_inner()?));
        }
        if !self.next_is_lparen() {
            return Err(self.error_here("expected expression"));
        }

        let checkpoint = self.pos;
        self.expect_lparen()?;
        let head = self.expect_symbol()?;
        let expr = match head.as_str() {
            "let-val" => {
                let name = self.expect_symbol()?;
                self.expect_colon()?;
                let ty = self.parse_type_inner()?;
                let value = self.parse_value_inner()?;
                let body = self.parse_expr_inner()?;
                CoreExpr::LetVal {
                    name,
                    ty,
                    value,
                    body: Box::new(body),
                }
            }
            "let-rec" => {
                let name = self.expect_symbol()?;
                self.expect_colon()?;
                let ty = self.parse_type_inner()?;
                let value = self.parse_value_inner()?;
                let body = self.parse_expr_inner()?;
                CoreExpr::LetRec {
                    name,
                    ty,
                    value,
                    body: Box::new(body),
                }
            }
            "let-prim" => {
                let name = self.expect_symbol()?;
                let op = parse_prim_op(&self.expect_symbol()?);
                let args = self.parse_atom_list()?;
                let body = self.parse_expr_inner()?;
                CoreExpr::LetPrim {
                    name,
                    op,
                    args,
                    body: Box::new(body),
                }
            }
            "let-call" => {
                let name = self.expect_symbol()?;
                let func = self.parse_atom_inner()?;
                let args = self.parse_atom_list()?;
                let body = self.parse_expr_inner()?;
                CoreExpr::LetCall {
                    name,
                    func,
                    args,
                    body: Box::new(body),
                }
            }
            "let-mode" => {
                let name = self.expect_symbol()?;
                let mode = self.parse_eval_mode()?;
                self.expect_colon()?;
                let ty = self.parse_type_inner()?;
                let expr = self.parse_expr_inner()?;
                let body = self.parse_expr_inner()?;
                CoreExpr::LetMode {
                    name,
                    mode,
                    ty,
                    expr: Box::new(expr),
                    body: Box::new(body),
                }
            }
            "force" => {
                let name = self.expect_symbol()?;
                let thunk = self.parse_atom_inner()?;
                let body = self.parse_expr_inner()?;
                CoreExpr::Force {
                    name,
                    thunk,
                    body: Box::new(body),
                }
            }
            "if" => {
                let cond = self.parse_atom_inner()?;
                let then_branch = self.parse_expr_inner()?;
                let else_branch = self.parse_expr_inner()?;
                CoreExpr::If {
                    cond,
                    then_branch: Box::new(then_branch),
                    else_branch: Box::new(else_branch),
                }
            }
            "call" => CoreExpr::Call {
                func: self.parse_atom_inner()?,
                args: self.parse_atom_list()?,
            },
            "jump" => CoreExpr::Jump {
                cont: self.parse_cont_ref_inner()?,
                arg: self.parse_atom_inner()?,
            },
            "let-cont-call" => CoreExpr::LetContCall {
                name: self.expect_symbol()?,
                cont: self.parse_cont_ref_inner()?,
                arg: self.parse_atom_inner()?,
                body: Box::new(self.parse_expr_inner()?),
            },
            "raise" => CoreExpr::Raise {
                op: self.parse_effect_op_inner()?,
                args: self.parse_atom_list()?,
            },
            "handle" => CoreExpr::Handle {
                clause: self.parse_handler_clause_inner()?,
                body: Box::new(self.parse_expr_inner()?),
            },
            "record-discharge" => CoreExpr::RecordDischarge {
                discharge: self.parse_contract_discharge_inner()?,
                body: Box::new(self.parse_expr_inner()?),
            },
            "trap" => CoreExpr::Trap {
                reason: self.parse_trap_reason_inner()?,
            },
            "lit-int" | "lit-string" | "lit-bool" | "lit-unit" | "prim" | "constructor" => {
                self.pos = checkpoint;
                return Ok(CoreExpr::Atom(self.parse_atom_inner()?));
            }
            other => {
                return Err(self.error_here(format!("unsupported expression form `{other}`")));
            }
        };
        self.expect_rparen()?;
        Ok(expr)
    }

    fn parse_atom_inner(&mut self) -> ParseResult<CoreAtom> {
        match self.next_kind()? {
            TokenKind::Symbol(symbol) => Ok(symbol_to_atom(symbol)),
            TokenKind::LParen => {
                let head = self.expect_symbol()?;
                let atom = match head.as_str() {
                    "lit-int" => {
                        let raw = self.expect_symbol()?;
                        let value = raw.parse::<i64>().map_err(|_| {
                            self.error_here(format!("invalid integer literal `{raw}`"))
                        })?;
                        CoreAtom::LitInt(value)
                    }
                    "lit-string" => CoreAtom::LitString(self.expect_string()?),
                    "lit-bool" => match self.expect_symbol()?.as_str() {
                        "true" => CoreAtom::LitBool(true),
                        "false" => CoreAtom::LitBool(false),
                        other => {
                            return Err(
                                self.error_here(format!("invalid boolean literal `{other}`"))
                            );
                        }
                    },
                    "lit-unit" => CoreAtom::LitUnit,
                    "prim" => CoreAtom::PrimName(parse_prim_op(&self.expect_symbol()?)),
                    "constructor" => CoreAtom::ConstructorName(self.expect_symbol()?),
                    other => {
                        return Err(self.error_here(format!("unsupported atom form `{other}`")));
                    }
                };
                self.expect_rparen()?;
                Ok(atom)
            }
            other => Err(self.error_from_kind(other, "expected atom")),
        }
    }

    fn parse_type_inner(&mut self) -> ParseResult<CoreType> {
        match self.next_kind()? {
            TokenKind::Symbol(symbol) => Ok(symbol_to_type(symbol)),
            TokenKind::LParen => {
                let head = self.expect_symbol()?;
                let ty = match head.as_str() {
                    "strict" => {
                        let inner = self.parse_type_inner()?;
                        CoreType::Mode {
                            mode: CoreEvalMode::Strict,
                            inner: Box::new(inner),
                            latent_row: None,
                        }
                    }
                    "lazy" => {
                        let inner = self.parse_type_inner()?;
                        let row = self.parse_row_inner()?;
                        CoreType::Mode {
                            mode: CoreEvalMode::Lazy,
                            inner: Box::new(inner),
                            latent_row: Some(row),
                        }
                    }
                    "memo" => {
                        let inner = self.parse_type_inner()?;
                        let row = self.parse_row_inner()?;
                        CoreType::Mode {
                            mode: CoreEvalMode::Memo,
                            inner: Box::new(inner),
                            latent_row: Some(row),
                        }
                    }
                    "fn" => self.parse_function_type()?,
                    "cont" => self.parse_cont_type()?,
                    "refine" => {
                        let base = self.parse_type_inner()?;
                        let predicate = self.expect_string()?;
                        CoreType::Refinement {
                            base: Box::new(base),
                            predicate,
                        }
                    }
                    "tuple" => {
                        let mut elems = Vec::new();
                        while !self.consume_rparen() {
                            elems.push(self.parse_type_inner()?);
                        }
                        return Ok(CoreType::Tuple(elems));
                    }
                    "record-type" => {
                        let mut fields = Vec::new();
                        while !self.consume_rparen() {
                            self.expect_lparen()?;
                            let name = self.expect_symbol()?;
                            self.expect_colon()?;
                            let ty = self.parse_type_inner()?;
                            self.expect_rparen()?;
                            fields.push((name, ty));
                        }
                        return Ok(CoreType::Record(fields));
                    }
                    "type-app" => {
                        let name = self.expect_symbol()?;
                        self.expect_lparen()?;
                        let mut args = Vec::new();
                        while !self.consume_rparen() {
                            args.push(self.parse_type_inner()?);
                        }
                        CoreType::App { name, args }
                    }
                    other => {
                        return Err(self.error_here(format!("unsupported type form `{other}`")));
                    }
                };
                self.expect_rparen()?;
                Ok(ty)
            }
            other => Err(self.error_from_kind(other, "expected type")),
        }
    }

    fn parse_function_type(&mut self) -> ParseResult<CoreType> {
        self.expect_lparen()?;
        let mut params = Vec::new();
        while !self.consume_rparen() {
            params.push(self.parse_type_inner()?);
        }
        self.expect_arrow()?;
        let result = self.parse_type_inner()?;
        let row = self.parse_row_inner()?;
        Ok(CoreType::Function {
            params,
            result: Box::new(result),
            row,
        })
    }

    fn parse_cont_type(&mut self) -> ParseResult<CoreType> {
        let input = self.parse_type_inner()?;
        let answer = self.parse_type_inner()?;
        let row = self.parse_row_inner()?;
        let multiplicity = match self.expect_symbol()?.as_str() {
            "affine" | "Affine" => CoreMultiplicity::Affine,
            "multi-shot-pure" | "MultiShotPure" => CoreMultiplicity::MultiShotPure,
            other => return Err(self.error_here(format!("unsupported multiplicity `{other}`"))),
        };
        Ok(CoreType::Cont {
            input: Box::new(input),
            answer: Box::new(answer),
            row,
            multiplicity,
        })
    }

    fn parse_row_inner(&mut self) -> ParseResult<CoreRow> {
        self.expect_lbrace()?;
        let mut items = Vec::new();
        let mut tail = None;
        if self.consume_rbrace() {
            return Ok(CoreRow::default());
        }
        loop {
            if self.peek_symbol().is_some_and(|symbol| symbol == "tail") {
                let _ = self.expect_symbol()?;
                if tail.replace(self.expect_symbol()?).is_some() {
                    return Err(self.error_here("duplicate row tail"));
                }
            } else {
                items.push(self.parse_row_item_inner()?);
            }
            if self.consume_comma() {
                continue;
            }
            if self.consume_rbrace() {
                break;
            }
            return Err(self.error_here("expected `,` or `}` after row item"));
        }
        Ok(CoreRow { items, tail })
    }

    fn parse_row_item_inner(&mut self) -> ParseResult<CoreRowItem> {
        let head = self.expect_symbol()?;
        match head.as_str() {
            "operation" => {
                let (path, operation) = split_path_operation(&self.expect_symbol()?)?;
                Ok(CoreRowItem::operation(path, operation))
            }
            "resource" => Ok(CoreRowItem::Resource {
                path: split_path(&self.expect_symbol()?),
                mode: self.expect_symbol()?,
            }),
            "role" => Ok(CoreRowItem::Role {
                path: split_path(&self.expect_symbol()?),
            }),
            "policy" => Ok(CoreRowItem::Policy {
                path: split_path(&self.expect_symbol()?),
            }),
            "contract" => Ok(CoreRowItem::Contract {
                contract: self.expect_symbol()?,
            }),
            "channel" => Ok(CoreRowItem::Channel {
                path: split_path(&self.expect_symbol()?),
                mode: self.expect_symbol()?,
                payload_type: Box::new(self.parse_type_inner()?),
            }),
            "process" => Ok(CoreRowItem::Process {
                operation: self.expect_symbol()?,
            }),
            "fail" => {
                let ty = if self.row_item_boundary() {
                    None
                } else {
                    Some(Box::new(self.parse_type_inner()?))
                };
                Ok(CoreRowItem::Failure { ty })
            }
            "evidence" => Ok(CoreRowItem::Evidence {
                path: split_path(&self.expect_symbol()?),
            }),
            "group" => Ok(CoreRowItem::EffectGroupRef {
                path: split_path(&self.expect_symbol()?),
            }),
            _ => Err(self.error_here(format!("unsupported row item `{head}`"))),
        }
    }

    fn parse_value_inner(&mut self) -> ParseResult<CoreValue> {
        if self.peek_symbol().is_some() {
            return Ok(CoreValue::Atom(self.parse_atom_inner()?));
        }
        if !self.next_is_lparen() {
            return Err(self.error_here("expected value"));
        }
        let checkpoint = self.pos;
        self.expect_lparen()?;
        let head = self.expect_symbol()?;
        let value = match head.as_str() {
            "lam" => self.parse_lam_value()?,
            "record" => self.parse_record_value()?,
            "tuple" => self.parse_tuple_value()?,
            "discharge-marker" => CoreValue::DischargeMarker {
                discharge: self.parse_contract_discharge_inner()?,
            },
            "thunk" => {
                let mode = match self.expect_symbol()?.as_str() {
                    "lazy" => CoreThunkMode::Lazy,
                    "memo" => CoreThunkMode::Memo,
                    other => {
                        return Err(self.error_here(format!("unsupported thunk mode `{other}`")));
                    }
                };
                let result_ty = self.parse_type_inner()?;
                let row = self.parse_row_inner()?;
                let body = self.parse_expr_inner()?;
                CoreValue::Thunk {
                    mode,
                    result_ty,
                    body: Box::new(body),
                    row,
                    captures: CoreCaptureSet { values: Vec::new() },
                }
            }
            "lit-int" | "lit-string" | "lit-bool" | "lit-unit" | "prim" | "constructor" => {
                self.pos = checkpoint;
                return Ok(CoreValue::Atom(self.parse_atom_inner()?));
            }
            other => return Err(self.error_here(format!("unsupported value form `{other}`"))),
        };
        self.expect_rparen()?;
        Ok(value)
    }

    fn parse_lam_value(&mut self) -> ParseResult<CoreValue> {
        self.expect_lparen()?;
        let mut params = Vec::new();
        while !self.consume_rparen() {
            self.expect_lparen()?;
            let name = self.expect_symbol()?;
            self.expect_colon()?;
            let ty = self.parse_type_inner()?;
            self.expect_rparen()?;
            params.push(CoreParam { name, ty });
        }
        self.expect_colon()?;
        let row = self.parse_row_inner()?;
        let body = self.parse_expr_inner()?;
        Ok(CoreValue::Lam {
            params,
            body: Box::new(body),
            row,
        })
    }

    fn parse_record_value(&mut self) -> ParseResult<CoreValue> {
        let mut fields = Vec::new();
        while !self.next_is_rparen() {
            self.expect_lparen()?;
            let name = self.expect_symbol()?;
            let value = self.parse_atom_inner()?;
            self.expect_rparen()?;
            fields.push((name, value));
        }
        Ok(CoreValue::Record { fields })
    }

    fn parse_tuple_value(&mut self) -> ParseResult<CoreValue> {
        let mut elems = Vec::new();
        while !self.next_is_rparen() {
            elems.push(self.parse_atom_inner()?);
        }
        Ok(CoreValue::Tuple { elems })
    }

    fn parse_contract_discharge_inner(&mut self) -> ParseResult<CoreContractDischarge> {
        self.expect_lparen()?;
        let head = self.expect_symbol()?;
        if head != "contract" {
            return Err(self.error_here(format!("expected `contract`, got `{head}`")));
        }
        let contract = self.expect_symbol()?;
        let mode = match self.expect_symbol()?.as_str() {
            "static" => CoreDischargeMode::Static,
            "evidence" => CoreDischargeMode::Evidence,
            "dynamic" => CoreDischargeMode::Dynamic,
            other => return Err(self.error_here(format!("unsupported discharge mode `{other}`"))),
        };
        let mut source_span = None;
        if self.peek_symbol().is_some_and(|symbol| symbol == "span") {
            let _ = self.expect_symbol()?;
            source_span = Some(CoreSourceSpan {
                file: None,
                start: self.expect_usize()?,
                end: self.expect_usize()?,
            });
        }
        self.expect_rparen()?;
        Ok(CoreContractDischarge {
            contract,
            mode,
            evidence: None,
            source_span,
        })
    }

    fn parse_effect_op_inner(&mut self) -> ParseResult<CoreEffectOp> {
        self.expect_lparen()?;
        let head = self.expect_symbol()?;
        let op = match head.as_str() {
            "operation" => {
                let (path, operation) = split_path_operation(&self.expect_symbol()?)?;
                self.expect_colon()?;
                let (arg_types, result_type) = self.parse_signature()?;
                CoreEffectOp::Operation {
                    path,
                    operation,
                    arg_types,
                    result_type,
                }
            }
            "channel" => {
                let path = split_path(&self.expect_symbol()?);
                let mode = self.expect_symbol()?;
                self.expect_colon()?;
                let payload_type = self.parse_type_inner()?;
                self.expect_arrow()?;
                let result_type = self.parse_type_inner()?;
                CoreEffectOp::Channel {
                    path,
                    mode,
                    payload_type,
                    result_type,
                }
            }
            "process" => {
                let operation = self.expect_symbol()?;
                self.expect_colon()?;
                let (arg_types, result_type) = self.parse_signature()?;
                CoreEffectOp::Process {
                    operation,
                    arg_types,
                    result_type,
                }
            }
            "fail" => {
                let ty = if self.next_is_rparen() {
                    None
                } else {
                    Some(self.parse_type_inner()?)
                };
                CoreEffectOp::Failure { ty }
            }
            other => return Err(self.error_here(format!("unsupported effect op `{other}`"))),
        };
        self.expect_rparen()?;
        Ok(op)
    }

    fn parse_handler_clause_inner(&mut self) -> ParseResult<CoreHandlerClause> {
        self.expect_lparen()?;
        let head = self.expect_symbol()?;
        if head != "clause" {
            return Err(self.error_here(format!("expected `clause`, got `{head}`")));
        }
        let op = self.parse_effect_op_inner()?;
        let params = self.parse_param_list()?;
        let resume = self.parse_resume_param()?;
        self.expect_colon()?;
        let row = self.parse_row_inner()?;
        let body = self.parse_expr_inner()?;
        self.expect_rparen()?;
        Ok(CoreHandlerClause {
            op,
            params,
            resume,
            body: Box::new(body),
            row,
        })
    }

    fn parse_trap_reason_inner(&mut self) -> ParseResult<CoreTrapReason> {
        self.expect_lparen()?;
        let head = self.expect_symbol()?;
        let reason = match head.as_str() {
            "contract-violation" => CoreTrapReason::ContractViolation(self.expect_symbol()?),
            "contract-violation-diagnostic" => CoreTrapReason::ContractViolationDiagnostic(
                self.expect_json_payload("contract violation diagnostic")?,
            ),
            "contract-predicate-fault" => CoreTrapReason::ContractPredicateFault(
                self.expect_json_payload("contract predicate fault diagnostic")?,
            ),
            "temporal-contract-violation" => CoreTrapReason::TemporalContractViolation(
                self.expect_json_payload("temporal contract diagnostic")?,
            ),
            "temporal-monitor-fault" => CoreTrapReason::TemporalMonitorFault(
                self.expect_json_payload("temporal monitor fault diagnostic")?,
            ),
            "unhandled-effect" => CoreTrapReason::UnhandledEffect(self.parse_effect_op_inner()?),
            "panic" => CoreTrapReason::Panic(self.expect_string()?),
            "non-exhaustive-match" => CoreTrapReason::NonExhaustiveMatch,
            other => return Err(self.error_here(format!("unsupported trap reason `{other}`"))),
        };
        self.expect_rparen()?;
        Ok(reason)
    }

    fn parse_signature(&mut self) -> ParseResult<(Vec<CoreType>, CoreType)> {
        self.expect_lparen()?;
        let mut arg_types = Vec::new();
        while !self.consume_rparen() {
            arg_types.push(self.parse_type_inner()?);
        }
        self.expect_arrow()?;
        let result_type = self.parse_type_inner()?;
        Ok((arg_types, result_type))
    }

    fn parse_atom_list(&mut self) -> ParseResult<Vec<CoreAtom>> {
        self.expect_lparen()?;
        let mut args = Vec::new();
        while !self.consume_rparen() {
            args.push(self.parse_atom_inner()?);
        }
        Ok(args)
    }

    fn parse_param_list(&mut self) -> ParseResult<Vec<CoreParam>> {
        self.expect_lparen()?;
        let mut params = Vec::new();
        while !self.consume_rparen() {
            self.expect_lparen()?;
            let name = self.expect_symbol()?;
            self.expect_colon()?;
            let ty = self.parse_type_inner()?;
            self.expect_rparen()?;
            params.push(CoreParam { name, ty });
        }
        Ok(params)
    }

    fn parse_resume_param(&mut self) -> ParseResult<CoreParam> {
        self.expect_lparen()?;
        let head = self.expect_symbol()?;
        if head != "resume" {
            return Err(self.error_here(format!("expected `resume`, got `{head}`")));
        }
        let name = self.expect_symbol()?;
        self.expect_colon()?;
        let ty = self.parse_type_inner()?;
        self.expect_rparen()?;
        Ok(CoreParam { name, ty })
    }

    fn parse_cont_ref_inner(&mut self) -> ParseResult<CoreContRef> {
        match self.next_kind()? {
            TokenKind::Symbol(name) => Ok(CoreContRef::Var(name)),
            TokenKind::LParen => {
                let head = self.expect_symbol()?;
                let cont = match head.as_str() {
                    "label" => CoreContRef::Label(self.expect_symbol()?),
                    "var" => CoreContRef::Var(self.expect_symbol()?),
                    other => {
                        return Err(
                            self.error_here(format!("unsupported continuation ref `{other}`"))
                        );
                    }
                };
                self.expect_rparen()?;
                Ok(cont)
            }
            other => Err(self.error_from_kind(other, "expected continuation reference")),
        }
    }

    fn parse_eval_mode(&mut self) -> ParseResult<CoreEvalMode> {
        match self.expect_symbol()?.as_str() {
            "strict" => Ok(CoreEvalMode::Strict),
            "lazy" => Ok(CoreEvalMode::Lazy),
            "memo" => Ok(CoreEvalMode::Memo),
            other => Err(self.error_here(format!(
                "unsupported eval mode `{other}`, expected strict, lazy, or memo"
            ))),
        }
    }

    fn expect_usize(&mut self) -> ParseResult<usize> {
        let raw = self.expect_symbol()?;
        raw.parse::<usize>()
            .map_err(|_| self.error_here(format!("invalid usize literal `{raw}`")))
    }

    fn row_item_boundary(&self) -> bool {
        self.is_eof()
            || matches!(
                self.peek_kind(),
                Some(TokenKind::Comma | TokenKind::RBrace | TokenKind::RParen)
            )
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos).map(|token| &token.kind)
    }

    fn peek_symbol(&self) -> Option<&str> {
        match self.peek_kind() {
            Some(TokenKind::Symbol(symbol)) => Some(symbol),
            _ => None,
        }
    }

    fn next_kind(&mut self) -> ParseResult<TokenKind> {
        let token = self
            .tokens
            .get(self.pos)
            .ok_or_else(|| self.error_here("unexpected end of input"))?
            .clone();
        self.pos += 1;
        Ok(token.kind)
    }

    fn expect_symbol(&mut self) -> ParseResult<String> {
        match self.next_kind()? {
            TokenKind::Symbol(symbol) => Ok(symbol),
            other => Err(self.error_from_kind(other, "expected symbol")),
        }
    }

    fn expect_string(&mut self) -> ParseResult<String> {
        match self.next_kind()? {
            TokenKind::String(value) => Ok(value),
            other => Err(self.error_from_kind(other, "expected string literal")),
        }
    }

    fn expect_json_payload<T>(&mut self, what: &str) -> ParseResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let payload = self.expect_string()?;
        serde_json::from_str(&payload)
            .map_err(|err| self.error_here(format!("invalid {what} JSON payload: {err}")))
    }

    fn expect_lparen(&mut self) -> ParseResult<()> {
        match self.next_kind()? {
            TokenKind::LParen => Ok(()),
            other => Err(self.error_from_kind(other, "expected `(`")),
        }
    }

    fn expect_rparen(&mut self) -> ParseResult<()> {
        match self.next_kind()? {
            TokenKind::RParen => Ok(()),
            other => Err(self.error_from_kind(other, "expected `)`")),
        }
    }

    fn expect_lbrace(&mut self) -> ParseResult<()> {
        match self.next_kind()? {
            TokenKind::LBrace => Ok(()),
            other => Err(self.error_from_kind(other, "expected `{`")),
        }
    }

    fn expect_colon(&mut self) -> ParseResult<()> {
        match self.next_kind()? {
            TokenKind::Colon => Ok(()),
            other => Err(self.error_from_kind(other, "expected `:`")),
        }
    }

    fn expect_arrow(&mut self) -> ParseResult<()> {
        match self.next_kind()? {
            TokenKind::Arrow => Ok(()),
            other => Err(self.error_from_kind(other, "expected `->`")),
        }
    }

    fn consume_rparen(&mut self) -> bool {
        self.consume_if(|kind| matches!(kind, TokenKind::RParen))
    }

    fn consume_rbrace(&mut self) -> bool {
        self.consume_if(|kind| matches!(kind, TokenKind::RBrace))
    }

    fn consume_comma(&mut self) -> bool {
        self.consume_if(|kind| matches!(kind, TokenKind::Comma))
    }

    fn consume_if(&mut self, predicate: impl FnOnce(&TokenKind) -> bool) -> bool {
        if self.peek_kind().is_some_and(predicate) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn next_is_lparen(&self) -> bool {
        matches!(self.peek_kind(), Some(TokenKind::LParen))
    }

    fn next_is_rparen(&self) -> bool {
        matches!(self.peek_kind(), Some(TokenKind::RParen))
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn error_here(&self, message: impl Into<String>) -> CoreTextError {
        let position = self.tokens.get(self.pos).map_or_else(
            || self.tokens.last().map_or(0, |token| token.position + 1),
            |token| token.position,
        );
        CoreTextError::new(position, message)
    }

    fn error_from_kind(&self, kind: TokenKind, expected: &str) -> CoreTextError {
        self.error_here(format!("{expected}, got {}", describe_token(&kind)))
    }
}

fn lex(source: &str) -> ParseResult<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = source.char_indices().peekable();
    while let Some((position, ch)) = chars.next() {
        match ch {
            ch if ch.is_whitespace() => {}
            '(' => tokens.push(Token {
                kind: TokenKind::LParen,
                position,
            }),
            ')' => tokens.push(Token {
                kind: TokenKind::RParen,
                position,
            }),
            '{' => tokens.push(Token {
                kind: TokenKind::LBrace,
                position,
            }),
            '}' => tokens.push(Token {
                kind: TokenKind::RBrace,
                position,
            }),
            ':' => tokens.push(Token {
                kind: TokenKind::Colon,
                position,
            }),
            ',' => tokens.push(Token {
                kind: TokenKind::Comma,
                position,
            }),
            '-' if chars.peek().is_some_and(|(_, next)| *next == '>') => {
                let _ = chars.next();
                tokens.push(Token {
                    kind: TokenKind::Arrow,
                    position,
                });
            }
            '"' => {
                let mut value = String::new();
                let mut terminated = false;
                while let Some((_, next)) = chars.next() {
                    match next {
                        '"' => {
                            terminated = true;
                            break;
                        }
                        '\\' => {
                            let escaped = chars.next().ok_or_else(|| {
                                CoreTextError::new(position, "unterminated string escape")
                            })?;
                            value.push(match escaped.1 {
                                'n' => '\n',
                                'r' => '\r',
                                't' => '\t',
                                '"' => '"',
                                '\\' => '\\',
                                other => other,
                            });
                        }
                        other => value.push(other),
                    }
                }
                if !terminated {
                    return Err(CoreTextError::new(position, "unterminated string literal"));
                }
                tokens.push(Token {
                    kind: TokenKind::String(value),
                    position,
                });
            }
            _ => {
                let mut symbol = String::from(ch);
                while let Some((_, next)) = chars.peek() {
                    if next.is_whitespace()
                        || matches!(next, '(' | ')' | '{' | '}' | ':' | ',' | '"')
                    {
                        break;
                    }
                    symbol.push(*next);
                    let _ = chars.next();
                }
                tokens.push(Token {
                    kind: TokenKind::Symbol(symbol),
                    position,
                });
            }
        }
    }
    Ok(tokens)
}

fn format_expr(expr: &CoreExpr) -> String {
    match expr {
        CoreExpr::Atom(atom) => format_atom(atom),
        CoreExpr::LetVal {
            name,
            ty,
            value,
            body,
        } => format!(
            "(let-val {name} : {} {} {})",
            format_type(ty),
            format_value(value),
            format_expr(body)
        ),
        CoreExpr::LetMode {
            name,
            mode,
            ty,
            expr,
            body,
        } => format!(
            "(let-mode {name} {} : {} {} {})",
            format_eval_mode(*mode),
            format_type(ty),
            format_expr(expr),
            format_expr(body)
        ),
        CoreExpr::LetRec {
            name,
            ty,
            value,
            body,
        } => format!(
            "(let-rec {name} : {} {} {})",
            format_type(ty),
            format_value(value),
            format_expr(body)
        ),
        CoreExpr::LetPrim {
            name,
            op,
            args,
            body,
        } => format!(
            "(let-prim {name} {} {} {})",
            format_prim_op(op),
            format_atom_list(args),
            format_expr(body)
        ),
        CoreExpr::LetCall {
            name,
            func,
            args,
            body,
        } => format!(
            "(let-call {name} {} {} {})",
            format_atom(func),
            format_atom_list(args),
            format_expr(body)
        ),
        CoreExpr::If {
            cond,
            then_branch,
            else_branch,
        } => format!(
            "(if {} {} {})",
            format_atom(cond),
            format_expr(then_branch),
            format_expr(else_branch)
        ),
        CoreExpr::Call { func, args } => {
            format!("(call {} {})", format_atom(func), format_atom_list(args))
        }
        CoreExpr::Jump { cont, arg } => {
            format!("(jump {} {})", format_cont_ref(cont), format_atom(arg))
        }
        CoreExpr::LetContCall {
            name,
            cont,
            arg,
            body,
        } => format!(
            "(let-cont-call {name} {} {} {})",
            format_cont_ref(cont),
            format_atom(arg),
            format_expr(body)
        ),
        CoreExpr::Raise { op, args } => {
            format!(
                "(raise {} {})",
                format_effect_op(op),
                format_atom_list(args)
            )
        }
        CoreExpr::Handle { clause, body } => {
            format!(
                "(handle {} {})",
                format_handler_clause(clause),
                format_expr(body)
            )
        }
        CoreExpr::Force { name, thunk, body } => format!(
            "(force {name} {} {})",
            format_atom(thunk),
            format_expr(body)
        ),
        CoreExpr::RecordDischarge { discharge, body } => format!(
            "(record-discharge {} {})",
            format_contract_discharge(discharge),
            format_expr(body)
        ),
        CoreExpr::Trap { reason } => format!("(trap {})", format_trap_reason(reason)),
    }
}

fn format_value(value: &CoreValue) -> String {
    match value {
        CoreValue::Atom(atom) => format_atom(atom),
        CoreValue::Lam { params, body, row } => format!(
            "(lam {} : {} {})",
            format_param_list(params),
            format_row(row),
            format_expr(body)
        ),
        CoreValue::Thunk {
            mode,
            result_ty,
            body,
            row,
            ..
        } => format!(
            "(thunk {} {} {} {})",
            format_thunk_mode(*mode),
            format_type(result_ty),
            format_row(row),
            format_expr(body)
        ),
        CoreValue::Record { fields } => {
            let fields = fields
                .iter()
                .map(|(name, atom)| format!("({name} {})", format_atom(atom)))
                .collect::<Vec<_>>()
                .join(" ");
            if fields.is_empty() {
                "(record)".to_string()
            } else {
                format!("(record {fields})")
            }
        }
        CoreValue::Tuple { elems } => {
            if elems.is_empty() {
                "(tuple)".to_string()
            } else {
                format!(
                    "(tuple {})",
                    elems.iter().map(format_atom).collect::<Vec<_>>().join(" ")
                )
            }
        }
        CoreValue::DischargeMarker { discharge } => {
            format!(
                "(discharge-marker {})",
                format_contract_discharge(discharge)
            )
        }
    }
}

fn format_atom(atom: &CoreAtom) -> String {
    match atom {
        CoreAtom::Var(name) => name.clone(),
        CoreAtom::LitInt(value) => format!("(lit-int {value})"),
        CoreAtom::LitString(value) => format!("(lit-string {})", format_string(value)),
        CoreAtom::LitBool(value) => format!("(lit-bool {value})"),
        CoreAtom::LitUnit => "(lit-unit)".to_string(),
        CoreAtom::PrimName(op) => format!("(prim {})", format_prim_op(op)),
        CoreAtom::ConstructorName(name) => format!("(constructor {name})"),
    }
}

fn format_type(ty: &CoreType) -> String {
    match ty {
        CoreType::Base(name) | CoreType::Named(name) | CoreType::Var(name) => name.clone(),
        CoreType::Function {
            params,
            result,
            row,
        } => format!(
            "(fn ({}) -> {} {})",
            params.iter().map(format_type).collect::<Vec<_>>().join(" "),
            format_type(result),
            format_row(row)
        ),
        CoreType::Refinement { base, predicate } => {
            format!(
                "(refine {} {})",
                format_type(base),
                format_string(predicate)
            )
        }
        CoreType::Cont {
            input,
            answer,
            row,
            multiplicity,
        } => format!(
            "(cont {} {} {} {})",
            format_type(input),
            format_type(answer),
            format_row(row),
            format_multiplicity(*multiplicity)
        ),
        CoreType::Tuple(elems) => format!(
            "(tuple {})",
            elems.iter().map(format_type).collect::<Vec<_>>().join(" ")
        ),
        CoreType::Record(fields) => format!(
            "(record-type {})",
            fields
                .iter()
                .map(|(name, ty)| format!("({name} : {})", format_type(ty)))
                .collect::<Vec<_>>()
                .join(" ")
        ),
        CoreType::Mode {
            mode,
            inner,
            latent_row,
        } => {
            let mode = format_eval_mode(*mode);
            if let Some(row) = latent_row {
                format!("({mode} {} {})", format_type(inner), format_row(row))
            } else {
                format!("(strict {})", format_type(inner))
            }
        }
        CoreType::App { name, args } => format!(
            "(type-app {name} ({}))",
            args.iter().map(format_type).collect::<Vec<_>>().join(" ")
        ),
    }
}

fn format_eval_mode(mode: CoreEvalMode) -> &'static str {
    match mode {
        CoreEvalMode::Strict => "strict",
        CoreEvalMode::Lazy => "lazy",
        CoreEvalMode::Memo => "memo",
    }
}

fn format_thunk_mode(mode: CoreThunkMode) -> &'static str {
    match mode {
        CoreThunkMode::Lazy => "lazy",
        CoreThunkMode::Memo => "memo",
    }
}

fn format_row(row: &CoreRow) -> String {
    let mut items = row.items.iter().map(format_row_item).collect::<Vec<_>>();
    if let Some(tail) = &row.tail {
        items.push(format!("tail {tail}"));
    }
    if items.is_empty() {
        "{}".to_string()
    } else {
        format!("{{{}}}", items.join(", "))
    }
}

/// Formats one Core row item using canonical target spelling.
#[must_use]
pub fn format_row_item(item: &CoreRowItem) -> String {
    match item {
        CoreRowItem::Operation { path, operation } => {
            format!("operation {}", format_path_operation(path, operation))
        }
        CoreRowItem::Resource { path, mode } => {
            format!("resource {} {mode}", format_path(path))
        }
        CoreRowItem::Role { path } => format!("role {}", format_path(path)),
        CoreRowItem::Policy { path } => format!("policy {}", format_path(path)),
        CoreRowItem::Contract { contract } => format!("contract {contract}"),
        CoreRowItem::Channel {
            path,
            mode,
            payload_type,
        } => format!(
            "channel {} {mode} {}",
            format_path(path),
            format_type(payload_type)
        ),
        CoreRowItem::Process { operation } => format!("process {operation}"),
        CoreRowItem::Failure { ty } => match ty {
            Some(ty) => format!("fail {}", format_type(ty)),
            None => "fail".to_string(),
        },
        CoreRowItem::Evidence { path } => format!("evidence {}", format_path(path)),
        CoreRowItem::EffectGroupRef { path } => format!("group {}", format_path(path)),
    }
}

fn format_effect_op(op: &CoreEffectOp) -> String {
    match op {
        CoreEffectOp::Operation {
            path,
            operation,
            arg_types,
            result_type,
        } => format!(
            "(operation {} : ({}) -> {})",
            format_path_operation(path, operation),
            arg_types
                .iter()
                .map(format_type)
                .collect::<Vec<_>>()
                .join(" "),
            format_type(result_type)
        ),
        CoreEffectOp::Channel {
            path,
            mode,
            payload_type,
            result_type,
        } => format!(
            "(channel {} {mode} : {} -> {})",
            format_path(path),
            format_type(payload_type),
            format_type(result_type)
        ),
        CoreEffectOp::Process {
            operation,
            arg_types,
            result_type,
        } => format!(
            "(process {operation} : ({}) -> {})",
            arg_types
                .iter()
                .map(format_type)
                .collect::<Vec<_>>()
                .join(" "),
            format_type(result_type)
        ),
        CoreEffectOp::Failure { ty } => match ty {
            Some(ty) => format!("(fail {})", format_type(ty)),
            None => "(fail)".to_string(),
        },
    }
}

fn format_handler_clause(clause: &CoreHandlerClause) -> String {
    format!(
        "(clause {} {} (resume {} : {}) : {} {})",
        format_effect_op(&clause.op),
        format_param_list(&clause.params),
        clause.resume.name,
        format_type(&clause.resume.ty),
        format_row(&clause.row),
        format_expr(&clause.body)
    )
}

fn format_contract_discharge(discharge: &CoreContractDischarge) -> String {
    let mut text = format!(
        "(contract {} {}",
        discharge.contract,
        format_discharge_mode(discharge.mode)
    );
    if let Some(span) = &discharge.source_span {
        let _ = write!(text, " span {} {}", span.start, span.end);
    }
    text.push(')');
    text
}

fn format_trap_reason(reason: &CoreTrapReason) -> String {
    match reason {
        CoreTrapReason::ContractViolation(contract) => {
            format!("(contract-violation {contract})")
        }
        CoreTrapReason::ContractViolationDiagnostic(diagnostic) => format!(
            "(contract-violation-diagnostic {})",
            format_string(&json_payload(diagnostic))
        ),
        CoreTrapReason::ContractPredicateFault(diagnostic) => format!(
            "(contract-predicate-fault {})",
            format_string(&json_payload(diagnostic))
        ),
        CoreTrapReason::TemporalContractViolation(diagnostic) => format!(
            "(temporal-contract-violation {})",
            format_string(&json_payload(diagnostic))
        ),
        CoreTrapReason::TemporalMonitorFault(diagnostic) => format!(
            "(temporal-monitor-fault {})",
            format_string(&json_payload(diagnostic))
        ),
        CoreTrapReason::UnhandledEffect(op) => {
            format!("(unhandled-effect {})", format_effect_op(op))
        }
        CoreTrapReason::Panic(message) => format!("(panic {})", format_string(message)),
        CoreTrapReason::NonExhaustiveMatch => "(non-exhaustive-match)".to_string(),
    }
}

fn format_param_list(params: &[CoreParam]) -> String {
    format!(
        "({})",
        params
            .iter()
            .map(|param| format!("({} : {})", param.name, format_type(&param.ty)))
            .collect::<Vec<_>>()
            .join(" ")
    )
}

fn format_atom_list(args: &[CoreAtom]) -> String {
    format!(
        "({})",
        args.iter().map(format_atom).collect::<Vec<_>>().join(" ")
    )
}

fn format_cont_ref(cont: &CoreContRef) -> String {
    match cont {
        CoreContRef::Label(name) => format!("(label {name})"),
        CoreContRef::Var(name) => name.clone(),
    }
}

fn format_prim_op(op: &CorePrimOp) -> String {
    match op {
        CorePrimOp::Add => "add".to_string(),
        CorePrimOp::Sub => "sub".to_string(),
        CorePrimOp::Mul => "mul".to_string(),
        CorePrimOp::Div => "div".to_string(),
        CorePrimOp::Eq => "eq".to_string(),
        CorePrimOp::Ne => "ne".to_string(),
        CorePrimOp::Lt => "lt".to_string(),
        CorePrimOp::Le => "le".to_string(),
        CorePrimOp::Gt => "gt".to_string(),
        CorePrimOp::Ge => "ge".to_string(),
        CorePrimOp::Neg => "neg".to_string(),
        CorePrimOp::Not => "not".to_string(),
        CorePrimOp::RecordGet(field) => format!("record-get-{field}"),
        CorePrimOp::TupleGet(index) => format!("tuple-get-{index}"),
        CorePrimOp::ConstructorTag(name) => name.clone(),
    }
}

fn format_multiplicity(multiplicity: CoreMultiplicity) -> &'static str {
    match multiplicity {
        CoreMultiplicity::Affine => "affine",
        CoreMultiplicity::MultiShotPure => "multi-shot-pure",
    }
}

fn format_discharge_mode(mode: CoreDischargeMode) -> &'static str {
    match mode {
        CoreDischargeMode::Static => "static",
        CoreDischargeMode::Evidence => "evidence",
        CoreDischargeMode::Dynamic => "dynamic",
    }
}

fn format_path(path: &[String]) -> String {
    path.join(".")
}

fn format_path_operation(path: &[String], operation: &str) -> String {
    if path.is_empty() {
        operation.to_string()
    } else {
        format!("{}.{}", format_path(path), operation)
    }
}

fn format_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for ch in value.chars() {
        match ch {
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            other => escaped.push(other),
        }
    }
    escaped.push('"');
    escaped
}

fn json_payload<T>(value: &T) -> String
where
    T: Serialize + fmt::Debug,
{
    serde_json::to_string(value).unwrap_or_else(|_| format!("{value:?}"))
}

fn symbol_to_atom(symbol: String) -> CoreAtom {
    match symbol.as_str() {
        "add" | "sub" | "mul" | "div" | "eq" | "ne" | "lt" | "le" | "gt" | "ge" | "neg" | "not" => {
            CoreAtom::PrimName(parse_prim_op(&symbol))
        }
        _ => CoreAtom::Var(symbol),
    }
}

fn parse_prim_op(symbol: &str) -> CorePrimOp {
    match symbol {
        "add" => CorePrimOp::Add,
        "sub" => CorePrimOp::Sub,
        "mul" => CorePrimOp::Mul,
        "div" => CorePrimOp::Div,
        "eq" => CorePrimOp::Eq,
        "ne" => CorePrimOp::Ne,
        "lt" => CorePrimOp::Lt,
        "le" => CorePrimOp::Le,
        "gt" => CorePrimOp::Gt,
        "ge" => CorePrimOp::Ge,
        "neg" => CorePrimOp::Neg,
        "not" => CorePrimOp::Not,
        other => CorePrimOp::ConstructorTag(other.to_string()),
    }
}

fn symbol_to_type(symbol: String) -> CoreType {
    match symbol.as_str() {
        "Int" | "String" | "Bool" | "Unit" => CoreType::Base(symbol),
        _ => CoreType::Named(symbol),
    }
}

fn split_path_operation(raw: &str) -> ParseResult<(Vec<String>, String)> {
    let mut parts = split_path(raw);
    let operation = parts
        .pop()
        .ok_or_else(|| CoreTextError::new(0, "expected operation path"))?;
    if parts.is_empty() {
        return Err(CoreTextError::new(
            0,
            format!("expected path.operation, got `{raw}`"),
        ));
    }
    Ok((parts, operation))
}

fn split_path(raw: &str) -> Vec<String> {
    raw.split('.')
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

fn describe_token(kind: &TokenKind) -> String {
    match kind {
        TokenKind::LParen => "`(`".to_string(),
        TokenKind::RParen => "`)`".to_string(),
        TokenKind::LBrace => "`{`".to_string(),
        TokenKind::RBrace => "`}`".to_string(),
        TokenKind::Colon => "`:`".to_string(),
        TokenKind::Comma => "`,`".to_string(),
        TokenKind::Arrow => "`->`".to_string(),
        TokenKind::Symbol(symbol) => format!("symbol `{symbol}`"),
        TokenKind::String(_) => "string literal".to_string(),
    }
}
