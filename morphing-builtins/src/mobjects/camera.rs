use morphing_core::config::Config;
use morphing_core::stage::ChannelType;
use morphing_core::stage::Layer;
use morphing_core::stage::LayerIndex;
use morphing_core::stage::World;
use morphing_core::timeline::Alive;
use morphing_core::timeline::CollapsedTimelineState;
use morphing_core::timeline::Spawn;
use morphing_core::timeline::TypeQueried;
use morphing_core::traits::Mobject;
use morphing_core::traits::MobjectBuilder;

use super::super::components::camera_transform::CameraTransform2D;
use super::super::components::motor::Motor2D;
use super::super::layers::builtin_planar::BuiltinPlanarLayer;
use super::super::layers::builtin_planar::BuiltinPlanarLayerPlanarTrimeshChannel;
use super::super::presentations::camera_transform::CameraTransform2DPresentation;
use super::super::presentations::planar_trimesh::PlanarTrimeshPresentation;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct Camera2DMobject {
    camera_transform_2d: CameraTransform2D,
}

impl Mobject for Camera2DMobject {}

// #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
// struct Camera3DMobject {
//     camera_transform_2d: CameraTransform2D,
// }

// impl Mobject for Camera3DMobject {}

#[derive(Default)]
struct PlanarCamera {
    pub aspect_ratio: Option<f32>,
    pub frame_height: Option<f32>,
}

impl MobjectBuilder<BuiltinPlanarLayer> for PlanarCamera {
    type OutputTypeQuery<W, LI> = TypeQueried<
        W,
        LI,
        BuiltinPlanarLayer,
        BuiltinPlanarLayerPlanarTrimeshChannel,
        ChannelType<PlanarTrimeshPresentation>,
        Camera2DMobject,
        CameraTransform2DPresentation
    >
    where
        W: World,
        LI: LayerIndex<W, Layer = BuiltinPlanarLayer>;

    fn instantiate<'t, 'a, W, LI>(
        self,
        layer_attachment_residue: &'a <BuiltinPlanarLayer as Layer>::Residue<'t, W, LI>,
        config: &'t Config,
    ) -> Alive<'t, 'a, Self::OutputTypeQuery<W, LI>, CollapsedTimelineState>
    where
        W: World,
        LI: LayerIndex<W, Layer = BuiltinPlanarLayer>,
    {
        // TODO: use config
        let aspect_ratio = self.aspect_ratio.unwrap_or(1.6);
        let frame_height = self.frame_height.unwrap_or(8.0);

        layer_attachment_residue
            .camera_transform_2d
            .spawn(Camera2DMobject {
                camera_transform_2d: CameraTransform2D {
                    view_motor: Motor2D(geometric_algebra::ppga2d::Motor::one()),
                    projection_matrix: nalgebra::Matrix3::new_nonuniform_scaling(
                        nalgebra::Vector3::new(
                            2.0 * aspect_ratio / frame_height,
                            2.0 / frame_height,
                            1.0,
                        ),
                    ),
                },
            })
    }
}

// #[derive(Default)]
// struct PerspectiveCamera {
//     pub aspect_ratio: Option<f32>,
//     pub frame_height: Option<f32>,
// }

// impl MobjectBuilder<BuiltinPlanarLayer> for PerspectiveCamera {
//     type OutputTypeQuery<W, LI> = TypeQueried<
//         W,
//         LI,
//         BuiltinPlanarLayer,
//         BuiltinPlanarLayerPlanarTrimeshChannel,
//         ChannelType<PlanarTrimeshPresentation>,
//         Camera3DMobject,
//         CameraTransform2DPresentation
//     >
//     where
//         W: World,
//         LI: LayerIndex<W, Layer = BuiltinPlanarLayer>;

//     fn instantiate<'t, 'a, W, LI>(
//         self,
//         layer_attachment_residue: &'a <BuiltinPlanarLayer as Layer>::Residue<'t, W, LI>,
//         config: &'t Config,
//     ) -> Alive<'t, 'a, Self::OutputTypeQuery<W, LI>, CollapsedTimelineState>
//     where
//         W: World,
//         LI: LayerIndex<W, Layer = BuiltinPlanarLayer>,
//     {
//         // TODO: use config
//         let aspect_ratio = self.aspect_ratio.unwrap_or(1.6);
//         let frame_height = self.frame_height.unwrap_or(8.0);

//         layer_attachment_residue
//             .camera_transform_2d
//             .spawn(Camera3DMobject {
//                 camera_transform_2d: CameraTransform2D {
//                     view_motor: Motor2D(geometric_algebra::ppga2d::Motor::one()),
//                     projection_matrix: nalgebra::Matrix3::new_nonuniform_scaling(
//                         nalgebra::Vector3::new(
//                             2.0 * aspect_ratio / frame_height,
//                             2.0 / frame_height,
//                             1.0,
//                         ),
//                     ),
//                 },
//             })
//     }
// }

// impl Camera2DMobject {
//     fn new(aspect_ratio: f32, frame_height: f32) -> Self {
//         Self {
//             view_motor: Motor2D(geometric_algebra::ppga2d::Motor::one()),
//             projection_matrix: nalgebra::Matrix3::new_nonuniform_scaling(nalgebra::Vector3::new(x, y, z)) ::new_perspective(
//                 16.0 / 9.0,
//                 40.0_f32.to_radians(),
//                 0.1,
//                 100.0,
//             ),
//         }
//     }
// }

// impl Default for Camera3DMobject {
//     fn default() -> Self {
//         Self {
//             view_motor: Motor3D(geometric_algebra::ppga3d::Motor::one().geometric_product(
//                 geometric_algebra::ppga3d::Translator::new(1.0, 0.0, 0.0, 5.0),
//             )),
//             projection_matrix: nalgebra::Matrix4::new_perspective(
//                 16.0 / 9.0,
//                 40.0_f32.to_radians(),
//                 0.1,
//                 100.0,
//             ),
//         }
//     }
// }
