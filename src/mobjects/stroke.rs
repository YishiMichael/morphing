use super::super::components::path::Path;
use super::mobject::Mobject;

#[derive(Clone)]
pub struct Stroke {
    pub path: Path,
    pub color: rgb::Rgba<f32>,
    pub options: lyon::tessellation::StrokeOptions,
}

// impl Mobject for Stroke {
//     fn render(&self) {
//         println!("Rendered Stroke!")
//     }
// }
