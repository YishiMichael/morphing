use std::path::PathBuf;

use super::palette::BLACK;
use super::palette::WHITE;

#[derive(Default)]
pub struct Config {
    pub window: WindowConfig,
    pub video: VideoConfig,
    pub style: StyleConfig,
    pub typst: TypstConfig,
}

pub struct WindowConfig {
    pub size: (u32, u32),
    pub base_speed: f32,
    pub forward_seconds: f32,
    pub fast_forward_seconds: f32,
}

pub struct VideoConfig {
    pub size: (u32, u32),
    pub fps: f32,
}

pub struct StyleConfig {
    pub color: palette::Srgb<f32>,
    pub background_color: palette::Srgb<f32>,
}

pub struct TypstConfig {
    pub root: PathBuf,
    pub inputs: Vec<(String, String)>,
    pub font_paths: Vec<PathBuf>,
    pub include_system_fonts: bool,
    pub include_embedded_fonts: bool,
    pub package_path: Option<PathBuf>,
    pub package_cache_path: Option<PathBuf>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            size: (960, 540),
            base_speed: 1.0,
            forward_seconds: 5.0,
            fast_forward_seconds: 30.0,
        }
    }
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            size: (1920, 1080),
            fps: 60.0,
        }
    }
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            color: WHITE.into(),
            background_color: BLACK.into(),
        }
    }
}

impl Default for TypstConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            inputs: Vec::new(),
            font_paths: Vec::new(),
            include_system_fonts: true,
            include_embedded_fonts: true,
            package_path: None,
            package_cache_path: None,
        }
    }
}
