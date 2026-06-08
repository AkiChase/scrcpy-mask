use std::{collections::HashMap, future::Future};

use bevy::math::Vec2;
use tokio::sync::broadcast;

use crate::{
    mask::mapping::{
        script::BindMappingScriptHooks,
        script_helper::{
            ScriptAST, ScriptError, ScriptRuntimeCommand, ScriptRuntimeCommandSender,
            ScriptSharedState,
        },
    },
    scrcpy::control_msg::ScrcpyControlMsg,
    utils::ChannelSenderCS,
};

#[derive(Clone)]
pub struct MappingExecutionContext {
    pub cs_tx: broadcast::Sender<ScrcpyControlMsg>,
    pub script_command_tx: crossbeam_channel::Sender<ScriptRuntimeCommand>,
    pub shared_state: ScriptSharedState,
    pub state_scope: String,
    pub original_size: Vec2,
    pub cursor_pos: Vec2,
    pub mask_size: Vec2,
    pub raw_input_flag: bool,
    pub fps_mode_flag: bool,
}

pub fn make_mapping_execution_context(
    cs_tx: &ChannelSenderCS,
    script_command_tx: &ScriptRuntimeCommandSender,
    shared_state: &ScriptSharedState,
    state_scope: String,
    original_size: Vec2,
    cursor_pos: Vec2,
    mask_size: Vec2,
    raw_input_flag: bool,
    fps_mode_flag: bool,
) -> MappingExecutionContext {
    MappingExecutionContext {
        cs_tx: cs_tx.0.clone(),
        script_command_tx: script_command_tx.0.clone(),
        shared_state: shared_state.clone(),
        state_scope,
        original_size,
        cursor_pos,
        mask_size,
        raw_input_flag,
        fps_mode_flag,
    }
}

#[derive(Debug)]
pub enum MappingExecutionError {
    Script(ScriptError),
    Action(String),
}

impl From<ScriptError> for MappingExecutionError {
    fn from(value: ScriptError) -> Self {
        Self::Script(value)
    }
}

pub async fn run_script_hook(
    ast: &ScriptAST,
    ctx: &MappingExecutionContext,
) -> Result<(), ScriptError> {
    if ast.empty {
        return Ok(());
    }

    ast.eval_script(
        &ctx.cs_tx,
        &ctx.script_command_tx,
        &ctx.shared_state,
        &ctx.state_scope,
        ctx.original_size,
        ctx.cursor_pos,
        ctx.mask_size,
        ctx.raw_input_flag,
        ctx.fps_mode_flag,
    )
    .await
}

/// Runs hooks for an atomic mapping action.
///
/// Atomic mappings must complete their full action inside `action`. Continuous mappings
/// with separate press/release phases should use `MappingLifecycleState` so a release
/// arriving while `before_script` is still running can be replayed after the start phase.
pub async fn run_with_hooks<F, Fut>(
    hooks: BindMappingScriptHooks,
    ctx: MappingExecutionContext,
    action: F,
) -> Result<(), MappingExecutionError>
where
    F: FnOnce(MappingExecutionContext) -> Fut,
    Fut: Future<Output = Result<(), MappingExecutionError>>,
{
    if !hooks.before_script_ast.empty {
        run_script_hook(&hooks.before_script_ast, &ctx).await?;
    }
    action(ctx.clone()).await?;
    if !hooks.after_script_ast.empty {
        run_script_hook(&hooks.after_script_ast, &ctx).await?;
    }
    Ok(())
}

pub enum MappingLifecycleStart<ReleaseContext> {
    Stale,
    Ready {
        pending_release: Option<ReleaseContext>,
    },
}

struct PendingLifecycleRelease<ReleaseContext> {
    version: u64,
    context: ReleaseContext,
}

/// Shared press/release bookkeeping for continuous mappings with script hooks.
///
/// Contract for mappings using this state:
/// - `before_script` runs once before the mapping begins.
/// - `after_script` runs once after the mapping ends.
/// - If release arrives before `before_script` finishes, record it with
///   `record_early_release`; once `before_script` succeeds, the mapping must still run
///   begin -> end -> after exactly once.
pub struct MappingLifecycleState<ReleaseContext> {
    versions: HashMap<String, u64>,
    pending_starts: HashMap<String, u64>,
    pending_releases: HashMap<String, PendingLifecycleRelease<ReleaseContext>>,
}

impl<ReleaseContext> Default for MappingLifecycleState<ReleaseContext> {
    fn default() -> Self {
        Self {
            versions: HashMap::default(),
            pending_starts: HashMap::default(),
            pending_releases: HashMap::default(),
        }
    }
}

impl<ReleaseContext> MappingLifecycleState<ReleaseContext> {
    pub fn begin_start(&mut self, action: &str) -> u64 {
        let version = self.versions.entry(action.to_string()).or_default();
        *version = version.wrapping_add(1);
        let version = *version;
        self.pending_starts.insert(action.to_string(), version);
        self.pending_releases.remove(action);
        version
    }

    pub fn cancel_start(&mut self, action: &str, version: u64) {
        if self.current_version(action) == version {
            self.pending_starts.remove(action);
            self.pending_releases.remove(action);
        }
    }

    pub fn finish_start(
        &mut self,
        action: &str,
        version: u64,
    ) -> MappingLifecycleStart<ReleaseContext> {
        if self.current_version(action) != version {
            return MappingLifecycleStart::Stale;
        }

        self.pending_starts.remove(action);
        let pending_release = self
            .pending_releases
            .remove(action)
            .filter(|release| release.version == version)
            .map(|release| release.context);

        MappingLifecycleStart::Ready { pending_release }
    }

    pub fn record_early_release(&mut self, action: &str, context: ReleaseContext) -> bool {
        let Some(version) = self.pending_starts.get(action).copied() else {
            return false;
        };

        self.pending_releases.insert(
            action.to_string(),
            PendingLifecycleRelease { version, context },
        );
        true
    }

    pub fn clear_pending(&mut self, action: &str) {
        self.pending_starts.remove(action);
        self.pending_releases.remove(action);
    }

    pub fn cancel_pending(&mut self, action: &str) {
        let version = self.versions.entry(action.to_string()).or_default();
        *version = version.wrapping_add(1);
        self.pending_starts.remove(action);
        self.pending_releases.remove(action);
    }

    fn current_version(&self, action: &str) -> u64 {
        self.versions.get(action).copied().unwrap_or_default()
    }
}
