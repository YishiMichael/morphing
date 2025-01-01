use super::mobject::Mobject;
use super::path::Path;

#[derive(Clone)]
pub struct Fill {
    pub path: Path,
    pub color: rgb::Rgba<f32>,
    pub options: lyon::tessellation::FillOptions,
}

impl Mobject for Fill {}
