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
fn probe_alpha_modes(handle: &RawHandleWrapper) -> Result<(CompositeAlphaMode, bool), String> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());

    // Safety: the RawHandleWrapper holds valid window/display handles from a live winit window.
    // The surface is created and dropped on the main thread during Startup, before Bevy's
    // renderer creates its own surface from the same HWND.
    let surface = match unsafe {
        instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: Some(handle.get_display_handle()),
            raw_window_handle: handle.get_window_handle(),
        })
    } {
        Ok(surface) => surface,
        Err(e) => return Err(format!("failed to create temporary surface: {e}")),
    };

    let adapter = match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        power_preference: wgpu::PowerPreference::default(),
        force_fallback_adapter: false,
    })) {
        Ok(adapter) => adapter,
        Err(e) => return Err(format!("failed to request compatible adapter: {e}")),
    };

    let caps = surface.get_capabilities(&adapter);
    info!("alpha probe: supported alpha_modes={:?}", caps.alpha_modes);

    let desired_supported = caps
        .alpha_modes
        .contains(&wgpu::CompositeAlphaMode::PreMultiplied);
    let chosen = if desired_supported {
        CompositeAlphaMode::PreMultiplied
    } else {
        // Auto lets the system decide — may still produce transparency on some GPUs,
        // and degrades gracefully to opaque on those that don't support it.
        CompositeAlphaMode::Auto
    };

    Ok((chosen, desired_supported))
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

    match probe_alpha_modes(handle) {
        Ok((chosen, desired_supported)) => {
            info!(
                "alpha probe: desired PreMultiplied supported={desired_supported}, final selection={chosen:?}"
            );
            commands.entity(entity).insert(PendingAlphaMode(chosen));
        }
        Err(e) => {
            warn!(
                "alpha probe: probe failed ({e}), keeping current mode {:?}",
                window.composite_alpha_mode
            );
        }
    }
}

/// Runs in `PostStartup`: reads `PendingAlphaMode` and writes the value to
/// `Window.composite_alpha_mode`, then removes the marker.
pub fn apply_alpha_mode(
    mut windows: Query<(Entity, &mut Window, &PendingAlphaMode)>,
    mut commands: Commands,
) {
    for (entity, mut window, pending) in &mut windows {
        window.composite_alpha_mode = pending.0;
        commands.entity(entity).remove::<PendingAlphaMode>();
    }
}
