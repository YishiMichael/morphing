use std::path::PathBuf;

use serde::Deserialize;

#[derive(Default, Deserialize)]
pub(crate) struct Config {
    #[serde(default = "WindowConfig::default")]
    pub(crate) window: WindowConfig,
    #[serde(default = "VideoConfig::default")]
    pub(crate) video: VideoConfig,
    #[serde(default = "TypstConfig::default")]
    pub(crate) typst: TypstConfig,
}

#[derive(Deserialize)]
pub(crate) struct WindowConfig {
    #[serde(default = "WindowConfig::default_size")]
    pub(crate) size: (u32, u32),
    #[serde(default = "WindowConfig::default_base_speed")]
    pub(crate) base_speed: f32,
    #[serde(default = "WindowConfig::default_forward_seconds")]
    pub(crate) forward_seconds: f32,
    #[serde(default = "WindowConfig::default_fast_forward_seconds")]
    pub(crate) fast_forward_seconds: f32,
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

impl WindowConfig {
    fn default_size() -> (u32, u32) {
        Self::default().size
    }

    fn default_base_speed() -> f32 {
        Self::default().base_speed
    }

    fn default_forward_seconds() -> f32 {
        Self::default().forward_seconds
    }

    fn default_fast_forward_seconds() -> f32 {
        Self::default().fast_forward_seconds
    }
}

#[derive(Deserialize)]
pub(crate) struct VideoConfig {
    #[serde(default = "VideoConfig::default_size")]
    pub(crate) size: (u32, u32),
    #[serde(default = "VideoConfig::default_fps")]
    pub(crate) fps: f32,
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            size: (1920, 1080),
            fps: 60.0,
        }
    }
}

impl VideoConfig {
    fn default_size() -> (u32, u32) {
        Self::default().size
    }

    fn default_fps() -> f32 {
        Self::default().fps
    }
}

#[derive(Deserialize)]
pub(crate) struct TypstConfig {
    #[serde(default = "TypstConfig::default_root")]
    pub(crate) root: PathBuf,
    #[serde(default = "TypstConfig::default_inputs")]
    pub(crate) inputs: Vec<(String, String)>,
    #[serde(default = "TypstConfig::default_font_paths")]
    pub(crate) font_paths: Vec<PathBuf>,
    #[serde(default = "TypstConfig::default_include_system_fonts")]
    pub(crate) include_system_fonts: bool,
    #[serde(default = "TypstConfig::default_include_embedded_fonts")]
    pub(crate) include_embedded_fonts: bool,
    #[serde(default = "TypstConfig::default_package_path")]
    pub(crate) package_path: Option<PathBuf>,
    #[serde(default = "TypstConfig::default_package_cache_path")]
    pub(crate) package_cache_path: Option<PathBuf>,
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

impl TypstConfig {
    fn default_root() -> PathBuf {
        Self::default().root
    }

    fn default_inputs() -> Vec<(String, String)> {
        Self::default().inputs
    }

    fn default_font_paths() -> Vec<PathBuf> {
        Self::default().font_paths
    }

    fn default_include_system_fonts() -> bool {
        Self::default().include_system_fonts
    }

    fn default_include_embedded_fonts() -> bool {
        Self::default().include_embedded_fonts
    }

    fn default_package_path() -> Option<PathBuf> {
        Self::default().package_path
    }

    fn default_package_cache_path() -> Option<PathBuf> {
        Self::default().package_cache_path
    }
}
