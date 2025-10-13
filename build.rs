fn main() {
    #[cfg(target_os = "linux")]
    {
        pkg_config::probe_library("x11").expect("Failed to find libX11 via pkg-config");
        pkg_config::probe_library("vdpau").expect("Failed to find libvdpau via pkg-config");
        pkg_config::probe_library("libva").expect("Failed to find libva");
        pkg_config::probe_library("libva-x11").expect("Failed to find libva-x11");
        pkg_config::probe_library("libva-drm").expect("Failed to find libva-drm");
        pkg_config::probe_library("libdrm").expect("Failed to find libdrm via pkg-config");
    }
}
