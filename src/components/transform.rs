use geometric_algebra::ppga3d as pga;
use geometric_algebra::Zero;

#[derive(Clone)]
pub struct Transform {
    motor: pga::Motor,
    scale_exponent: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            motor: pga::Motor::zero(),
            scale_exponent: 0.0,
        }
    }
}
