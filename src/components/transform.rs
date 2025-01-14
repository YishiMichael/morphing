use geometric_algebra::ppga3d as pga;

#[derive(Clone)]
pub struct Transform {
    motor: pga::Motor,
    scale_exponent: f32,
}
