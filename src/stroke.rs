use super::mobject::Mobject;
use super::path::Path;

#[derive(Clone)]
pub struct Stroke {
    pub path: Path,
    pub color: rgb::Rgba<f32>,
    pub options: lyon::tessellation::StrokeOptions,
}

impl Mobject for Stroke {}
