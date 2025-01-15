#[derive(Clone)]
pub struct Paint {
    pub solid: Option<palette::Srgba<f32>>,
    pub gradients: Vec<Gradient>,
}

#[derive(Clone)]
pub struct Gradient {
    pub from: glam::Vec2,
    pub to: glam::Vec2,
    pub radius_diff: f32,
    pub radius_quotient: f32,
    pub radial_stops: Option<Vec<(f32, palette::Srgba<f32>)>>,
    pub angular_stops: Option<Vec<(f32, palette::Srgba<f32>)>>,
}
