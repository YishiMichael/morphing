// use morphing_core::scene::export_scenes;
// use morphing_core::scene::scene;
// use morphing_core::scene::SceneFilter;
// use morphing_core::timeline::Supervisor;
// use morphing_gizmos::mobjects::shape::Rect;

use morphing_core::{chapter, scene, GetField, Supervisor};

// #[scene]
// fn demo_scene(sv: &Supervisor<'_>) {
//     sv.wait(1.0);
//     let mobject = sv.spawn(Rect(nalgebra::Vector2::new(1.0, 1.0)));
//     sv.wait(6.0);
//     drop(mobject);
//     sv.wait(12.0);
// }

// #[scene(config = "my_config.toml")]
// fn another_demo_scene(sv: &Supervisor<'_>) {
//     sv.wait(1.0);
//     let mobject = sv.spawn(Rect(nalgebra::Vector2::new(1.0, 1.0)));
//     sv.wait(6.0);
//     drop(mobject);
//     sv.wait(10.0);
// }

#[chapter(config(toml = "", toml = "", yamla = ""))]
extern crate self;

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct Triangle {
    pub view_motor: Motor2D,
    pub projection_matrix: nalgebra::Matrix3<f32>,
    pub vertices: Vec<nalgebra::Vector2<f32>>,
}

pub struct TriangleShaderTypes {
    triangle_uniform: TriangleUniform,
    triangle_vertex: Vec<TriangleVertex>,
}

pub struct TriangleBuffers {
    triangle_uniform: wgpu::Buffer,
    triangle_vertex: wgpu::Buffer,
}

#[derive(encase::ShaderType)]
struct TriangleUniform {
    view_motor: nalgebra::Vector3<f32>,
    projection_matrix: nalgebra::Matrix3<f32>,
}

#[derive(encase::ShaderType)]
struct TriangleVertex {
    vertex: nalgebra::Vector2<f32>,
}

struct TriangleLifecycle {}

impl Lifecycle for TriangleLifecycle {
    // type Signal = f32;
    // type Resource = (bool,);

    fn setup(&self) -> Resource {
        (false,)
    }

    fn prepare(&self, signal: Signal, resource: &mut Resource) {
        resource.0 = (signal as i32) % 2 == 1;
    }

    fn render(&self, resource: &Resource) {
        if resource.0 {}
    }
}

#[derive(serde::Deserialize, GetField)]
struct MyConfig {}

// #[scene(config = toml, )]
fn my_scene(sv: &mut Supervisor<MyConfig>) {
    sv.with(TriangleLifecycle {});
}
