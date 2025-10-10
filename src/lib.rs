pub mod config;
pub mod mask;
pub mod scrcpy;
pub mod utils;
pub mod web;

rust_i18n::i18n!(
    "assets/locales",
    fallback = "en-US",
    minify_key = true,
    minify_key_len = 12,
    minify_key_prefix = "t_",
    minify_key_thresh = 64
);