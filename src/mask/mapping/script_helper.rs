use std::{
    collections::HashMap,
    fmt,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard},
};

use bevy::{
    ecs::{
        resource::Resource,
        system::{Res, ResMut},
    },
    math::Vec2,
    state::state::{NextState, State},
};
use pest::error::LineColLocation;
use pest::iterators::Pair;
use pest::{Parser, Span};
use pest_derive::Parser;
use rust_i18n::t;
use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::{broadcast, oneshot};

use crate::mask::mapping::utils::{
    ControlMsgHelper, SingleSwipeStrategy, build_single_segment_swipe_intermediate_points,
};
use crate::mask::mapping::{
    MappingState,
    cast_spell::{ActiveCastSpell, cancel_active_cast_with_completion, release_active_cast},
    config::{ActiveMappingConfig, BindMappingType},
    cursor::{ActiveCursorFpsConfig, CursorPosition, CursorState},
    direction_pad::BlockDirectionPad,
    fire::{
        ActiveFireMap, enter_fps_mode, exit_fps_mode, spawn_fire_after_hooks_for_external_release,
    },
    raw_input::{enter_raw_input_mode, exit_raw_input_mode},
};
use crate::mask::mask_command::MaskSize;
use crate::scrcpy::constant::{KeyEventAction, Keycode, MetaState, MotionEventAction};
use crate::scrcpy::control_msg::ScrcpyControlMsg;
use crate::tokio_tasks::TokioTasksRuntime;
use crate::utils::ChannelSenderCS;

#[derive(Parser)]
#[grammar = "src/mask/mapping/script.pest"]
struct ScriptParser;

pub enum ScriptRuntimeCommand {
    EnterFps {
        id: String,
        ack: oneshot::Sender<Result<(), String>>,
    },
    ExitFps {
        ack: oneshot::Sender<Result<(), String>>,
    },
    EnterRawInput {
        ack: oneshot::Sender<Result<(), String>>,
    },
    ExitRawInput {
        ack: oneshot::Sender<Result<(), String>>,
    },
    CancelCast {
        id: String,
        ack: oneshot::Sender<Result<(), String>>,
    },
    ReleaseCast {
        ack: oneshot::Sender<Result<(), String>>,
    },
}

#[derive(Resource, Clone)]
pub struct ScriptRuntimeCommandSender(pub crossbeam_channel::Sender<ScriptRuntimeCommand>);

#[derive(Resource)]
pub struct ScriptRuntimeCommandReceiver(pub crossbeam_channel::Receiver<ScriptRuntimeCommand>);

type ScriptStateMap = HashMap<String, HashMap<String, Value>>;

#[derive(Resource, Clone, Default)]
pub struct ScriptSharedState(Arc<Mutex<ScriptStateMap>>);

struct ScriptFuncContext<'a> {
    cs_tx: &'a broadcast::Sender<ScrcpyControlMsg>,
    runtime_command_tx: &'a crossbeam_channel::Sender<ScriptRuntimeCommand>,
    shared_state: ScriptSharedState,
    state_scope: String,
    original_size: Vec2,
}

enum ScriptAction {
    Print {
        output: String,
    },
    Wait {
        ms: u64,
    },
    Touch {
        pointer_id: u64,
        action: MotionEventAction,
        position: Vec2,
        tap_default: bool,
    },
    Swipe {
        pointer_id: u64,
        interval: u64,
        points: Vec<Vec2>,
    },
    Key {
        keycode: Keycode,
        action: KeyEventAction,
        metastate: MetaState,
        key_default: bool,
    },
    PasteText {
        text: String,
    },
}

type EvalFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

async fn execute_script_action(
    _source: &str,
    _span: &SourceSpan,
    ctx: &ScriptFuncContext<'_>,
    action: ScriptAction,
) -> Result<Value, ScriptError> {
    match action {
        ScriptAction::Print { output } => {
            log::info!("{}", output);
        }
        ScriptAction::Wait { ms } => {
            tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
        }
        ScriptAction::Touch {
            pointer_id,
            action,
            position,
            tap_default,
        } => {
            ControlMsgHelper::send_touch(
                ctx.cs_tx,
                action,
                pointer_id,
                ctx.original_size,
                position,
            );

            if tap_default {
                tokio::time::sleep(std::time::Duration::from_millis(30)).await;
                ControlMsgHelper::send_touch(
                    ctx.cs_tx,
                    MotionEventAction::Up,
                    pointer_id,
                    ctx.original_size,
                    position,
                );
            }
        }
        ScriptAction::Swipe {
            pointer_id,
            interval,
            points,
        } => {
            let mut cur_pos = points[0];
            ControlMsgHelper::send_touch(
                ctx.cs_tx,
                MotionEventAction::Down,
                pointer_id,
                ctx.original_size,
                cur_pos,
            );
            for next_pos in points.into_iter().skip(1) {
                let steps = build_single_segment_swipe_intermediate_points(
                    cur_pos,
                    next_pos,
                    SingleSwipeStrategy::Linear,
                    interval,
                );
                for step in steps {
                    ControlMsgHelper::send_touch(
                        ctx.cs_tx,
                        MotionEventAction::Move,
                        pointer_id,
                        ctx.original_size,
                        step.pos,
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(step.wait_ms)).await;
                }
                cur_pos = next_pos;
            }
            ControlMsgHelper::send_touch(
                ctx.cs_tx,
                MotionEventAction::Up,
                pointer_id,
                ctx.original_size,
                cur_pos,
            );
        }
        ScriptAction::Key {
            keycode,
            action,
            metastate,
            key_default,
        } => {
            if key_default {
                ctx.cs_tx
                    .send(ScrcpyControlMsg::InjectKeycode {
                        action: KeyEventAction::Down,
                        keycode: keycode.clone(),
                        repeat: 0,
                        metastate: metastate.clone(),
                    })
                    .unwrap();
            }

            ctx.cs_tx
                .send(ScrcpyControlMsg::InjectKeycode {
                    action,
                    keycode,
                    repeat: 0,
                    metastate,
                })
                .unwrap();
        }
        ScriptAction::PasteText { text } => {
            let sequence = rand::random::<u64>();
            ctx.cs_tx
                .send(ScrcpyControlMsg::SetClipboard {
                    sequence,
                    paste: true,
                    text,
                })
                .unwrap();
        }
    }

    Ok(Value::Int(0))
}

async fn execute_runtime_command(
    source: &str,
    span: &SourceSpan,
    ctx: &ScriptFuncContext<'_>,
    build_command: impl FnOnce(oneshot::Sender<Result<(), String>>) -> ScriptRuntimeCommand,
) -> Result<Value, ScriptError> {
    let (ack_tx, ack_rx) = oneshot::channel();
    ctx.runtime_command_tx
        .send(build_command(ack_tx))
        .map_err(|e| {
            ScriptError::from_span(
                span.clone(),
                source,
                format!("Failed to send script runtime command: {e}"),
            )
        })?;

    match ack_rx.await {
        Ok(Ok(())) => Ok(Value::Int(0)),
        Ok(Err(e)) => Err(ScriptError::from_span(span.clone(), source, e)),
        Err(e) => Err(ScriptError::from_span(
            span.clone(),
            source,
            format!("Failed to receive script runtime command acknowledgement: {e}"),
        )),
    }
}

fn ack_immediately(ack: oneshot::Sender<Result<(), String>>, result: Result<(), String>) {
    let _ = ack.send(result);
}

fn ack_after_next_update(
    runtime: &TokioTasksRuntime,
    ack: oneshot::Sender<Result<(), String>>,
    result: Result<(), String>,
) {
    runtime.spawn_background_task(move |mut ctx| async move {
        ctx.sleep_updates(1).await;
        let _ = ack.send(result);
    });
}

pub fn handle_script_runtime_commands(
    command_rx: Res<ScriptRuntimeCommandReceiver>,
    script_command_tx: Res<ScriptRuntimeCommandSender>,
    shared_state: Res<ScriptSharedState>,
    active_mapping: Res<ActiveMappingConfig>,
    cs_tx_res: Res<ChannelSenderCS>,
    mut fps_config: ResMut<ActiveCursorFpsConfig>,
    mapping_state: Res<State<MappingState>>,
    cursor_state: Res<State<CursorState>>,
    mut next_cursor_state: ResMut<NextState<CursorState>>,
    cursor_pos: Res<CursorPosition>,
    mask_size: Res<MaskSize>,
    mut active_fire_map: ResMut<ActiveFireMap>,
    mut next_mapping_state: ResMut<NextState<MappingState>>,
    runtime: ResMut<TokioTasksRuntime>,
    mut active_cast: ResMut<ActiveCastSpell>,
    mut block_direction_pad: ResMut<BlockDirectionPad>,
) {
    for command in command_rx.0.try_iter() {
        match command {
            ScriptRuntimeCommand::EnterFps { id, ack } => {
                let Some(active_mapping) = &active_mapping.0 else {
                    let message = "[Script] enter_fps failed: no active mapping config".to_string();
                    log::error!("{}", message);
                    ack_immediately(ack, Err(message));
                    continue;
                };
                let Some(action) = active_mapping.mapping_id_actions.get(&id) else {
                    let message = format!("[Script] enter_fps failed: mapping id not found: {id}");
                    log::error!("{}", message);
                    ack_immediately(ack, Err(message));
                    continue;
                };
                let Some(BindMappingType::Fps(mapping)) = active_mapping.mappings.get(action)
                else {
                    let message = format!("[Script] enter_fps failed: mapping id is not Fps: {id}");
                    log::error!("{}", message);
                    ack_immediately(ack, Err(message));
                    continue;
                };

                if cursor_state.get() == &CursorState::Fps {
                    let released_fire_actions = exit_fps_mode(
                        &cs_tx_res.0,
                        &mut fps_config,
                        &mut active_fire_map,
                        &mut next_cursor_state,
                        mask_size.0,
                        cursor_pos.0,
                    );
                    spawn_fire_after_hooks_for_external_release(
                        released_fire_actions,
                        active_mapping,
                        &cs_tx_res,
                        &script_command_tx,
                        &shared_state,
                        &runtime,
                        cursor_pos.0,
                        mask_size.0,
                        mapping_state.get() == &MappingState::RawInput,
                        true,
                    );
                }
                enter_fps_mode(
                    &cs_tx_res.0,
                    &mut fps_config,
                    &mut next_cursor_state,
                    mapping,
                    active_mapping.original_size.into(),
                );
                ack_after_next_update(&runtime, ack, Ok(()));
            }
            ScriptRuntimeCommand::ExitFps { ack } => {
                if cursor_state.get() == &CursorState::Fps {
                    let released_fire_actions = exit_fps_mode(
                        &cs_tx_res.0,
                        &mut fps_config,
                        &mut active_fire_map,
                        &mut next_cursor_state,
                        mask_size.0,
                        cursor_pos.0,
                    );
                    if let Some(active_mapping) = &active_mapping.0 {
                        spawn_fire_after_hooks_for_external_release(
                            released_fire_actions,
                            active_mapping,
                            &cs_tx_res,
                            &script_command_tx,
                            &shared_state,
                            &runtime,
                            cursor_pos.0,
                            mask_size.0,
                            mapping_state.get() == &MappingState::RawInput,
                            true,
                        );
                    }
                }
                ack_after_next_update(&runtime, ack, Ok(()));
            }
            ScriptRuntimeCommand::EnterRawInput { ack } => {
                if cursor_state.get() == &CursorState::Fps {
                    let message =
                        "[Script] enter_raw_input ignored while cursor is in FPS mode".to_string();
                    log::error!("{}", message);
                    ack_immediately(ack, Err(message));
                    continue;
                }
                enter_raw_input_mode(&mut next_mapping_state);
                ack_after_next_update(&runtime, ack, Ok(()));
            }
            ScriptRuntimeCommand::ExitRawInput { ack } => {
                exit_raw_input_mode(&mut next_mapping_state);
                ack_after_next_update(&runtime, ack, Ok(()));
            }
            ScriptRuntimeCommand::CancelCast { id, ack } => {
                let Some(active_mapping) = &active_mapping.0 else {
                    let message =
                        "[Script] cancel_cast failed: no active mapping config".to_string();
                    log::error!("{}", message);
                    ack_immediately(ack, Err(message));
                    continue;
                };
                let Some(action) = active_mapping.mapping_id_actions.get(&id) else {
                    let message =
                        format!("[Script] cancel_cast failed: mapping id not found: {id}");
                    log::error!("{}", message);
                    ack_immediately(ack, Err(message));
                    continue;
                };
                let Some(BindMappingType::CancelCast(mapping)) =
                    active_mapping.mappings.get(action)
                else {
                    let message =
                        format!("[Script] cancel_cast failed: mapping id is not CancelCast: {id}");
                    log::error!("{}", message);
                    ack_immediately(ack, Err(message));
                    continue;
                };
                cancel_active_cast_with_completion(
                    &cs_tx_res.0,
                    &runtime,
                    &mut active_cast,
                    mapping,
                    active_mapping.original_size.into(),
                    mask_size.0,
                    &script_command_tx,
                    &shared_state,
                    cursor_pos.0,
                    mapping_state.get() == &MappingState::RawInput,
                    cursor_state.get() == &CursorState::Fps,
                    Some(ack),
                );
            }
            ScriptRuntimeCommand::ReleaseCast { ack } => {
                release_active_cast(
                    &cs_tx_res.0,
                    &runtime,
                    mask_size.0,
                    cursor_pos.0,
                    &mut active_cast,
                    &mut block_direction_pad,
                    &script_command_tx,
                    &shared_state,
                    mapping_state.get() == &MappingState::RawInput,
                    cursor_state.get() == &CursorState::Fps,
                );
                ack_immediately(ack, Ok(()));
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Bool(bool),
    Str(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticType {
    Int,
    Bool,
    Str,
    Unknown,
}

impl StaticType {
    fn name(self) -> &'static str {
        match self {
            StaticType::Int => "integer",
            StaticType::Bool => "boolean",
            StaticType::Str => "string",
            StaticType::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone)]
struct ExprInfo {
    ty: StaticType,
    const_value: Option<Value>,
}

impl ExprInfo {
    fn new(ty: StaticType) -> Self {
        Self {
            ty,
            const_value: None,
        }
    }

    fn constant(value: Value) -> Self {
        Self {
            ty: static_type_of_value(&value),
            const_value: Some(value),
        }
    }
}

fn static_type_of_value(value: &Value) -> StaticType {
    match value {
        Value::Int(_) => StaticType::Int,
        Value::Bool(_) => StaticType::Bool,
        Value::Str(_) => StaticType::Str,
    }
}

fn merge_static_type(lhs: StaticType, rhs: StaticType) -> StaticType {
    if lhs == rhs { lhs } else { StaticType::Unknown }
}

fn merge_variable_maps(
    lhs: &HashMap<String, StaticType>,
    rhs: &HashMap<String, StaticType>,
) -> HashMap<String, StaticType> {
    lhs.iter()
        .filter_map(|(name, lhs_ty)| {
            rhs.get(name)
                .map(|rhs_ty| (name.clone(), merge_static_type(*lhs_ty, *rhs_ty)))
        })
        .collect()
}

fn builtin_script_vars() -> HashMap<String, StaticType> {
    HashMap::from([
        ("ORIGINAL_W".to_string(), StaticType::Int),
        ("ORIGINAL_H".to_string(), StaticType::Int),
        ("CURSOR_X".to_string(), StaticType::Int),
        ("CURSOR_Y".to_string(), StaticType::Int),
        ("RawInputFlag".to_string(), StaticType::Bool),
        ("FpsModeFlag".to_string(), StaticType::Bool),
    ])
}

#[derive(Default, Debug, Clone)]
pub struct ScriptAST {
    pub program: Program,
    pub script: String,
    pub empty: bool,
}

fn normalize_script_semicolons(script: &str) -> String {
    let mut normalized = String::with_capacity(script.len());
    let mut chars = script.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    let mut in_line_comment = false;
    let mut paren_depth = 0usize;
    let mut needs_semicolon = false;

    while let Some(ch) = chars.next() {
        if in_line_comment {
            normalized.push(ch);
            if matches!(ch, '\n' | '\r') {
                in_line_comment = false;
            }
            continue;
        }

        if in_string {
            normalized.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
                needs_semicolon = true;
            }
            continue;
        }

        match ch {
            '"' => {
                normalized.push(ch);
                in_string = true;
                escaped = false;
                needs_semicolon = false;
            }
            '/' if matches!(chars.peek(), Some('/')) => {
                let mut lookahead = chars.clone();
                lookahead.next();
                if paren_depth == 0
                    && needs_semicolon
                    && next_significant_char_after_line_comment(lookahead) != Some('{')
                {
                    normalized.push(';');
                    needs_semicolon = false;
                }
                normalized.push(ch);
                if let Some(next) = chars.next() {
                    normalized.push(next);
                }
                in_line_comment = true;
            }
            '\n' | '\r' => {
                if paren_depth == 0
                    && needs_semicolon
                    && next_significant_char(chars.clone()) != Some('{')
                {
                    normalized.push(';');
                    needs_semicolon = false;
                }
                normalized.push(ch);
            }
            '}' => {
                if paren_depth == 0 && needs_semicolon {
                    normalized.push(';');
                }
                normalized.push(ch);
                needs_semicolon = false;
            }
            '(' => {
                paren_depth += 1;
                normalized.push(ch);
                needs_semicolon = false;
            }
            ')' => {
                paren_depth = paren_depth.saturating_sub(1);
                normalized.push(ch);
                needs_semicolon = true;
            }
            ';' | '{' | ',' | '+' | '-' | '*' | '/' | '%' | '!' | '=' | '<' | '>' | '&' | '|' => {
                normalized.push(ch);
                needs_semicolon = false;
            }
            c if c.is_whitespace() => {
                normalized.push(c);
            }
            _ => {
                normalized.push(ch);
                needs_semicolon = true;
            }
        }
    }

    if !in_string && !in_line_comment && paren_depth == 0 && needs_semicolon {
        normalized.push(';');
    }

    normalized
}

fn next_significant_char(mut chars: std::iter::Peekable<std::str::Chars<'_>>) -> Option<char> {
    while let Some(ch) = chars.next() {
        if ch.is_whitespace() {
            continue;
        }

        if ch == '/' && matches!(chars.peek(), Some('/')) {
            chars.next();
            return next_significant_char_after_line_comment(chars);
        }

        return Some(ch);
    }

    None
}

fn next_significant_char_after_line_comment(
    mut chars: std::iter::Peekable<std::str::Chars<'_>>,
) -> Option<char> {
    for ch in chars.by_ref() {
        if matches!(ch, '\n' | '\r') {
            break;
        }
    }

    next_significant_char(chars)
}

impl ScriptAST {
    pub fn new(script: &str) -> Result<Self, String> {
        let normalized_script = normalize_script_semicolons(script);
        let program_pair = ScriptParser::parse(Rule::program, &normalized_script)
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
        ast.program.errors.extend(ast.validate_program());
        if !ast.program.errors.is_empty() {
            return Err(ast
                .program
                .errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n\n"));
        }

        Ok(ast)
    }

    pub fn validate_source(script: &str) -> Result<(), String> {
        Self::new(script).map(|_| ())
    }

    pub fn validate_diagnostics(script: &str) -> Vec<ScriptDiagnostic> {
        let normalized_script = normalize_script_semicolons(script);
        let program_pair = match ScriptParser::parse(Rule::program, &normalized_script) {
            Ok(mut pairs) => match pairs.next() {
                Some(pair) => pair,
                None => {
                    return vec![ScriptDiagnostic::error(
                        t!("mask.mapping.noProgramFound").to_string(),
                        SourceSpan::point_at_start(),
                    )];
                }
            },
            Err(err) => {
                return vec![ScriptDiagnostic::from_pest_error(err)];
            }
        };

        let mut ast = ScriptAST::default();
        if script.is_empty() {
            return Vec::new();
        }

        ast.script = script.to_string();
        ast.program = ast.parse_program(program_pair);
        ast.program.errors.extend(ast.validate_program());
        ast.program
            .errors
            .iter()
            .map(ScriptDiagnostic::from)
            .collect()
    }

    fn validate_program(&self) -> Vec<ScriptError> {
        let mut analyzer = ScriptAnalyzer::new(&self.script);
        analyzer.analyze_program(&self.program);
        analyzer.errors
    }

    pub async fn run_script(
        &self,
        cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
        script_command_tx: &crossbeam_channel::Sender<ScriptRuntimeCommand>,
        shared_state: &ScriptSharedState,
        state_scope: &str,
        original_size: Vec2,
        cursor_pos: Vec2,
        mask_size: Vec2,
        raw_input_flag: bool,
        fps_mode_flag: bool,
    ) -> Result<(), ScriptError> {
        if self.empty {
            return Ok(());
        }
        let cursor_relative_pos = cursor_pos / mask_size * original_size;
        let mut vars: HashMap<String, Value> = HashMap::new();
        vars.insert(
            "ORIGINAL_W".to_string(),
            Value::Int((original_size.x) as i64),
        );
        vars.insert("ORIGINAL_H".to_string(), Value::Int(original_size.y as i64));
        vars.insert(
            "CURSOR_X".to_string(),
            Value::Int(cursor_relative_pos.x as i64),
        );
        vars.insert(
            "CURSOR_Y".to_string(),
            Value::Int(cursor_relative_pos.y as i64),
        );
        vars.insert("RawInputFlag".to_string(), Value::Bool(raw_input_flag));
        vars.insert("FpsModeFlag".to_string(), Value::Bool(fps_mode_flag));

        let script_func_ctx = ScriptFuncContext {
            cs_tx,
            runtime_command_tx: script_command_tx,
            shared_state: shared_state.clone(),
            state_scope: state_scope.to_string(),
            original_size,
        };

        for stmt in self.program.stmts.iter() {
            self.eval_stmt(stmt, &mut vars, &script_func_ctx).await?;
        }

        Ok(())
    }

    fn eval_stmt<'a>(
        &'a self,
        stmt: &'a Stmt,
        vars: &'a mut HashMap<String, Value>,
        ctx: &'a ScriptFuncContext<'a>,
    ) -> EvalFuture<'a, Result<(), ScriptError>> {
        Box::pin(async move {
            match stmt {
                Stmt::Let { name, expr, span } => {
                    let val = self
                        .eval_expr(expr, vars, ctx)
                        .await
                        .map_err(|e| e.with_outer_span(span.clone(), &self.script))?;
                    vars.insert(name.clone(), val);
                    Ok(())
                }
                Stmt::Assign { name, expr, span } => {
                    let val = self
                        .eval_expr(expr, vars, ctx)
                        .await
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
                Stmt::Expr { expr, span } => match self.eval_expr(expr, vars, ctx).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.with_outer_span(span.clone(), &self.script)),
                },
                Stmt::Block { stmts, .. } => {
                    for stmt in stmts {
                        self.eval_stmt(stmt, vars, ctx).await?;
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
                        .eval_expr(condition, vars, ctx)
                        .await
                        .map_err(|e| e.with_outer_span(span.clone(), &self.script))?;

                    if Self::is_truthy(&cond_val) {
                        self.eval_stmt(then_block, vars, ctx).await?;
                    } else if let Some(else_stmt) = else_block {
                        self.eval_stmt(else_stmt.as_ref(), vars, ctx).await?;
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
                            .eval_expr(condition, vars, ctx)
                            .await
                            .map_err(|e| e.with_outer_span(span.clone(), &self.script))?;
                        Self::is_truthy(&cond_val)
                    } {
                        self.eval_stmt(body, vars, ctx).await?;
                    }
                    Ok(())
                }
                Stmt::Empty { .. } => Ok(()),
                Stmt::Error { .. } => unreachable!("Error statement reached"),
            }
        })
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

    fn eval_expr<'a>(
        &'a self,
        expr: &'a Expr,
        vars: &'a mut HashMap<String, Value>,
        ctx: &'a ScriptFuncContext<'a>,
    ) -> EvalFuture<'a, Result<Value, ScriptError>> {
        Box::pin(async move {
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
                    let mut arg_values = Vec::new();
                    for arg in args {
                        arg_values.push(self.eval_expr(arg, vars, ctx).await?);
                    }
                    self.call_func(ctx, &self.script, span, name, &arg_values)
                        .await
                        .map_err(|e| e.with_outer_span(span.clone(), &self.script))
                }
                Expr::Unary { op, rhs, span } => {
                    let rhs_val = self.eval_expr(rhs, vars, ctx).await?;
                    match op {
                        UnaryOp::Plus => {
                            if Self::is_numeric_value(&rhs_val) {
                                Ok(Value::Int(Self::to_int_value(&rhs_val)))
                            } else {
                                Err(ScriptError::from_span(
                                    span.clone(),
                                    &self.script,
                                    format!(
                                        "Unary plus operator only supports integers or booleans"
                                    ),
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
                                    format!(
                                        "Unary minus operator only supports integers or booleans"
                                    ),
                                ))
                            }
                        }
                        UnaryOp::Not => Ok(Value::Bool(!Self::is_truthy(&rhs_val))),
                    }
                }
                Expr::Binary { lhs, op, rhs, span } => {
                    let lhs_val = self.eval_expr(lhs, vars, ctx).await?;
                    let rhs_val = self.eval_expr(rhs, vars, ctx).await?;

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
        })
    }

    async fn call_func(
        &self,
        ctx: &ScriptFuncContext<'_>,
        source: &str,
        span: &SourceSpan,
        name: &str,
        args: &[Value],
    ) -> Result<Value, ScriptError> {
        match name {
            "print" => print_func(ctx, source, span, args).await,
            "wait" => wait_func(ctx, source, span, args).await,
            "tap" => tap_func(ctx, source, span, args).await,
            "swipe" => swipe_func(ctx, source, span, args).await,
            "send_key" => send_key_func(ctx, source, span, args).await,
            "paste_text" => paste_text_func(ctx, source, span, args).await,
            "state_set" => state_set_func(ctx, source, span, args).await,
            "state_get" => state_get_func(ctx, source, span, args).await,
            "state_has" => state_has_func(ctx, source, span, args).await,
            "state_delete" => state_delete_func(ctx, source, span, args).await,
            "state_clear" => state_clear_func(ctx, source, span, args).await,
            "enter_fps" => enter_fps_func(ctx, source, span, args).await,
            "exit_fps" => exit_fps_func(ctx, source, span, args).await,
            "enter_raw_input" => enter_raw_input_func(ctx, source, span, args).await,
            "exit_raw_input" => exit_raw_input_func(ctx, source, span, args).await,
            "cancel_cast" => cancel_cast_func(ctx, source, span, args).await,
            "release_cast" => release_cast_func(ctx, source, span, args).await,
            _ => Err(ScriptError::from_span(
                span.clone(),
                source,
                format!("Function '{}' not defined", name),
            )),
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
        let core = it.next().unwrap();

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
            Rule::empty_stmt => Stmt::Empty { span },
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

struct ScriptAnalyzer<'a> {
    source: &'a str,
    errors: Vec<ScriptError>,
}

impl<'a> ScriptAnalyzer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            errors: Vec::new(),
        }
    }

    fn analyze_program(&mut self, program: &Program) {
        let mut vars = builtin_script_vars();
        for stmt in &program.stmts {
            self.analyze_stmt(stmt, &mut vars);
        }
    }

    fn analyze_stmt(&mut self, stmt: &Stmt, vars: &mut HashMap<String, StaticType>) {
        match stmt {
            Stmt::Let { name, expr, .. } => {
                let info = self.analyze_expr(expr, vars);
                vars.insert(name.clone(), info.ty);
            }
            Stmt::Assign { name, expr, span } => {
                let info = self.analyze_expr(expr, vars);
                if vars.contains_key(name) {
                    vars.insert(name.clone(), info.ty);
                } else {
                    self.error(
                        *span,
                        format!("Variable '{}' is assigned before it is defined", name),
                    );
                }
            }
            Stmt::Expr { expr, .. } => {
                self.analyze_expr(expr, vars);
            }
            Stmt::Block { stmts, .. } => {
                for stmt in stmts {
                    self.analyze_stmt(stmt, vars);
                }
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                let condition_info = self.analyze_expr(condition, vars);

                let mut then_vars = vars.clone();
                self.analyze_stmt(then_block, &mut then_vars);

                let mut else_vars = vars.clone();
                if let Some(else_stmt) = else_block {
                    self.analyze_stmt(else_stmt, &mut else_vars);
                }

                *vars = match condition_info.const_value {
                    Some(value) if Self::const_truthy(&value) => then_vars,
                    Some(_) => else_vars,
                    None => merge_variable_maps(&then_vars, &else_vars),
                };
            }
            Stmt::While {
                condition,
                body,
                span,
            } => {
                let condition_info = self.analyze_expr(condition, vars);
                if condition_info
                    .const_value
                    .as_ref()
                    .is_some_and(Self::const_truthy)
                {
                    self.error(
                        *span,
                        "while condition is always truthy and may block script execution",
                    );
                }

                let before_loop = vars.clone();
                let mut body_vars = vars.clone();
                self.analyze_stmt(body, &mut body_vars);
                *vars = merge_variable_maps(&before_loop, &body_vars);
            }
            Stmt::Empty { .. } | Stmt::Error { .. } => {}
        }
    }

    fn analyze_expr(&mut self, expr: &Expr, vars: &HashMap<String, StaticType>) -> ExprInfo {
        match expr {
            Expr::Number { value, .. } => ExprInfo::constant(Value::Int(*value)),
            Expr::Bool { value, .. } => ExprInfo::constant(Value::Bool(*value)),
            Expr::Str { value, .. } => ExprInfo::constant(Value::Str(value.clone())),
            Expr::Var { name, span } => {
                if let Some(ty) = vars.get(name) {
                    ExprInfo::new(*ty)
                } else {
                    self.error(*span, format!("Variable '{}' not defined", name));
                    ExprInfo::new(StaticType::Unknown)
                }
            }
            Expr::Call { name, args, span } => {
                let args = args
                    .iter()
                    .map(|arg| self.analyze_expr(arg, vars))
                    .collect::<Vec<_>>();
                self.analyze_call(name, &args, *span)
            }
            Expr::Unary { op, rhs, span } => {
                let rhs = self.analyze_expr(rhs, vars);
                match op {
                    UnaryOp::Plus | UnaryOp::Minus => {
                        self.expect_numeric(rhs.ty, *span, "Unary plus/minus operator");
                        let const_value = rhs.const_value.as_ref().and_then(|value| {
                            Self::const_int(value).map(|n| {
                                if matches!(op, UnaryOp::Minus) {
                                    Value::Int(-n)
                                } else {
                                    Value::Int(n)
                                }
                            })
                        });
                        ExprInfo {
                            ty: if rhs.ty == StaticType::Unknown {
                                StaticType::Unknown
                            } else {
                                StaticType::Int
                            },
                            const_value,
                        }
                    }
                    UnaryOp::Not => ExprInfo {
                        ty: StaticType::Bool,
                        const_value: rhs
                            .const_value
                            .as_ref()
                            .map(|value| Value::Bool(!Self::const_truthy(value))),
                    },
                }
            }
            Expr::Binary { lhs, op, rhs, span } => {
                let lhs = self.analyze_expr(lhs, vars);
                let rhs = self.analyze_expr(rhs, vars);
                self.analyze_binary(&lhs, *op, &rhs, *span)
            }
        }
    }

    fn analyze_binary(
        &mut self,
        lhs: &ExprInfo,
        op: BinOp,
        rhs: &ExprInfo,
        span: SourceSpan,
    ) -> ExprInfo {
        match op {
            BinOp::Add => {
                if lhs.ty == StaticType::Str && rhs.ty == StaticType::Str {
                    return ExprInfo {
                        ty: StaticType::Str,
                        const_value: match (&lhs.const_value, &rhs.const_value) {
                            (Some(Value::Str(l)), Some(Value::Str(r))) => {
                                Some(Value::Str(format!("{l}{r}")))
                            }
                            _ => None,
                        },
                    };
                }

                if self.expect_numeric_pair(lhs.ty, rhs.ty, span, "Addition") {
                    return ExprInfo {
                        ty: StaticType::Int,
                        const_value: Self::const_int_pair(lhs, rhs).map(|(l, r)| Value::Int(l + r)),
                    };
                }

                ExprInfo::new(StaticType::Unknown)
            }
            BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                if !self.expect_numeric_pair(lhs.ty, rhs.ty, span, "Arithmetic operator") {
                    return ExprInfo::new(StaticType::Unknown);
                }

                let const_value = Self::const_int_pair(lhs, rhs).and_then(|(l, r)| match op {
                    BinOp::Sub => Some(Value::Int(l - r)),
                    BinOp::Mul => Some(Value::Int(l * r)),
                    BinOp::Div => {
                        if r == 0 {
                            self.error(span, "Division by zero");
                            None
                        } else {
                            Some(Value::Int(l / r))
                        }
                    }
                    BinOp::Mod => {
                        if r == 0 {
                            self.error(span, "Modulo by zero");
                            None
                        } else {
                            Some(Value::Int(l % r))
                        }
                    }
                    _ => None,
                });

                ExprInfo {
                    ty: StaticType::Int,
                    const_value,
                }
            }
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                if !Self::types_are_comparable(lhs.ty, rhs.ty) {
                    self.error(
                        span,
                        format!(
                            "Comparison is not supported between {} and {}",
                            lhs.ty.name(),
                            rhs.ty.name()
                        ),
                    );
                }
                ExprInfo {
                    ty: StaticType::Bool,
                    const_value: None,
                }
            }
            BinOp::Eq | BinOp::Neq => ExprInfo {
                ty: StaticType::Bool,
                const_value: None,
            },
            BinOp::And | BinOp::Or => ExprInfo {
                ty: StaticType::Bool,
                const_value: match (&lhs.const_value, &rhs.const_value) {
                    (Some(l), Some(r)) => {
                        let l = Self::const_truthy(l);
                        let r = Self::const_truthy(r);
                        Some(Value::Bool(if op == BinOp::And { l && r } else { l || r }))
                    }
                    _ => None,
                },
            },
        }
    }

    fn analyze_call(&mut self, name: &str, args: &[ExprInfo], span: SourceSpan) -> ExprInfo {
        match name {
            "print" => ExprInfo::new(StaticType::Int),
            "wait" => {
                self.expect_arity(name, args.len(), 1, Some(1), span);
                self.expect_type(args, 0, StaticType::Int, name, span);
                self.expect_non_negative_int(args, 0, name, span);
                ExprInfo::new(StaticType::Int)
            }
            "tap" => {
                self.expect_arity(name, args.len(), 3, Some(4), span);
                for index in 0..3 {
                    self.expect_type(args, index, StaticType::Int, name, span);
                }
                self.expect_non_negative_int(args, 0, name, span);
                if args.len() >= 4 {
                    self.expect_type(args, 3, StaticType::Str, name, span);
                    self.expect_string_one_of(
                        args,
                        3,
                        &["default", "down", "up", "move"],
                        name,
                        span,
                    );
                }
                ExprInfo::new(StaticType::Int)
            }
            "swipe" => {
                if args.len() < 6 || args.len() % 2 != 0 {
                    self.error(
                        span,
                        "The swipe function takes at least 6 arguments and an even number of arguments",
                    );
                }
                for index in 0..args.len() {
                    self.expect_type(args, index, StaticType::Int, name, span);
                }
                self.expect_non_negative_int(args, 0, name, span);
                self.expect_non_negative_int(args, 1, name, span);
                ExprInfo::new(StaticType::Int)
            }
            "send_key" => {
                self.expect_arity(name, args.len(), 1, Some(3), span);
                self.expect_type(args, 0, StaticType::Str, name, span);
                if let Some(Value::Str(key_name)) =
                    args.get(0).and_then(|arg| arg.const_value.as_ref())
                {
                    if !json_enum_value_is_valid::<Keycode>(key_name) {
                        self.error(span, format!("Invalid key name '{}'", key_name));
                    }
                }
                if args.len() >= 2 {
                    self.expect_type(args, 1, StaticType::Str, name, span);
                    self.expect_string_one_of(args, 1, &["default", "down", "up"], name, span);
                }
                if args.len() >= 3 {
                    self.expect_type(args, 2, StaticType::Str, name, span);
                    if let Some(Value::Str(metastate)) =
                        args.get(2).and_then(|arg| arg.const_value.as_ref())
                    {
                        if !json_enum_value_is_valid::<MetaState>(metastate) {
                            self.error(span, format!("Invalid metastate '{}'", metastate));
                        }
                    }
                }
                ExprInfo::new(StaticType::Int)
            }
            "paste_text" => {
                self.expect_arity(name, args.len(), 1, Some(1), span);
                self.expect_type(args, 0, StaticType::Str, name, span);
                ExprInfo::new(StaticType::Int)
            }
            "state_set" => {
                self.expect_arity(name, args.len(), 2, Some(2), span);
                self.expect_non_empty_string(args, 0, name, span);
                ExprInfo::new(StaticType::Int)
            }
            "state_get" => {
                self.expect_arity(name, args.len(), 2, Some(2), span);
                self.expect_non_empty_string(args, 0, name, span);
                args.get(1)
                    .cloned()
                    .unwrap_or_else(|| ExprInfo::new(StaticType::Unknown))
            }
            "state_has" | "state_delete" => {
                self.expect_arity(name, args.len(), 1, Some(1), span);
                self.expect_non_empty_string(args, 0, name, span);
                ExprInfo::new(StaticType::Bool)
            }
            "state_clear" | "exit_fps" | "enter_raw_input" | "exit_raw_input" | "release_cast" => {
                self.expect_arity(name, args.len(), 0, Some(0), span);
                ExprInfo::new(StaticType::Int)
            }
            "enter_fps" | "cancel_cast" => {
                self.expect_arity(name, args.len(), 1, Some(1), span);
                self.expect_non_empty_string(args, 0, name, span);
                ExprInfo::new(StaticType::Int)
            }
            _ => {
                self.error(span, format!("Function '{}' not defined", name));
                ExprInfo::new(StaticType::Unknown)
            }
        }
    }

    fn expect_arity(
        &mut self,
        name: &str,
        actual: usize,
        min: usize,
        max: Option<usize>,
        span: SourceSpan,
    ) {
        let valid = match max {
            Some(max) => actual >= min && actual <= max,
            None => actual >= min,
        };
        if !valid {
            let expected = match max {
                Some(max) if min == max => min.to_string(),
                Some(max) => format!("{min}-{max}"),
                None => format!("at least {min}"),
            };
            self.error(
                span,
                format!("The {name} function takes {expected} argument(s), got {actual}"),
            );
        }
    }

    fn expect_type(
        &mut self,
        args: &[ExprInfo],
        index: usize,
        expected: StaticType,
        function_name: &str,
        span: SourceSpan,
    ) {
        let Some(arg) = args.get(index) else {
            return;
        };

        if arg.ty != StaticType::Unknown && arg.ty != expected {
            self.error(
                span,
                format!(
                    "Argument {} of {function_name} must be {}, got {}",
                    index + 1,
                    expected.name(),
                    arg.ty.name()
                ),
            );
        }
    }

    fn expect_non_negative_int(
        &mut self,
        args: &[ExprInfo],
        index: usize,
        function_name: &str,
        span: SourceSpan,
    ) {
        if let Some(Value::Int(value)) = args.get(index).and_then(|arg| arg.const_value.as_ref()) {
            if *value < 0 {
                self.error(
                    span,
                    format!(
                        "Argument {} of {function_name} must be a non-negative integer",
                        index + 1
                    ),
                );
            }
        }
    }

    fn expect_non_empty_string(
        &mut self,
        args: &[ExprInfo],
        index: usize,
        function_name: &str,
        span: SourceSpan,
    ) {
        self.expect_type(args, index, StaticType::Str, function_name, span);
        if let Some(Value::Str(value)) = args.get(index).and_then(|arg| arg.const_value.as_ref()) {
            if value.trim().is_empty() {
                self.error(
                    span,
                    format!(
                        "Argument {} of {function_name} must be a non-empty string",
                        index + 1
                    ),
                );
            }
        }
    }

    fn expect_string_one_of(
        &mut self,
        args: &[ExprInfo],
        index: usize,
        allowed: &[&str],
        function_name: &str,
        span: SourceSpan,
    ) {
        if let Some(Value::Str(value)) = args.get(index).and_then(|arg| arg.const_value.as_ref()) {
            if !allowed.contains(&value.as_str()) {
                self.error(
                    span,
                    format!(
                        "Argument {} of {function_name} must be one of: {}",
                        index + 1,
                        allowed.join(", ")
                    ),
                );
            }
        }
    }

    fn expect_numeric(&mut self, ty: StaticType, span: SourceSpan, operation: &str) -> bool {
        if ty == StaticType::Unknown || Self::type_is_numeric(ty) {
            true
        } else {
            self.error(
                span,
                format!("{operation} only supports integers or booleans"),
            );
            false
        }
    }

    fn expect_numeric_pair(
        &mut self,
        lhs: StaticType,
        rhs: StaticType,
        span: SourceSpan,
        operation: &str,
    ) -> bool {
        if lhs == StaticType::Unknown || rhs == StaticType::Unknown {
            return false;
        }

        if Self::type_is_numeric(lhs) && Self::type_is_numeric(rhs) {
            true
        } else {
            self.error(
                span,
                format!(
                    "{operation} is not supported between {} and {}",
                    lhs.name(),
                    rhs.name()
                ),
            );
            false
        }
    }

    fn type_is_numeric(ty: StaticType) -> bool {
        matches!(ty, StaticType::Int | StaticType::Bool)
    }

    fn types_are_comparable(lhs: StaticType, rhs: StaticType) -> bool {
        lhs == StaticType::Unknown
            || rhs == StaticType::Unknown
            || (Self::type_is_numeric(lhs) && Self::type_is_numeric(rhs))
            || (lhs == StaticType::Str && rhs == StaticType::Str)
    }

    fn const_int(value: &Value) -> Option<i64> {
        match value {
            Value::Int(value) => Some(*value),
            Value::Bool(value) => Some(i64::from(*value)),
            Value::Str(_) => None,
        }
    }

    fn const_int_pair(lhs: &ExprInfo, rhs: &ExprInfo) -> Option<(i64, i64)> {
        Some((
            Self::const_int(lhs.const_value.as_ref()?)?,
            Self::const_int(rhs.const_value.as_ref()?)?,
        ))
    }

    fn const_truthy(value: &Value) -> bool {
        match value {
            Value::Int(value) => *value != 0,
            Value::Bool(value) => *value,
            Value::Str(value) => !value.is_empty(),
        }
    }

    fn error(&mut self, span: SourceSpan, message: impl ToString) {
        self.errors
            .push(ScriptError::from_span(span, self.source, message));
    }
}

fn json_enum_value_is_valid<T: DeserializeOwned>(value: &str) -> bool {
    serde_json::from_str::<T>(&format!("\"{}\"", value)).is_ok()
}

async fn print_func(
    ctx: &ScriptFuncContext<'_>,
    source: &str,
    span: &SourceSpan,
    args: &[Value],
) -> Result<Value, ScriptError> {
    let output = args
        .iter()
        .map(|val| match val {
            Value::Int(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Str(s) => s.clone(),
        })
        .collect::<Vec<String>>()
        .join(" ");

    execute_script_action(source, span, ctx, ScriptAction::Print { output }).await
}

async fn wait_func(
    ctx: &ScriptFuncContext<'_>,
    source: &str,
    span: &SourceSpan,
    args: &[Value],
) -> Result<Value, ScriptError> {
    let ms = match args {
        [Value::Int(ms)] if *ms >= 0 => *ms as u64,
        _ => {
            return Err(ScriptError::from_span(
                span.clone(),
                source,
                "The wait function takes one argument: time (non-negative int)".to_string(),
            ));
        }
    };

    execute_script_action(source, span, ctx, ScriptAction::Wait { ms }).await
}

async fn tap_func(
    ctx: &ScriptFuncContext<'_>,
    source: &str,
    span: &SourceSpan,
    args: &[Value],
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

            execute_script_action(
                source,
                span,
                ctx,
                ScriptAction::Touch {
                    pointer_id,
                    action,
                    position: (*x as f32, *y as f32).into(),
                    tap_default: action_str == "default",
                },
            )
            .await
        }
        _ => Err(ScriptError::from_span(
            span.clone(),
            source,
            format_msg.to_string(),
        )),
    }
}

fn expect_no_args_func(
    source: &str,
    span: &SourceSpan,
    args: &[Value],
    function_name: &str,
) -> Result<(), ScriptError> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(ScriptError::from_span(
            span.clone(),
            source,
            format!("The {function_name} function takes no arguments"),
        ))
    }
}

fn expect_id_arg_func<'a>(
    source: &'a str,
    span: &SourceSpan,
    args: &'a [Value],
    function_name: &str,
) -> Result<&'a str, ScriptError> {
    match args {
        [Value::Str(id)] if !id.trim().is_empty() => Ok(id),
        _ => Err(ScriptError::from_span(
            span.clone(),
            source,
            format!("The {function_name} function takes one argument: id (non-empty string)"),
        )),
    }
}

fn expect_state_name_arg<'a>(
    source: &'a str,
    span: &SourceSpan,
    value: &'a Value,
    function_name: &str,
) -> Result<&'a str, ScriptError> {
    match value {
        Value::Str(name) if !name.trim().is_empty() => Ok(name),
        _ => Err(ScriptError::from_span(
            span.clone(),
            source,
            format!("The {function_name} function requires a non-empty string state name"),
        )),
    }
}

fn lock_shared_state<'a>(
    ctx: &'a ScriptFuncContext<'_>,
    source: &str,
    span: &SourceSpan,
) -> Result<MutexGuard<'a, ScriptStateMap>, ScriptError> {
    ctx.shared_state.0.lock().map_err(|e| {
        ScriptError::from_span(
            span.clone(),
            source,
            format!("Failed to lock script shared state: {e}"),
        )
    })
}

async fn state_set_func(
    ctx: &ScriptFuncContext<'_>,
    source: &str,
    span: &SourceSpan,
    args: &[Value],
) -> Result<Value, ScriptError> {
    let (name, value) = match args {
        [name, value] => (
            expect_state_name_arg(source, span, name, "state_set")?,
            value,
        ),
        _ => {
            return Err(ScriptError::from_span(
                span.clone(),
                source,
                "The state_set function takes two arguments: name (non-empty string), value"
                    .to_string(),
            ));
        }
    };

    let mut shared_state = lock_shared_state(ctx, source, span)?;
    shared_state
        .entry(ctx.state_scope.clone())
        .or_default()
        .insert(name.to_string(), value.clone());
    Ok(Value::Int(0))
}

async fn state_get_func(
    ctx: &ScriptFuncContext<'_>,
    source: &str,
    span: &SourceSpan,
    args: &[Value],
) -> Result<Value, ScriptError> {
    let (name, default_value) = match args {
        [name, default_value] => (
            expect_state_name_arg(source, span, name, "state_get")?,
            default_value,
        ),
        _ => {
            return Err(ScriptError::from_span(
                span.clone(),
                source,
                "The state_get function takes two arguments: name (non-empty string), default_value"
                    .to_string(),
            ));
        }
    };

    let shared_state = lock_shared_state(ctx, source, span)?;
    Ok(shared_state
        .get(&ctx.state_scope)
        .and_then(|scope| scope.get(name))
        .cloned()
        .unwrap_or_else(|| default_value.clone()))
}

async fn state_has_func(
    ctx: &ScriptFuncContext<'_>,
    source: &str,
    span: &SourceSpan,
    args: &[Value],
) -> Result<Value, ScriptError> {
    let name = match args {
        [name] => expect_state_name_arg(source, span, name, "state_has")?,
        _ => {
            return Err(ScriptError::from_span(
                span.clone(),
                source,
                "The state_has function takes one argument: name (non-empty string)".to_string(),
            ));
        }
    };

    let shared_state = lock_shared_state(ctx, source, span)?;
    Ok(Value::Bool(
        shared_state
            .get(&ctx.state_scope)
            .is_some_and(|scope| scope.contains_key(name)),
    ))
}

async fn state_delete_func(
    ctx: &ScriptFuncContext<'_>,
    source: &str,
    span: &SourceSpan,
    args: &[Value],
) -> Result<Value, ScriptError> {
    let name = match args {
        [name] => expect_state_name_arg(source, span, name, "state_delete")?,
        _ => {
            return Err(ScriptError::from_span(
                span.clone(),
                source,
                "The state_delete function takes one argument: name (non-empty string)".to_string(),
            ));
        }
    };

    let mut shared_state = lock_shared_state(ctx, source, span)?;
    Ok(Value::Bool(
        shared_state
            .get_mut(&ctx.state_scope)
            .is_some_and(|scope| scope.remove(name).is_some()),
    ))
}

async fn state_clear_func(
    ctx: &ScriptFuncContext<'_>,
    source: &str,
    span: &SourceSpan,
    args: &[Value],
) -> Result<Value, ScriptError> {
    expect_no_args_func(source, span, args, "state_clear")?;
    let mut shared_state = lock_shared_state(ctx, source, span)?;
    shared_state.remove(&ctx.state_scope);
    Ok(Value::Int(0))
}

async fn enter_fps_func(
    ctx: &ScriptFuncContext<'_>,
    source: &str,
    span: &SourceSpan,
    args: &[Value],
) -> Result<Value, ScriptError> {
    let id = expect_id_arg_func(source, span, args, "enter_fps")?;
    execute_runtime_command(source, span, ctx, |ack| ScriptRuntimeCommand::EnterFps {
        id: id.to_string(),
        ack,
    })
    .await
}

async fn exit_fps_func(
    ctx: &ScriptFuncContext<'_>,
    source: &str,
    span: &SourceSpan,
    args: &[Value],
) -> Result<Value, ScriptError> {
    expect_no_args_func(source, span, args, "exit_fps")?;
    execute_runtime_command(source, span, ctx, |ack| ScriptRuntimeCommand::ExitFps {
        ack,
    })
    .await
}

async fn enter_raw_input_func(
    ctx: &ScriptFuncContext<'_>,
    source: &str,
    span: &SourceSpan,
    args: &[Value],
) -> Result<Value, ScriptError> {
    expect_no_args_func(source, span, args, "enter_raw_input")?;
    execute_runtime_command(source, span, ctx, |ack| {
        ScriptRuntimeCommand::EnterRawInput { ack }
    })
    .await
}

async fn exit_raw_input_func(
    ctx: &ScriptFuncContext<'_>,
    source: &str,
    span: &SourceSpan,
    args: &[Value],
) -> Result<Value, ScriptError> {
    expect_no_args_func(source, span, args, "exit_raw_input")?;
    execute_runtime_command(source, span, ctx, |ack| {
        ScriptRuntimeCommand::ExitRawInput { ack }
    })
    .await
}

async fn cancel_cast_func(
    ctx: &ScriptFuncContext<'_>,
    source: &str,
    span: &SourceSpan,
    args: &[Value],
) -> Result<Value, ScriptError> {
    let id = expect_id_arg_func(source, span, args, "cancel_cast")?;
    execute_runtime_command(source, span, ctx, |ack| ScriptRuntimeCommand::CancelCast {
        id: id.to_string(),
        ack,
    })
    .await
}

async fn release_cast_func(
    ctx: &ScriptFuncContext<'_>,
    source: &str,
    span: &SourceSpan,
    args: &[Value],
) -> Result<Value, ScriptError> {
    expect_no_args_func(source, span, args, "release_cast")?;
    execute_runtime_command(source, span, ctx, |ack| ScriptRuntimeCommand::ReleaseCast {
        ack,
    })
    .await
}

async fn swipe_func(
    ctx: &ScriptFuncContext<'_>,
    source: &str,
    span: &SourceSpan,
    args: &[Value],
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

    execute_script_action(
        source,
        span,
        ctx,
        ScriptAction::Swipe {
            pointer_id,
            interval,
            points,
        },
    )
    .await
}

async fn send_key_func(
    ctx: &ScriptFuncContext<'_>,
    source: &str,
    span: &SourceSpan,
    args: &[Value],
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

    execute_script_action(
        source,
        span,
        ctx,
        ScriptAction::Key {
            keycode,
            action: key_action,
            metastate,
            key_default: action == "default",
        },
    )
    .await
}

async fn paste_text_func(
    ctx: &ScriptFuncContext<'_>,
    source: &str,
    span: &SourceSpan,
    args: &[Value],
) -> Result<Value, ScriptError> {
    // paste_text(text)
    let format_msg = "The paste_text function takes one argument: text (string)";

    let text = match args {
        [Value::Str(text)] => text.clone(),
        _ => {
            return Err(ScriptError::from_span(
                span.clone(),
                source,
                format_msg.to_string(),
            ));
        }
    };

    execute_script_action(source, span, ctx, ScriptAction::PasteText { text }).await
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSpan {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl SourceSpan {
    fn point_at_start() -> Self {
        Self {
            start_line: 1,
            start_col: 1,
            end_line: 1,
            end_col: 2,
        }
    }
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptDiagnostic {
    pub severity: ScriptDiagnosticSeverity,
    pub message: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScriptDiagnosticSeverity {
    Error,
}

impl ScriptDiagnostic {
    fn error(message: String, span: SourceSpan) -> Self {
        Self {
            severity: ScriptDiagnosticSeverity::Error,
            message,
            span,
        }
    }

    fn from_pest_error(err: pest::error::Error<Rule>) -> Self {
        let span = match err.line_col {
            LineColLocation::Pos((line, col)) => SourceSpan {
                start_line: line,
                start_col: col,
                end_line: line,
                end_col: col + 1,
            },
            LineColLocation::Span((start_line, start_col), (end_line, end_col)) => {
                let min_end_col = if start_line == end_line {
                    start_col + 1
                } else {
                    1
                };
                SourceSpan {
                    start_line,
                    start_col,
                    end_line,
                    end_col: end_col.max(min_end_col),
                }
            }
        };

        Self::error(
            format!("{}\n{}", t!("mask.mapping.parseScriptFailed"), err),
            span,
        )
    }
}

impl From<&ScriptError> for ScriptDiagnostic {
    fn from(error: &ScriptError) -> Self {
        Self::error(error.message.clone(), error.span)
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
    Empty {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_statements_without_semicolons() {
        let ast = ScriptAST::new(
            r#"
let x = 1
x = x + 1
print(x)
"#,
        )
        .unwrap();

        assert_eq!(ast.program.stmts.len(), 3);
    }

    #[test]
    fn parses_blocks_without_trailing_semicolons() {
        let ast = ScriptAST::new(
            r#"
let x = 2
if x > 1 {
    print("a")
} else {
    print("b")
}
while x > 0 {
    x = x - 1
}
"#,
        )
        .unwrap();

        assert_eq!(ast.program.stmts.len(), 3);
        assert!(matches!(ast.program.stmts[1], Stmt::If { .. }));
        assert!(matches!(ast.program.stmts[2], Stmt::While { .. }));
    }

    #[test]
    fn parses_control_blocks_with_brace_on_next_line() {
        let ast = ScriptAST::new(
            r#"
let x = 2
if x > 1
{
    print("a")
}
while x > 0
{
    x = x - 1
}
"#,
        )
        .unwrap();

        assert_eq!(ast.program.stmts.len(), 3);
        assert!(matches!(ast.program.stmts[1], Stmt::If { .. }));
        assert!(matches!(ast.program.stmts[2], Stmt::While { .. }));
    }

    #[test]
    fn parses_control_blocks_with_comment_before_next_line_brace() {
        let ast = ScriptAST::new(
            r#"
let x = 2
if x > 1 // condition comment
{
    print("a")
}
while x > 0 // condition comment
{
    x = x - 1
}
"#,
        )
        .unwrap();

        assert_eq!(ast.program.stmts.len(), 3);
        assert!(matches!(ast.program.stmts[1], Stmt::If { .. }));
        assert!(matches!(ast.program.stmts[2], Stmt::While { .. }));
    }

    #[test]
    fn parses_simple_statement_before_closing_brace_on_same_line() {
        let ast = ScriptAST::new(r#"if true { print("x") }"#).unwrap();

        assert_eq!(ast.program.stmts.len(), 1);
        assert!(matches!(ast.program.stmts[0], Stmt::If { .. }));
    }

    #[test]
    fn parses_trailing_semicolon_after_block_as_empty_statement() {
        let ast = ScriptAST::new(r#"if true { print("x") };"#).unwrap();

        assert_eq!(ast.program.stmts.len(), 2);
        assert!(matches!(ast.program.stmts[0], Stmt::If { .. }));
        assert!(matches!(ast.program.stmts[1], Stmt::Empty { .. }));
    }

    #[test]
    fn parses_line_comments_after_unterminated_statement() {
        let ast = ScriptAST::new(
            r#"
let x = 1 // comment
print(x)
"#,
        )
        .unwrap();

        assert_eq!(ast.program.stmts.len(), 2);
    }

    #[test]
    fn rejects_same_line_simple_statements_without_separator() {
        assert!(ScriptAST::new("let x = 1 print(x)").is_err());
    }

    #[test]
    fn rejects_unknown_function_before_runtime() {
        let err = ScriptAST::new("tapp(1, 2, 3)").unwrap_err();

        assert!(err.contains("Function 'tapp' not defined"));
    }

    #[test]
    fn rejects_invalid_function_arguments_before_runtime() {
        let err = ScriptAST::new(
            r#"
wait("100")
tap(-1, 2, 3, "bad")
swipe(0, 10, 1, 2, 3)
"#,
        )
        .unwrap_err();

        assert!(err.contains("Argument 1 of wait must be integer"));
        assert!(err.contains("Argument 1 of tap must be a non-negative integer"));
        assert!(err.contains("must be one of: default, down, up, move"));
        assert!(err.contains("swipe function takes at least 6 arguments"));
    }

    #[test]
    fn rejects_undefined_variables_before_runtime() {
        let err = ScriptAST::new("print(x)").unwrap_err();

        assert!(err.contains("Variable 'x' not defined"));
    }

    #[test]
    fn rejects_variables_that_are_not_definitely_defined() {
        let err = ScriptAST::new(
            r#"
if RawInputFlag {
    let x = 1
}
print(x)
"#,
        )
        .unwrap_err();

        assert!(err.contains("Variable 'x' not defined"));
    }

    #[test]
    fn allows_variables_defined_by_constant_true_branch() {
        let ast = ScriptAST::new(
            r#"
if true {
    let x = 1
}
print(x)
"#,
        )
        .unwrap();

        assert_eq!(ast.program.stmts.len(), 2);
    }

    #[test]
    fn rejects_static_operator_errors_before_runtime() {
        let err = ScriptAST::new(
            r#"
let x = "a" - 1
let y = 1 / 0
"#,
        )
        .unwrap_err();

        assert!(err.contains("Arithmetic operator is not supported between string and integer"));
        assert!(err.contains("Division by zero"));
    }

    #[test]
    fn rejects_invalid_string_escape_sequences() {
        assert!(ScriptAST::new(r#"print("bad\q")"#).is_err());
    }

    #[test]
    fn rejects_raw_newline_in_string() {
        assert!(
            ScriptAST::new(
                "print(\"bad
string\")"
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_always_truthy_while_condition() {
        let err = ScriptAST::new("while true { wait(1) }").unwrap_err();

        assert!(err.contains("while condition is always truthy"));
    }
}
