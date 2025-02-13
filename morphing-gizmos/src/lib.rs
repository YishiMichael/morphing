use morphing_core::config::ConfigFallbackContent;

pub mod components;
pub mod mobjects;
pub mod timelines;

inventory::submit! {
    ConfigFallbackContent(include_str!("general_config.toml"))
}
inventory::submit! {
    ConfigFallbackContent(include_str!("typst_config.toml"))
}
