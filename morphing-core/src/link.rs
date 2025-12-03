// trait Mobject {
//     type ResourceRef<'r>;
//     type Context;

//     fn render<'r>(resource_ref: Self::ResourceRef<'r>, context: &mut Self::Context);
// }

use super::scene::Lifecycle;
use super::scene::Supervisor;

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

// struct TimelineBuilder<A, TM, R> {
//     action: A,
//     timeline_metric: TM,
//     rate: R,
// }

pub use morphing_macros::chapter;
pub use morphing_macros::scene;

#[doc(hidden)]
pub use config;

#[doc(hidden)]
pub use inventory;

#[doc(hidden)]
pub struct Symbol<T> {
    pub(crate) name: String,
    pub(crate) config: Vec<config::File<config::FileSourceString, config::FileFormat>>,
    pub(crate) content: T,
}

inventory::collect!(SceneSymbol);

#[doc(hidden)]
pub type SceneSymbol = Symbol<
    Box<
        dyn Fn(config::ConfigBuilder<config::builder::DefaultState>) -> Vec<Box<dyn Lifecycle>>
            + Sync,
    >,
>;

#[doc(hidden)]
pub fn scene_symbol<C: 'static + serde::de::DeserializeOwned, const N: usize>(
    name: &str,
    config: [config::File<config::FileSourceString, config::FileFormat>; N],
    scene: fn(&mut Supervisor<C>),
) -> SceneSymbol {
    Symbol {
        name: name.into(),
        config: config.into(),
        content: Box::new(move |config_builder| {
            let configuration = config_builder.build().unwrap().try_deserialize().unwrap();
            let mut supervisor = Supervisor {
                time: 0.0,
                lifecycles: Vec::new(),
                config: configuration,
            };
            scene(&mut supervisor);
            supervisor.lifecycles
        }),
    }
}

#[doc(hidden)]
pub type ChapterSymbol = Symbol<std::collections::HashMap<String, &'static SceneSymbol>>;

#[doc(hidden)]
pub fn chapter_symbol<const N: usize>(
    name: &str,
    config: [config::File<config::FileSourceString, config::FileFormat>; N],
    scenes: inventory::iter<SceneSymbol>,
) -> ChapterSymbol {
    Symbol {
        name: name.into(),
        config: config.into(),
        content: scenes
            .into_iter()
            .map(|symbol| (symbol.name.clone(), symbol))
            .collect(),
    }
}

pub(crate) fn call_entrypoint(chapter_path: &str) -> ChapterSymbol {
    let func: libloading::Symbol<extern "Rust" fn() -> ChapterSymbol> = unsafe {
        let lib = libloading::Library::new(chapter_path).unwrap();
        lib.get(b"__morphing_entrypoint__\0").unwrap(); // expecting #[chapter] invocation
    };
    func()
}

// pub mod config_formats {
//     macro_rules! config_format {
//         ($name:ident = $format:expr) => {
//             pub fn $name(s: &str) -> config::File<config::FileSourceString, config::FileFormat> {
//                 config::File::from_str(s, $format)
//             }
//         };
//     }

//     config_format!(toml = config::FileFormat::Toml);
//     config_format!(json = config::FileFormat::Json);
//     config_format!(yaml = config::FileFormat::Yaml);
//     config_format!(ini = config::FileFormat::Ini);
//     config_format!(ron = config::FileFormat::Ron);
//     config_format!(json5 = config::FileFormat::Json5);
// }
