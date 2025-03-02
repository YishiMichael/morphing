#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(from = "nalgebra::Vector4<f32>", into = "nalgebra::Vector4<f32>")]
pub struct Motor2D(pub geometric_algebra::ppga2d::Motor);

impl From<nalgebra::Vector4<f32>> for Motor2D {
    fn from(m: nalgebra::Vector4<f32>) -> Self {
        Motor2D(geometric_algebra::ppga2d::Motor::new(
            m[0], m[1], m[2], m[3],
        ))
    }
}

impl From<Motor2D> for nalgebra::Vector4<f32> {
    fn from(Motor2D(m): Motor2D) -> Self {
        nalgebra::Vector4::new(m[0], m[1], m[2], m[3])
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(from = "nalgebra::Matrix4x2<f32>", into = "nalgebra::Matrix4x2<f32>")]
pub struct Motor3D(pub geometric_algebra::ppga3d::Motor); // transpose?

impl From<nalgebra::Matrix4x2<f32>> for Motor3D {
    fn from(m: nalgebra::Matrix4x2<f32>) -> Self {
        Motor3D(geometric_algebra::ppga3d::Motor::new(
            m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7],
        ))
    }
}

impl From<Motor3D> for nalgebra::Matrix4x2<f32> {
    fn from(Motor3D(m): Motor3D) -> Self {
        nalgebra::Matrix4x2::new(m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7])
    }
}
