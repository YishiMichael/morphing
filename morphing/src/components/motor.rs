use geometric_algebra::ppga3d as pga;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(from = "nalgebra::Matrix4x2<f32>", into = "nalgebra::Matrix4x2<f32>")]
pub struct Motor(pub pga::Motor); // transpose?

impl From<nalgebra::Matrix4x2<f32>> for Motor {
    fn from(m: nalgebra::Matrix4x2<f32>) -> Self {
        Motor(pga::Motor::new(
            m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7],
        ))
    }
}

impl From<Motor> for nalgebra::Matrix4x2<f32> {
    fn from(Motor(m): Motor) -> Self {
        nalgebra::Matrix4x2::new(m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7])
    }
}
