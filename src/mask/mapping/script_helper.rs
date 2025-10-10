use std::collections::HashMap;
use std::fmt;

use bevy::math::Vec2;
use pest::iterators::Pair;
use pest::{Parser, Span};
use pest_derive::Parser;
use rust_i18n::t;
use tokio::sync::broadcast;

use crate::mask::mapping::utils::{ControlMsgHelper, MIN_MOVE_STEP_INTERVAL, ease_sigmoid_like};
use crate::scrcpy::constant::{KeyEventAction, Keycode, MetaState, MotionEventAction};
use crate::scrcpy::control_msg::ScrcpyControlMsg;

#[derive(Parser)]
#[grammar = "src/mask/mapping/script.pest"]
struct ScriptParser;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Bool(bool),
    Str(String),
}

#[derive(Default, Debug, Clone)]
pub struct ScriptAST {
    pub program: Program,
    pub script: String,
    pub empty: bool,
    pub build_in_funcs:
        HashMap<String, fn(&str, &SourceSpan, &[Value]) -> Result<Value, ScriptError>>,
}

impl ScriptAST {
    pub fn new(script: &str) -> Result<Self, String> {
        let program_pair = ScriptParser::parse(Rule::program, script)
            .map_err(|e| {
                format!(
                    "{}\n: {}",
                    t!("mask.mapping.parseScriptFailed"),
                    e.to_string()
                )
            })?
            .next()
            .ok_or_else(|| t!("mask.mapping.noProgramFound").to_string())?;

        let mut ast = ScriptAST::default();
        if script.is_empty() {
            ast.empty = true;
            return Ok(ast);
        }

        ast.empty = false;
        ast.script = script.to_string();
        ast.program = ast.parse_program(program_pair);
        if !ast.program.errors.is_empty() {
            return Err(ast
                .program
                .errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n\n"));
        }

        ast.build_in_funcs
            .insert("print".to_string(), |_source, _span, args: &[Value]| {
                let output = args
                    .iter()
                    .map(|val| match val {
                        Value::Int(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Str(s) => s.clone(),
                    })
                    .collect::<Vec<String>>()
                    .join(" ");
                log::info!("{}", output);
                Ok(Value::Int(0))
            });

        ast.build_in_funcs.insert(
            "wait".to_string(),
            |source: &str, span: &SourceSpan, args: &[Value]| {
                if args.len() != 1 || !matches!(args[0], Value::Int(_)) {
                    return Err(ScriptError::from_span(
                        span.clone(),
                        source,
                        "The wait function takes one argument: time (int)".to_string(),
                    ));
                }
                std::thread::sleep(std::time::Duration::from_millis(
                    Self::to_int_value(&args[0]) as u64,
                ));
                Ok(Value::Int(0))
            },
        );

        Ok(ast)
    }

    pub fn eval_script(
        &self,
        cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
        original_size: Vec2,
        cursor_pos: Vec2,
    ) -> Result<(), ScriptError> {
        if self.empty {
            return Ok(());
        }

        let mut vars: HashMap<String, Value> = HashMap::new();
        vars.insert("ORIGINAL_W".to_string(), Value::Int(original_size.x as i64));
        vars.insert("ORIGINAL_H".to_string(), Value::Int(original_size.y as i64));
        vars.insert("CURSOR_X".to_string(), Value::Int(cursor_pos.x as i64));
        vars.insert("CURSOR_Y".to_string(), Value::Int(cursor_pos.y as i64));

        let mut funcs: HashMap<
            String,
            Box<dyn Fn(&str, &SourceSpan, &[Value]) -> Result<Value, ScriptError>>,
        > = HashMap::new();

        funcs.insert(
            "tap".to_string(),
            Box::new(move |s, span, args| tap_func(s, span, args, cs_tx, original_size)),
        );
        funcs.insert(
            "swipe".to_string(),
            Box::new(move |s, span, args| swipe_func(s, span, args, cs_tx, original_size)),
        );
        funcs.insert(
            "send_key".to_string(),
            Box::new(move |s, span, args| send_key_func(s, span, args, cs_tx)),
        );
        funcs.insert(
            "paste_text".to_string(),
            Box::new(move |s, span, args| paste_text_func(s, span, args, cs_tx)),
        );

        for stmt in self.program.stmts.iter() {
            self.eval_stmt(stmt, &mut vars, &funcs)?;
        }

        Ok(())
    }

    fn eval_stmt(
        &self,
        stmt: &Stmt,
        vars: &mut HashMap<String, Value>,
        funcs: &HashMap<String, impl Fn(&str, &SourceSpan, &[Value]) -> Result<Value, ScriptError>>,
    ) -> Result<(), ScriptError> {
        match stmt {
            Stmt::Let { name, expr, span } => {
                let val = self
                    .eval_expr(expr, vars, funcs)
                    .map_err(|e| e.with_outer_span(span.clone(), &self.script))?;
                vars.insert(name.clone(), val);
                Ok(())
            }
            Stmt::Assign { name, expr, span } => {
                let val = self
                    .eval_expr(expr, vars, funcs)
                    .map_err(|e| e.with_outer_span(span.clone(), &self.script))?;

                if vars.contains_key(name) {
                    vars.insert(name.clone(), val);
                    Ok(())
                } else {
                    Err(ScriptError::from_span(
                        span.clone(),
                        &self.script,
                        format!("Variable '{}' not defined", name),
                    ))
                }
            }
            Stmt::Expr { expr, span } => match self.eval_expr(expr, vars, funcs) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.with_outer_span(span.clone(), &self.script)),
            },
            Stmt::Block { stmts, .. } => {
                for stmt in stmts {
                    self.eval_stmt(stmt, vars, funcs)?;
                }
                Ok(())
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
                span,
            } => {
                let cond_val = self
                    .eval_expr(condition, vars, funcs)
                    .map_err(|e| e.with_outer_span(span.clone(), &self.script))?;

                if Self::is_truthy(&cond_val) {
                    self.eval_stmt(then_block, vars, funcs)?;
                } else if let Some(else_stmt) = else_block {
                    self.eval_stmt(else_stmt.as_ref(), vars, funcs)?;
                }

                Ok(())
            }
            Stmt::While {
                condition,
                body,
                span,
            } => {
                while {
                    let cond_val = self
                        .eval_expr(condition, vars, funcs)
                        .map_err(|e| e.with_outer_span(span.clone(), &self.script))?;
                    Self::is_truthy(&cond_val)
                } {
                    self.eval_stmt(body, vars, funcs)?;
                }
                Ok(())
            }
            Stmt::Error { .. } => unreachable!("Error statement reached"),
        }
    }

    fn to_int_value(val: &Value) -> i64 {
        match val {
            Value::Int(n) => *n,
            Value::Bool(b) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            _ => unreachable!(),
        }
    }

    fn is_truthy(val: &Value) -> bool {
        match val {
            Value::Int(n) => *n != 0,
            Value::Bool(b) => *b,
            Value::Str(s) => !s.is_empty(),
        }
    }

    fn is_numeric_value(val: &Value) -> bool {
        matches!(val, Value::Int(_) | Value::Bool(_))
    }

    fn are_numeric_values(lhs: &Value, rhs: &Value) -> bool {
        matches!(lhs, Value::Int(_) | Value::Bool(_))
            && matches!(rhs, Value::Int(_) | Value::Bool(_))
    }

    fn are_comparable_values(lhs: &Value, rhs: &Value) -> bool {
        matches!(
            (lhs, rhs),
            (Value::Int(_), Value::Int(_))
                | (Value::Bool(_), Value::Bool(_))
                | (Value::Str(_), Value::Str(_))
                | (
                    Value::Int(_) | Value::Bool(_),
                    Value::Int(_) | Value::Bool(_)
                )
        )
    }

    fn eval_expr(
        &self,
        expr: &Expr,
        vars: &mut HashMap<String, Value>,
        funcs: &HashMap<String, impl Fn(&str, &SourceSpan, &[Value]) -> Result<Value, ScriptError>>,
    ) -> Result<Value, ScriptError> {
        match expr {
            Expr::Number { value, .. } => Ok(Value::Int(*value)),
            Expr::Bool { value, .. } => Ok(Value::Bool(*value)),
            Expr::Str { value, .. } => Ok(Value::Str(value.clone())),
            Expr::Var { name, span } => {
                if let Some(val) = vars.get(name) {
                    Ok(val.clone())
                } else {
                    Err(ScriptError::from_span(
                        span.clone(),
                        &self.script,
                        format!("Variable '{}' not defined", name),
                    ))
                }
            }
            Expr::Call { name, args, span } => {
                // build in funcs
                if let Some(func) = self.build_in_funcs.get(name) {
                    let mut arg_values = Vec::new();
                    for arg in args {
                        arg_values.push(self.eval_expr(arg, vars, funcs)?);
                    }
                    func(&self.script, span, &arg_values)
                        .map_err(|e| e.with_outer_span(span.clone(), &self.script))
                // custom funcs
                } else if let Some(func) = funcs.get(name) {
                    let mut arg_values = Vec::new();
                    for arg in args {
                        arg_values.push(self.eval_expr(arg, vars, funcs)?);
                    }
                    func(&self.script, span, &arg_values)
                        .map_err(|e| e.with_outer_span(span.clone(), &self.script))
                } else {
                    Err(ScriptError::from_span(
                        span.clone(),
                        &self.script,
                        format!("Function '{}' not defined", name),
                    ))
                }
            }
            Expr::Unary { op, rhs, span } => {
                let rhs_val = self.eval_expr(rhs, vars, funcs)?;
                match op {
                    UnaryOp::Plus => {
                        if Self::is_numeric_value(&rhs_val) {
                            Ok(Value::Int(Self::to_int_value(&rhs_val)))
                        } else {
                            Err(ScriptError::from_span(
                                span.clone(),
                                &self.script,
                                format!("Unary plus operator only supports integers or booleans"),
                            ))
                        }
                    }
                    UnaryOp::Minus => {
                        if Self::is_numeric_value(&rhs_val) {
                            Ok(Value::Int(-Self::to_int_value(&rhs_val)))
                        } else {
                            Err(ScriptError::from_span(
                                span.clone(),
                                &self.script,
                                format!("Unary minus operator only supports integers or booleans"),
                            ))
                        }
                    }
                    UnaryOp::Not => Ok(Value::Bool(!Self::is_truthy(&rhs_val))),
                }
            }
            Expr::Binary { lhs, op, rhs, span } => {
                let lhs_val = self.eval_expr(lhs, vars, funcs)?;
                let rhs_val = self.eval_expr(rhs, vars, funcs)?;

                match op {
                    BinOp::Add => match (&lhs_val, &rhs_val) {
                        (Value::Str(l), Value::Str(r)) => Ok(Value::Str(format!("{}{}", l, r))),
                        _ => {
                            if Self::are_numeric_values(&lhs_val, &rhs_val) {
                                let l = Self::to_int_value(&lhs_val);
                                let r = Self::to_int_value(&rhs_val);
                                Ok(Value::Int(l + r))
                            } else {
                                Err(ScriptError::from_span(
                                    span.clone(),
                                    &self.script,
                                    format!(
                                        "Addition not supported between {:?} and {:?}",
                                        lhs_val, rhs_val
                                    ),
                                ))
                            }
                        }
                    },
                    BinOp::Sub => {
                        if Self::are_numeric_values(&lhs_val, &rhs_val) {
                            let l = Self::to_int_value(&lhs_val);
                            let r = Self::to_int_value(&rhs_val);
                            Ok(Value::Int(l - r))
                        } else {
                            Err(ScriptError::from_span(
                                span.clone(),
                                &self.script,
                                format!(
                                    "Subtraction not supported between {:?} and {:?}",
                                    lhs_val, rhs_val
                                ),
                            ))
                        }
                    }
                    BinOp::Mul => {
                        if Self::are_numeric_values(&lhs_val, &rhs_val) {
                            let l = Self::to_int_value(&lhs_val);
                            let r = Self::to_int_value(&rhs_val);
                            Ok(Value::Int(l * r))
                        } else {
                            Err(ScriptError::from_span(
                                span.clone(),
                                &self.script,
                                format!(
                                    "Multiplication not supported between {:?} and {:?}",
                                    lhs_val, rhs_val
                                ),
                            ))
                        }
                    }
                    BinOp::Div => {
                        if Self::are_numeric_values(&lhs_val, &rhs_val) {
                            let l = Self::to_int_value(&lhs_val);
                            let r = Self::to_int_value(&rhs_val);
                            if r == 0 {
                                Err(ScriptError::from_span(
                                    span.clone(),
                                    &self.script,
                                    "Division by zero".to_string(),
                                ))
                            } else {
                                Ok(Value::Int(l / r))
                            }
                        } else {
                            Err(ScriptError::from_span(
                                span.clone(),
                                &self.script,
                                format!(
                                    "Division not supported between {:?} and {:?}",
                                    lhs_val, rhs_val
                                ),
                            ))
                        }
                    }
                    BinOp::Mod => {
                        if Self::are_numeric_values(&lhs_val, &rhs_val) {
                            let l = Self::to_int_value(&lhs_val);
                            let r = Self::to_int_value(&rhs_val);
                            if r == 0 {
                                Err(ScriptError::from_span(
                                    span.clone(),
                                    &self.script,
                                    "Modulo by zero".to_string(),
                                ))
                            } else {
                                Ok(Value::Int(l % r))
                            }
                        } else {
                            Err(ScriptError::from_span(
                                span.clone(),
                                &self.script,
                                format!(
                                    "Modulo not supported between {:?} and {:?}",
                                    lhs_val, rhs_val
                                ),
                            ))
                        }
                    }
                    BinOp::Lt => {
                        if Self::are_comparable_values(&lhs_val, &rhs_val) {
                            match (&lhs_val, &rhs_val) {
                                (Value::Str(l), Value::Str(r)) => Ok(Value::Bool(l < r)),
                                _ => {
                                    let l = Self::to_int_value(&lhs_val);
                                    let r = Self::to_int_value(&rhs_val);
                                    Ok(Value::Bool(l < r))
                                }
                            }
                        } else {
                            Err(ScriptError::from_span(
                                span.clone(),
                                &self.script,
                                format!(
                                    "Less than comparison not supported between {:?} and {:?}",
                                    lhs_val, rhs_val
                                ),
                            ))
                        }
                    }
                    BinOp::Le => {
                        if Self::are_comparable_values(&lhs_val, &rhs_val) {
                            match (&lhs_val, &rhs_val) {
                                (Value::Str(l), Value::Str(r)) => Ok(Value::Bool(l <= r)),
                                _ => {
                                    let l = Self::to_int_value(&lhs_val);
                                    let r = Self::to_int_value(&rhs_val);
                                    Ok(Value::Bool(l <= r))
                                }
                            }
                        } else {
                            Err(ScriptError::from_span(
                                span.clone(),
                                &self.script,
                                format!(
                                    "Less than or equal comparison not supported between {:?} and {:?}",
                                    lhs_val, rhs_val
                                ),
                            ))
                        }
                    }
                    BinOp::Gt => {
                        if Self::are_comparable_values(&lhs_val, &rhs_val) {
                            match (&lhs_val, &rhs_val) {
                                (Value::Str(l), Value::Str(r)) => Ok(Value::Bool(l > r)),
                                _ => {
                                    let l = Self::to_int_value(&lhs_val);
                                    let r = Self::to_int_value(&rhs_val);
                                    Ok(Value::Bool(l > r))
                                }
                            }
                        } else {
                            Err(ScriptError::from_span(
                                span.clone(),
                                &self.script,
                                format!(
                                    "Greater than comparison not supported between {:?} and {:?}",
                                    lhs_val, rhs_val
                                ),
                            ))
                        }
                    }
                    BinOp::Ge => {
                        if Self::are_comparable_values(&lhs_val, &rhs_val) {
                            match (&lhs_val, &rhs_val) {
                                (Value::Str(l), Value::Str(r)) => Ok(Value::Bool(l >= r)),
                                _ => {
                                    let l = Self::to_int_value(&lhs_val);
                                    let r = Self::to_int_value(&rhs_val);
                                    Ok(Value::Bool(l >= r))
                                }
                            }
                        } else {
                            Err(ScriptError::from_span(
                                span.clone(),
                                &self.script,
                                format!(
                                    "Greater than or equal comparison not supported between {:?} and {:?}",
                                    lhs_val, rhs_val
                                ),
                            ))
                        }
                    }
                    BinOp::Eq => {
                        if Self::are_comparable_values(&lhs_val, &rhs_val) {
                            match (&lhs_val, &rhs_val) {
                                (Value::Str(l), Value::Str(r)) => Ok(Value::Bool(l == r)),
                                _ => {
                                    let l = Self::to_int_value(&lhs_val);
                                    let r = Self::to_int_value(&rhs_val);
                                    Ok(Value::Bool(l == r))
                                }
                            }
                        } else {
                            Ok(Value::Bool(false))
                        }
                    }
                    BinOp::Neq => {
                        if Self::are_comparable_values(&lhs_val, &rhs_val) {
                            match (&lhs_val, &rhs_val) {
                                (Value::Str(l), Value::Str(r)) => Ok(Value::Bool(l != r)),
                                _ => {
                                    let l = Self::to_int_value(&lhs_val);
                                    let r = Self::to_int_value(&rhs_val);
                                    Ok(Value::Bool(l != r))
                                }
                            }
                        } else {
                            Ok(Value::Bool(true))
                        }
                    }
                    BinOp::And => Ok(Value::Bool(
                        Self::is_truthy(&lhs_val) && Self::is_truthy(&rhs_val),
                    )),
                    BinOp::Or => Ok(Value::Bool(
                        Self::is_truthy(&lhs_val) || Self::is_truthy(&rhs_val),
                    )),
                }
            }
        }
    }

    fn parse_program(&self, pair: Pair<Rule>) -> Program {
        let mut stmts = Vec::new();
        let mut errors = Vec::new();

        for stmt_pair in pair.into_inner() {
            match stmt_pair.as_rule() {
                Rule::stmt => stmts.push(self.parse_stmt(stmt_pair, &mut errors)),
                Rule::EOI => {}
                _ => unreachable!(),
            }
        }
        Program { stmts, errors }
    }

    fn parse_stmt(&self, pair: Pair<Rule>, errors: &mut Vec<ScriptError>) -> Stmt {
        let span: SourceSpan = pair.as_span().into();
        let mut it = pair.into_inner();
        let core = it.next().unwrap(); // let_stmt / assign_stmt / expr_stmt

        let rule: Rule = core.as_rule();
        match rule {
            Rule::let_stmt | Rule::assign_stmt => {
                let mut it = core.into_inner();
                let name = it.next().unwrap().as_str().to_string();
                let expr_pair = it.next().unwrap();
                match self.parse_expr(expr_pair) {
                    Ok(expr) => match rule {
                        Rule::let_stmt => Stmt::Let { name, expr, span },
                        Rule::assign_stmt => Stmt::Assign { name, expr, span },
                        r => unreachable!("Unexpected rule {:?}", r),
                    },
                    Err(err) => {
                        errors.push(err.with_outer_span(span, &self.script));
                        Stmt::Error { span }
                    }
                }
            }
            Rule::expr_stmt => {
                let expr_pair = core.into_inner().next().unwrap();
                match self.parse_expr(expr_pair) {
                    Ok(expr) => Stmt::Expr { expr, span },
                    Err(err) => {
                        errors.push(err.with_outer_span(span, &self.script));
                        Stmt::Error { span }
                    }
                }
            }
            Rule::block => {
                let mut stmts = Vec::new();
                let span: SourceSpan = core.as_span().into();

                for stmt_pair in core.into_inner() {
                    if stmt_pair.as_rule() == Rule::stmt {
                        stmts.push(self.parse_stmt(stmt_pair, errors));
                    }
                }

                Stmt::Block { stmts, span }
            }
            Rule::while_stmt => {
                let while_span: SourceSpan = core.as_span().into();
                let mut inner = core.into_inner();

                let condition_pair = inner.next().unwrap();
                let condition = match self.parse_expr(condition_pair) {
                    Ok(expr) => expr,
                    Err(err) => {
                        errors.push(err.with_outer_span(span, &self.script));
                        return Stmt::Error { span };
                    }
                };

                let body_pair = inner.next().unwrap();
                let body = match body_pair.as_rule() {
                    Rule::block => {
                        let mut stmts = Vec::new();
                        let block_span: SourceSpan = body_pair.as_span().into();

                        for stmt_pair in body_pair.into_inner() {
                            if stmt_pair.as_rule() == Rule::stmt {
                                stmts.push(self.parse_stmt(stmt_pair, errors));
                            }
                        }

                        Stmt::Block {
                            stmts,
                            span: block_span,
                        }
                    }
                    r => {
                        errors.push(
                            ScriptError::from_span(
                                body_pair.as_span().into(),
                                &self.script,
                                format!("Expected block statement for while body, but got {:?}", r),
                            )
                            .with_outer_span(span, &self.script),
                        );
                        return Stmt::Error { span };
                    }
                };

                Stmt::While {
                    condition,
                    body: Box::new(body),
                    span: while_span,
                }
            }
            Rule::if_stmt => {
                let if_span: SourceSpan = core.as_span().into();
                let mut inner = core.into_inner();

                let condition_pair = inner.next().unwrap();
                let condition = match self.parse_expr(condition_pair) {
                    Ok(expr) => expr,
                    Err(err) => {
                        errors.push(err.with_outer_span(span, &self.script));
                        return Stmt::Error { span };
                    }
                };

                let then_pair = inner.next().unwrap();
                let then_block = match then_pair.as_rule() {
                    Rule::block => {
                        let mut stmts = Vec::new();
                        let block_span: SourceSpan = then_pair.as_span().into();

                        for stmt_pair in then_pair.into_inner() {
                            if stmt_pair.as_rule() == Rule::stmt {
                                stmts.push(self.parse_stmt(stmt_pair, errors));
                            }
                        }

                        Stmt::Block {
                            stmts,
                            span: block_span,
                        }
                    }
                    r => {
                        errors.push(
                            ScriptError::from_span(
                                then_pair.as_span().into(),
                                &self.script,
                                format!("Expected block statement for if branch, but got {:?}", r),
                            )
                            .with_outer_span(span, &self.script),
                        );
                        return Stmt::Error { span };
                    }
                };

                let else_block = if let Some(else_pair) = inner.next() {
                    match else_pair.as_rule() {
                        Rule::block => {
                            let mut stmts = Vec::new();
                            let block_span: SourceSpan = else_pair.as_span().into();

                            for stmt_pair in else_pair.into_inner() {
                                if stmt_pair.as_rule() == Rule::stmt {
                                    stmts.push(self.parse_stmt(stmt_pair, errors));
                                }
                            }

                            Some(Box::new(Stmt::Block {
                                stmts,
                                span: block_span,
                            }))
                        }
                        r => {
                            errors.push(ScriptError::from_span(
                                else_pair.as_span().into(),
                                &self.script,
                                format!(
                                    "Expected block statement for else branch, but got {:?}",
                                    r
                                ),
                            ));
                            return Stmt::Error { span };
                        }
                    }
                } else {
                    None
                };

                Stmt::If {
                    condition,
                    then_block: Box::new(then_block),
                    else_block,
                    span: if_span,
                }
            }
            _ => Stmt::Error { span },
        }
    }

    fn parse_expr(&self, pair: Pair<Rule>) -> Result<Expr, ScriptError> {
        match pair.as_rule() {
            Rule::number => {
                let span = pair.as_span();
                let val: i64 = pair
                    .as_str()
                    .parse()
                    .map_err(|e: std::num::ParseIntError| {
                        ScriptError::from_span(span.into(), &self.script, e.to_string())
                    })?;
                Ok(Expr::Number {
                    value: val,
                    span: span.into(),
                })
            }
            Rule::boolean => {
                let val = match pair.as_str() {
                    "true" => true,
                    "false" => false,
                    p => unreachable!("Unexpected pair {p}",),
                };

                Ok(Expr::Bool {
                    value: val,
                    span: pair.as_span().into(),
                })
            }
            Rule::string => {
                let raw = pair.as_str();
                // escape \n, \t, \", \\
                let s = raw[1..raw.len() - 1]
                    .replace("\\n", "\n")
                    .replace("\\t", "\t")
                    .replace("\\\"", "\"")
                    .replace("\\\\", "\\");

                Ok(Expr::Str {
                    value: s,
                    span: pair.as_span().into(),
                })
            }
            Rule::ident => Ok(Expr::Var {
                name: pair.as_str().to_string(),
                span: pair.as_span().into(),
            }),
            Rule::call => {
                let expr_span = pair.as_span();
                let mut inner = pair.into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let args = if let Some(arg_list) = inner.next() {
                    arg_list
                        .into_inner()
                        .map(|pair| self.parse_expr(pair))
                        .collect::<Result<Vec<_>, _>>()?
                } else {
                    vec![]
                };
                Ok(Expr::Call {
                    name,
                    args,
                    span: expr_span.into(),
                })
            }
            Rule::prefix => {
                let expr_span = pair.as_span();
                let mut inner = pair.into_inner();
                let first = inner.next().unwrap(); // UnaryOp or atom

                let (op, expr_pair) = match first.as_rule() {
                    Rule::PLUS => (Some(UnaryOp::Plus), inner.next().unwrap()),
                    Rule::MINUS => (Some(UnaryOp::Minus), inner.next().unwrap()),
                    Rule::NOT => (Some(UnaryOp::Not), inner.next().unwrap()),
                    _ => (None, first),
                };

                let rhs = self.parse_expr(expr_pair)?;
                let expr = if let Some(op) = op {
                    Expr::Unary {
                        op,
                        rhs: Box::new(rhs),
                        span: expr_span.into(),
                    }
                } else {
                    rhs
                };

                Ok(expr)
            }
            Rule::product
            | Rule::sum
            | Rule::comparison
            | Rule::equality
            | Rule::logic_and
            | Rule::logic_or => {
                let expr_span = pair.as_span();
                let mut inner = pair.into_inner();
                let mut lhs = self.parse_expr(inner.next().unwrap())?;
                while let Some(op_pair) = inner.next() {
                    let op = match op_pair.as_rule() {
                        Rule::PLUS => BinOp::Add,
                        Rule::MINUS => BinOp::Sub,
                        Rule::STAR => BinOp::Mul,
                        Rule::SLASH => BinOp::Div,
                        Rule::MOD => BinOp::Mod,
                        Rule::LT => BinOp::Lt,
                        Rule::LTE => BinOp::Le,
                        Rule::GT => BinOp::Gt,
                        Rule::GTE => BinOp::Ge,
                        Rule::EQ => BinOp::Eq,
                        Rule::NEQ => BinOp::Neq,
                        Rule::AND => BinOp::And,
                        Rule::OR => BinOp::Or,
                        r => unreachable!("Unexpected rule {:?}", r),
                    };
                    let rhs = self.parse_expr(inner.next().unwrap())?;

                    lhs = Expr::Binary {
                        lhs: Box::new(lhs),
                        op,
                        rhs: Box::new(rhs),
                        span: expr_span.into(),
                    };
                }
                Ok(lhs)
            }
            _ => Err(ScriptError::from_span(
                pair.as_span().into(),
                &self.script,
                format!("Unsupported expr: {:?}", pair.as_rule()),
            )),
        }
    }
}

fn tap_func(
    source: &str,
    span: &SourceSpan,
    args: &[Value],
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
    original_size: Vec2,
) -> Result<Value, ScriptError> {
    // tap(pointer_id, x, y, action?)
    let format_msg = "The tap function takes 3-4 arguments: pointer_id (int), x (int), y (int), action (optional string: 'default', 'down', 'up', or 'move', default is 'default')";

    if args.len() < 3 || args.len() > 4 {
        return Err(ScriptError::from_span(
            span.clone(),
            source,
            format_msg.to_string(),
        ));
    }

    let (pointer_id_val, x_val, y_val, action_val) = match args.len() {
        3 => (
            &args[0],
            &args[1],
            &args[2],
            &Value::Str("default".to_string()),
        ),
        4 => (&args[0], &args[1], &args[2], &args[3]),
        _ => unreachable!(),
    };

    match (pointer_id_val, x_val, y_val, action_val) {
        (Value::Int(p), Value::Int(x), Value::Int(y), Value::Str(action_str)) => {
            let action = match action_str.as_str() {
                "default" | "down" => MotionEventAction::Down,
                "up" => MotionEventAction::Up,
                "move" => MotionEventAction::Move,
                _ => {
                    return Err(ScriptError::from_span(
                        span.clone(),
                        source,
                        format!(
                            "Invalid action '{action_str}', action must be one of 'default', 'down', 'up', or 'move'"
                        ),
                    ));
                }
            };
            let pointer_id: u64 = if *p < 0 {
                return Err(ScriptError::from_span(
                    span.clone(),
                    source,
                    "The pointer_id must be non-negative".to_string(),
                ));
            } else {
                *p as u64
            };

            ControlMsgHelper::send_touch(
                cs_tx,
                action,
                pointer_id,
                original_size,
                (*x as f32, *y as f32).into(),
            );

            if action_str == "default" {
                std::thread::sleep(std::time::Duration::from_millis(30));
                ControlMsgHelper::send_touch(
                    cs_tx,
                    MotionEventAction::Up,
                    pointer_id,
                    original_size,
                    (*x as f32, *y as f32).into(),
                );
            }

            Ok(Value::Int(0))
        }
        _ => Err(ScriptError::from_span(
            span.clone(),
            source,
            format_msg.to_string(),
        )),
    }
}

fn swipe_func(
    source: &str,
    span: &SourceSpan,
    args: &[Value],
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
    original_size: Vec2,
) -> Result<Value, ScriptError> {
    // swipe(pointer_id, interval, x1, y1, x2, y2...)
    let format_msg = "The swipe function takes at least 6 arguments: pointer_id (int), interval (int), x1 (int), y1 (int), x2 (int), y2 (int)...";
    if args.len() < 6 || args.len() % 2 != 0 {
        return Err(ScriptError::from_span(
            span.clone(),
            source,
            format_msg.to_string(),
        ));
    }

    let (pointer_id, interval) = match (&args[0], &args[1]) {
        (Value::Int(p), Value::Int(i)) if *p >= 0 && *i >= 0 => (*p as u64, *i as u64),
        _ => {
            return Err(ScriptError::from_span(
                span.clone(),
                source,
                "The pointer_id and interval must be non-negative integers".to_string(),
            ));
        }
    };

    let points: Result<Vec<Vec2>, ScriptError> = (2..args.len())
        .step_by(2)
        .map(|i| match (&args[i], &args[i + 1]) {
            (Value::Int(x), Value::Int(y)) => Ok(Vec2::new(*x as f32, *y as f32)),
            _ => Err(ScriptError::from_span(
                span.clone(),
                source,
                format!("Coordinates at index {} and {} must be integers", i, i + 1),
            )),
        })
        .collect();

    let points = points?;

    let mut cur_pos = points[0];
    ControlMsgHelper::send_touch(
        &cs_tx,
        MotionEventAction::Down,
        pointer_id,
        original_size,
        cur_pos,
    );
    for i in 1..points.len() {
        let next_pos = points[i];
        let delta = next_pos - cur_pos;
        let steps = std::cmp::max(1, interval / MIN_MOVE_STEP_INTERVAL);
        let step_duration = interval / steps;

        for step in 1..=steps {
            let linear_t = step as f32 / steps as f32;
            let eased_t = ease_sigmoid_like(linear_t);
            let interp = cur_pos + delta * eased_t;
            ControlMsgHelper::send_touch(
                &cs_tx,
                MotionEventAction::Move,
                pointer_id,
                original_size,
                interp,
            );
            std::thread::sleep(std::time::Duration::from_millis(step_duration));
        }

        cur_pos = next_pos;
    }
    ControlMsgHelper::send_touch(
        &cs_tx,
        MotionEventAction::Up,
        pointer_id,
        original_size,
        cur_pos,
    );

    Ok(Value::Int(0))
}

fn send_key_func(
    source: &str,
    span: &SourceSpan,
    args: &[Value],
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
) -> Result<Value, ScriptError> {
    // send_key(key_name, action?, metastate?)
    let format_msg = "The send_key function takes 1-3 arguments: key_name (string), action (optional string: 'down' or 'up', default 'default'), metastate (optional string, default 'NONE')";

    if args.is_empty() || args.len() > 3 {
        return Err(ScriptError::from_span(
            span.clone(),
            source,
            format_msg.to_string(),
        ));
    }

    let key_name = match &args[0] {
        Value::Str(s) => s.as_str(),
        _ => {
            return Err(ScriptError::from_span(
                span.clone(),
                source,
                "First argument must be a string (key_name)".to_string(),
            ));
        }
    };

    let action = if args.len() >= 2 {
        match &args[1] {
            Value::Str(s) => s.as_str(),
            _ => {
                return Err(ScriptError::from_span(
                    span.clone(),
                    source,
                    "Second argument must be a string (action)".to_string(),
                ));
            }
        }
    } else {
        "default"
    };

    let metastate_str = if args.len() >= 3 {
        match &args[2] {
            Value::Str(s) => s.as_str(),
            _ => {
                return Err(ScriptError::from_span(
                    span.clone(),
                    source,
                    "Third argument must be a string (metastate)".to_string(),
                ));
            }
        }
    } else {
        "NONE"
    };

    let key_action = match action {
        "down" => KeyEventAction::Down,
        "up" | "default" => KeyEventAction::Up,
        _ => {
            return Err(ScriptError::from_span(
                span.clone(),
                source,
                format!(
                    "Invalid action '{}', must be 'default', 'down' or 'up'",
                    action
                ),
            ));
        }
    };

    let keycode = match serde_json::from_str::<Keycode>(&format!("\"{}\"", key_name)) {
        Ok(k) => k,
        Err(_) => {
            return Err(ScriptError::from_span(
                span.clone(),
                source,
                format!("Invalid key name '{}'", key_name),
            ));
        }
    };

    let metastate = match serde_json::from_str::<MetaState>(&format!("\"{}\"", metastate_str)) {
        Ok(m) => m,
        Err(_) => {
            return Err(ScriptError::from_span(
                span.clone(),
                source,
                format!("Invalid metastate '{}'", metastate_str),
            ));
        }
    };

    if action == "default" {
        cs_tx
            .send(ScrcpyControlMsg::InjectKeycode {
                action: KeyEventAction::Down,
                keycode: keycode.clone(),
                repeat: 0,
                metastate: metastate.clone(),
            })
            .unwrap();
    }

    cs_tx
        .send(ScrcpyControlMsg::InjectKeycode {
            action: key_action,
            keycode,
            repeat: 0,
            metastate,
        })
        .unwrap();

    Ok(Value::Int(0))
}

fn paste_text_func(
    source: &str,
    span: &SourceSpan,
    args: &[Value],
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
) -> Result<Value, ScriptError> {
    // paste_text(text)
    let format_msg = "The paste_text function takes one argument: text (string)";

    let text = match args {
        [Value::Str(text)] => text,
        _ => {
            return Err(ScriptError::from_span(
                span.clone(),
                source,
                format_msg.to_string(),
            ));
        }
    };

    let sequence = rand::random::<u64>();

    cs_tx
        .send(ScrcpyControlMsg::SetClipboard {
            sequence,
            paste: true,
            text: text.clone(),
        })
        .unwrap();

    Ok(Value::Int(0))
}

#[derive(Clone, Copy, Debug)]
pub struct SourceSpan {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl<'i> From<Span<'i>> for SourceSpan {
    fn from(s: Span<'i>) -> Self {
        let (start_line, start_col) = s.start_pos().line_col();
        let (end_line, end_col) = s.end_pos().line_col();
        Self {
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScriptError {
    pub message: String,
    pub span: SourceSpan,
    pub outer_span: Option<SourceSpan>,
    pub snippet_lines: Vec<String>,
}

impl ScriptError {
    pub fn from_span(span: SourceSpan, source: &str, message: impl ToString) -> ScriptError {
        let snippet_lines: Vec<String> = source
            .lines()
            .skip(span.start_line - 1)
            .take(span.end_line - span.start_line + 1)
            .map(|s| s.to_string())
            .collect();

        ScriptError {
            message: message.to_string(),
            span,
            outer_span: None,
            snippet_lines,
        }
    }

    pub fn with_outer_span(mut self, span: SourceSpan, source: &str) -> Self {
        let snippet_lines: Vec<String> = source
            .lines()
            .skip(span.start_line - 1)
            .take(span.end_line - span.start_line + 1)
            .map(|s| s.to_string())
            .collect();

        self.outer_span = Some(span);
        self.snippet_lines = snippet_lines;
        self
    }
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "error: {}", self.message)?;

        let display_span = self.outer_span.unwrap_or(self.span);

        writeln!(
            f,
            " --> line {}, column {} to line {}, column {}",
            display_span.start_line,
            display_span.start_col,
            display_span.end_line,
            display_span.end_col
        )?;

        let line_number_width = (display_span.end_line as f64).log10() as usize + 1;

        for (i, line) in self.snippet_lines.iter().enumerate() {
            let current_line = display_span.start_line + i;
            writeln!(
                f,
                "{:>width$} | {}",
                current_line,
                line,
                width = line_number_width
            )?;

            let in_span =
                self.span.start_line <= current_line && current_line <= self.span.end_line;

            if in_span {
                let highlight = if self.span.start_line == self.span.end_line {
                    " ".repeat(self.span.start_col.saturating_sub(1))
                        + &"^".repeat(self.span.end_col.saturating_sub(self.span.start_col))
                } else if current_line == self.span.start_line {
                    " ".repeat(self.span.start_col.saturating_sub(1))
                        + &"^".repeat(line.len().saturating_sub(self.span.start_col - 1))
                } else if current_line == self.span.end_line {
                    "^".repeat(self.span.end_col.saturating_sub(1))
                } else {
                    "^".repeat(line.len())
                };

                writeln!(
                    f,
                    "{:>width$} | {}",
                    "",
                    highlight,
                    width = line_number_width
                )?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number {
        value: i64,
        span: SourceSpan,
    },
    Str {
        value: String,
        span: SourceSpan,
    },
    Bool {
        value: bool,
        span: SourceSpan,
    },
    Var {
        name: String,
        span: SourceSpan,
    },
    Unary {
        op: UnaryOp,
        rhs: Box<Expr>,
        span: SourceSpan,
    },
    Binary {
        lhs: Box<Expr>,
        op: BinOp,
        rhs: Box<Expr>,
        span: SourceSpan,
    },
    Call {
        name: String,
        args: Vec<Expr>,
        span: SourceSpan,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        expr: Expr,
        span: SourceSpan,
    },
    Assign {
        name: String,
        expr: Expr,
        span: SourceSpan,
    },
    Expr {
        expr: Expr,
        span: SourceSpan,
    },
    Block {
        stmts: Vec<Stmt>,
        span: SourceSpan,
    },
    If {
        condition: Expr,
        then_block: Box<Stmt>,         // Block
        else_block: Option<Box<Stmt>>, // Block
        span: SourceSpan,
    },
    While {
        condition: Expr,
        body: Box<Stmt>, // Block
        span: SourceSpan,
    },
    Error {
        span: SourceSpan,
    },
}

#[derive(Debug, Default, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
    pub errors: Vec<ScriptError>,
}
