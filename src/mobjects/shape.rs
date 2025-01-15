use super::super::components::fill::Fill;
use super::super::components::path::Path;
use super::super::components::stroke::Stroke;
use super::super::components::transform::Transform;
use super::super::toplevel::renderer::Renderer;
use super::mobject::Mobject;

#[derive(Clone)]
pub struct ShapeMobject {
    pub(crate) transform: Transform,
    pub(crate) path: Path,
    pub(crate) fill: Option<Fill>,
    pub(crate) stroke: Option<Stroke>,
}

impl Mobject for ShapeMobject {
    fn render(&self, renderer: &Renderer) {
        println!("Rendered Shape!")
    }
}
