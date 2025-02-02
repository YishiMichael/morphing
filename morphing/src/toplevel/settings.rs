use std::path::PathBuf;

use super::palette::BLACK;
use super::palette::WHITE;

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct SceneSettings {
    pub video: VideoSettings,
    pub style: StyleSettings,
    pub typst: TypstSettings,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct VideoSettings {
    pub size: (u32, u32),
    pub background_color: palette::Srgb<f32>,
    pub fps: f32,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct StyleSettings {
    pub color: palette::Srgb<f32>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct TypstSettings {
    pub root: PathBuf,
    pub inputs: Vec<(String, String)>,
    pub font_paths: Vec<PathBuf>,
    pub include_system_fonts: bool,
    pub include_embedded_fonts: bool,
    pub package_path: Option<PathBuf>,
    pub package_cache_path: Option<PathBuf>,
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            size: (1920, 1080),
            background_color: BLACK.into(),
            fps: 60.0,
        }
    }
}

impl Default for StyleSettings {
    fn default() -> Self {
        Self {
            color: WHITE.into(),
        }
    }
}

impl Default for TypstSettings {
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
