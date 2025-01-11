use super::super::components::path::Path;
use super::mobject::Mobject;

#[derive(Clone)]
pub struct Fill {
    pub path: Path,
    pub color: rgb::Rgba<f32>,
    pub options: lyon::tessellation::FillOptions,
}

impl Mobject for Fill {
    fn render(&self) {
        println!("Rendered Fill!")
    }
}
