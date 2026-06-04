use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use bevy::{
    ecs::schedule::{InternedScheduleLabel, ScheduleLabel},
    prelude::*,
};
use tokio::{runtime::Runtime, task::JoinHandle};

#[derive(Resource)]
struct UpdateTicks {
    ticks: Arc<AtomicUsize>,
    update_watch_tx: tokio::sync::watch::Sender<()>,
}

impl UpdateTicks {
    fn increment_ticks(&self) -> usize {
        let new_ticks = self.ticks.fetch_add(1, Ordering::SeqCst).wrapping_add(1);
        self.update_watch_tx
            .send(())
            .expect("failed to send update watch channel message");
        new_ticks
    }
}

pub struct TokioTasksPlugin {
    pub make_runtime: Box<dyn Fn() -> Runtime + Send + Sync + 'static>,
    pub schedule_label: InternedScheduleLabel,
}

impl Default for TokioTasksPlugin {
    fn default() -> Self {
        Self {
            make_runtime: Box::new(|| {
                let mut runtime = tokio::runtime::Builder::new_multi_thread();
                runtime.enable_all();
                runtime
                    .build()
                    .expect("failed to create Tokio runtime for background tasks")
            }),
            schedule_label: Update.intern(),
        }
    }
}

impl Plugin for TokioTasksPlugin {
    fn build(&self, app: &mut App) {
        let ticks = Arc::new(AtomicUsize::new(0));
        let (update_watch_tx, update_watch_rx) = tokio::sync::watch::channel(());
        let runtime = (self.make_runtime)();

        app.insert_resource(UpdateTicks {
            ticks: ticks.clone(),
            update_watch_tx,
        });
        app.insert_resource(TokioTasksRuntime::new(ticks, runtime, update_watch_rx));
        app.add_systems(self.schedule_label, tick_runtime_update);
    }
}

pub fn tick_runtime_update(world: &mut World) {
    let current_tick = {
        let tick_counter = match world.get_resource::<UpdateTicks>() {
            Some(counter) => counter,
            None => return,
        };
        tick_counter.increment_ticks()
    };

    if let Some(mut runtime) = world.remove_resource::<TokioTasksRuntime>() {
        runtime.execute_main_thread_work(world, current_tick);
        world.insert_resource(runtime);
    }
}

type MainThreadCallback = Box<dyn FnOnce(MainThreadContext) + Send + 'static>;

#[derive(Resource)]
pub struct TokioTasksRuntime(Box<TokioTasksRuntimeInner>);

struct TokioTasksRuntimeInner {
    runtime: Runtime,
    ticks: Arc<AtomicUsize>,
    update_watch_rx: tokio::sync::watch::Receiver<()>,
    update_run_tx: tokio::sync::mpsc::UnboundedSender<MainThreadCallback>,
    update_run_rx: tokio::sync::mpsc::UnboundedReceiver<MainThreadCallback>,
}

impl TokioTasksRuntime {
    fn new(
        ticks: Arc<AtomicUsize>,
        runtime: Runtime,
        update_watch_rx: tokio::sync::watch::Receiver<()>,
    ) -> Self {
        let (update_run_tx, update_run_rx) = tokio::sync::mpsc::unbounded_channel();

        Self(Box::new(TokioTasksRuntimeInner {
            runtime,
            ticks,
            update_watch_rx,
            update_run_tx,
            update_run_rx,
        }))
    }

    pub fn spawn_background_task<Task, Output, Spawnable>(
        &self,
        spawnable_task: Spawnable,
    ) -> JoinHandle<Output>
    where
        Task: Future<Output = Output> + Send + 'static,
        Output: Send + 'static,
        Spawnable: FnOnce(TaskContext) -> Task + Send + 'static,
    {
        let inner = &self.0;
        let context = TaskContext {
            update_watch_rx: inner.update_watch_rx.clone(),
            ticks: inner.ticks.clone(),
            update_run_tx: inner.update_run_tx.clone(),
        };
        inner.runtime.spawn(spawnable_task(context))
    }

    fn execute_main_thread_work(&mut self, world: &mut World, current_tick: usize) {
        self.0.runtime.block_on(async {
            tokio::task::yield_now().await;
        });
        while let Ok(runnable) = self.0.update_run_rx.try_recv() {
            runnable(MainThreadContext {
                world,
                current_tick,
            });
        }
    }
}

pub struct MainThreadContext<'a> {
    pub world: &'a mut World,
    pub current_tick: usize,
}

#[derive(Clone)]
pub struct TaskContext {
    update_watch_rx: tokio::sync::watch::Receiver<()>,
    update_run_tx: tokio::sync::mpsc::UnboundedSender<MainThreadCallback>,
    ticks: Arc<AtomicUsize>,
}

impl TaskContext {
    pub fn current_tick(&self) -> usize {
        self.ticks.load(Ordering::SeqCst)
    }

    pub async fn sleep_updates(&mut self, updates_to_sleep: usize) {
        let target_tick = self
            .ticks
            .load(Ordering::SeqCst)
            .wrapping_add(updates_to_sleep);
        while self.ticks.load(Ordering::SeqCst) < target_tick {
            if self.update_watch_rx.changed().await.is_err() {
                return;
            }
        }
    }

    pub async fn run_on_main_thread<Runnable, Output>(&mut self, runnable: Runnable) -> Output
    where
        Runnable: FnOnce(MainThreadContext) -> Output + Send + 'static,
        Output: Send + 'static,
    {
        let (output_tx, output_rx) = tokio::sync::oneshot::channel();
        if self
            .update_run_tx
            .send(Box::new(move |ctx| {
                if output_tx.send(runnable(ctx)).is_err() {
                    panic!("failed to send main thread callback output");
                }
            }))
            .is_err()
        {
            panic!("failed to send main thread callback");
        }
        output_rx
            .await
            .expect("failed to receive main thread callback output")
    }
}
