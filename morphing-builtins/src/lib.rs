#![feature(new_range_api)]

use morphing_core::config::ConfigFallbackContent;

pub mod components;
pub mod mobjects;
pub mod timelines;

inventory::submit! {
    ConfigFallbackContent(include_str!("configs/general.toml"))
}
inventory::submit! {
    ConfigFallbackContent(include_str!("configs/typst.toml"))
}
