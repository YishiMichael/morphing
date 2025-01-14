use super::super::components::shape::Shape;
use super::super::components::transform::Transform;
use super::super::toplevel::renderer::Renderer;
use super::mobject::Mobject;

#[derive(Clone)]
pub struct ShapeMobject {
    transform: Transform,
    shape: Shape,
}

impl Mobject for ShapeMobject {
    type Diff = Self;

    fn apply_diff(&self, diff: Self::Diff) -> Self {
        todo!()
    }

    fn render(&self, renderer: &Renderer) {
        println!("Rendered Shape!")
    }
}
