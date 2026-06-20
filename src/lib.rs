pub mod config;
pub mod mask;
pub mod scrcpy;
pub mod tokio_tasks;
pub mod utils;
pub mod web;

#[cfg(not(target_os = "macos"))]
pub mod window_alpha;

rust_i18n::i18n!(
    "assets/locales",
    fallback = "en-US",
    minify_key = true,
    minify_key_len = 12,
    minify_key_prefix = "t_",
    minify_key_thresh = 64
);

pub const DEFAULT_LANGUAGE: &str = "en-US";

pub fn available_languages() -> Vec<&'static str> {
    rust_i18n::available_locales!()
}

pub fn is_available_language(language: &str) -> bool {
    available_languages().contains(&language)
}
