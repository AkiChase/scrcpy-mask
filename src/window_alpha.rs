use bevy::{
    prelude::*,
    window::{CompositeAlphaMode, PrimaryWindow, RawHandleWrapper},
};

/// Marker component inserted on the window entity after probing.
/// `apply_alpha_mode` in PostStartup reads this and writes the real `Window.composite_alpha_mode`.
#[derive(Component, Debug)]
pub struct PendingAlphaMode(pub CompositeAlphaMode);

/// Probe the current GPU for supported alpha compositing modes.
///
/// Creates a temporary wgpu instance + surface (no swap chain, no configure)
/// to query `SurfaceCapabilities::alpha_modes`, then returns the best choice.
/// All temporary resources are dropped before this function returns.
fn probe_alpha_modes(handle: &RawHandleWrapper) -> Option<CompositeAlphaMode> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

    // Safety: the RawHandleWrapper holds valid window/display handles from a live winit window.
    // The surface is created and dropped on the main thread during Startup, before Bevy's
    // renderer creates its own surface from the same HWND.
    let surface = unsafe {
        instance
            .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: handle.get_display_handle(),
                raw_window_handle: handle.get_window_handle(),
            })
    }
    .ok()?;

    debug!(
        "alpha probe: surface created (window={:?}, display={:?})",
        handle.get_window_handle(),
        handle.get_display_handle(),
    );

    let adapter = pollster::block_on(instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
        },
    ))
    .ok()?;

    debug!(
        "alpha probe: adapter obtained: {:?}",
        adapter.get_info(),
    );

    let caps = surface.get_capabilities(&adapter);
    debug!(
        "alpha probe: supported alpha_modes={:?}, formats={:?}",
        caps.alpha_modes,
        caps.formats.iter().take(8).collect::<Vec<_>>(),
    );

    let chosen = if caps
        .alpha_modes
        .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
    {
        CompositeAlphaMode::PreMultiplied
    } else {
        // Auto lets the system decide — may still produce transparency on some GPUs,
        // and degrades gracefully to opaque on those that don't support it.
        CompositeAlphaMode::Auto
    };

    Some(chosen)
}

/// Runs in `Startup`: extracts `RawHandleWrapper` from the primary window, probes GPU
/// capabilities via a temporary wgpu instance, and queues the chosen alpha mode via
/// `PendingAlphaMode` marker for the next schedule.
pub fn detect_alpha_mode(
    mut has_run: Local<bool>,
    windows: Query<(Entity, &Window, &RawHandleWrapper), With<PrimaryWindow>>,
    mut commands: Commands,
) {
    if *has_run {
        return;
    }
    *has_run = true;

    let Some((entity, window, handle)) = windows.iter().next() else {
        warn!("alpha probe: no primary window with RawHandleWrapper found");
        return;
    };

    debug!(
        "alpha probe: current composite_alpha_mode={:?}",
        window.composite_alpha_mode,
    );

    match probe_alpha_modes(handle) {
        Some(chosen) => {
            info!(
                "alpha probe: chose {chosen:?} (was {:?})",
                window.composite_alpha_mode,
            );
            commands.entity(entity).insert(PendingAlphaMode(chosen));
        }
        None => {
            warn!(
                "alpha probe: probe failed, keeping current mode {:?}",
                window.composite_alpha_mode,
            );
        }
    }
}

/// Runs in `PostStartup`: reads `PendingAlphaMode` and writes the value to
/// `Window.composite_alpha_mode`, then removes the marker.
pub fn apply_alpha_mode(
    mut windows: Query<(
        Entity,
        &mut Window,
        &PendingAlphaMode,
    )>,
    mut commands: Commands,
) {
    for (entity, mut window, pending) in &mut windows {
        debug!(
            "alpha probe: applying {pending:?} (was {:?})",
            window.composite_alpha_mode,
        );
        window.composite_alpha_mode = pending.0;
        commands.entity(entity).remove::<PendingAlphaMode>();
    }
}
