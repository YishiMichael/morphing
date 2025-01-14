use super::fill::Fill;
use super::path::Path;
use super::stroke::Stroke;

#[derive(Clone)]
pub struct Shape {
    pub path: Path,
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
}
