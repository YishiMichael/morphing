use std::borrow::Cow;
use std::sync::OnceLock;

use super::super::components::camera_transform::CameraTransform2DShaderTypes;
use super::super::components::paint::PaintShaderTypes;
use super::super::components::transform::TransformShaderTypes;
use super::super::presentations::camera_transform::CameraTransform2DPresentation;
use super::super::presentations::planar_trimesh::PlanarTrimeshPresentation;

/*
struct Vertex {
    @location(0) position: vec2<f32>,
}
*/
#[derive(encase::ShaderType)]
pub(crate) struct Vertex {
    position: nalgebra::Vector2<f32>,
}

#[allow(non_camel_case_types)]
pub struct BuiltinPlanarLayer<
    camera_transform_2d = ::morphing_core::stage::ChannelType<CameraTransform2DPresentation>,
    planar_trimesh = ::morphing_core::stage::ChannelType<PlanarTrimeshPresentation>,
> {
    pub camera_transform_2d: camera_transform_2d,
    pub planar_trimesh: planar_trimesh,
}

impl ::morphing_core::stage::Archive for BuiltinPlanarLayer {
    type Output = BuiltinPlanarLayer<
        <::morphing_core::stage::ChannelType<CameraTransform2DPresentation> as ::morphing_core::stage::Archive>::Output,
        <::morphing_core::stage::ChannelType<PlanarTrimeshPresentation> as ::morphing_core::stage::Archive>::Output,
    >;

    fn new() -> Self {
        BuiltinPlanarLayer {
            camera_transform_2d: ::morphing_core::stage::Archive::new(),
            planar_trimesh: ::morphing_core::stage::Archive::new(),
        }
    }

    fn merge<TE>(
        &self,
        output: Self::Output,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: ::core::range::Range<::morphing_core::timer::Time>,
        child_time_interval: ::core::range::Range<::morphing_core::timer::Time>,
    ) where
        TE: ::morphing_core::timer::IncreasingTimeEval<
            OutputTimeMetric = ::morphing_core::timer::NormalizedTimeMetric,
        >,
    {
        self.camera_transform_2d.merge(
            output.camera_transform_2d,
            alive_id,
            time_eval,
            parent_time_interval,
            child_time_interval,
        );
        self.planar_trimesh.merge(
            output.planar_trimesh,
            alive_id,
            time_eval,
            parent_time_interval,
            child_time_interval,
        );
    }

    fn archive(self) -> Self::Output {
        BuiltinPlanarLayer {
            camera_transform_2d: self.camera_transform_2d.archive(),
            planar_trimesh: self.planar_trimesh.archive(),
        }
    }
}

impl ::morphing_core::stage::Layer for BuiltinPlanarLayer {
    type Residue<'t, W, LI> = BuiltinPlanarLayer<
        ::morphing_core::stage::ChannelAttachment<'t, W, LI, Self, BuiltinPlanarLayerCameraTransfrom2DChannel, ::morphing_core::stage::ChannelType<CameraTransform2DPresentation>, CameraTransform2DPresentation>,
        ::morphing_core::stage::ChannelAttachment<'t, W, LI, Self, BuiltinPlanarLayerPlanarTrimeshChannel, ::morphing_core::stage::ChannelType<PlanarTrimeshPresentation>, PlanarTrimeshPresentation>,
    > where
        W: ::morphing_core::stage::World,
        LI: ::morphing_core::stage::LayerIndex<W, Layer = Self>;

    fn attachment<'t, W, LI>(
        &'t self,
        config: &'t ::morphing_core::config::Config,
        timer: &'t ::morphing_core::timer::Timer,
        world: &'t W,
    ) -> ::morphing_core::stage::LayerAttachment<'t, W, LI, Self, Self::Residue<'t, W, LI>>
    where
        W: ::morphing_core::stage::World,
        LI: ::morphing_core::stage::LayerIndex<W, Layer = Self>,
    {
        ::morphing_core::stage::LayerAttachment {
            config,
            timer,
            world,
            layer_index: ::std::marker::PhantomData,
            layer: self,
            residue: BuiltinPlanarLayer {
                camera_transform_2d: self
                    .camera_transform_2d
                    .attachment(config, timer, world, self),
                planar_trimesh: self.planar_trimesh.attachment(config, timer, world, self),
            },
        }
    }
}

impl ::morphing_core::stage::Allocate
    for BuiltinPlanarLayer<
        <::morphing_core::stage::ChannelType<CameraTransform2DPresentation> as ::morphing_core::stage::Archive>::Output,
        <::morphing_core::stage::ChannelType<PlanarTrimeshPresentation> as ::morphing_core::stage::Archive>::Output,
    >
{
    type Output = BuiltinPlanarLayer<
        <<::morphing_core::stage::ChannelType<CameraTransform2DPresentation> as ::morphing_core::stage::Archive>::Output as ::morphing_core::stage::Allocate>::Output,
        <<::morphing_core::stage::ChannelType<PlanarTrimeshPresentation> as ::morphing_core::stage::Archive>::Output as ::morphing_core::stage::Allocate>::Output,
    >;

    fn allocate(self, slot_key_generator_type_map: &mut ::morphing_core::storable::SlotKeyGeneratorTypeMap) -> Self::Output {
        BuiltinPlanarLayer {
            camera_transform_2d: self.camera_transform_2d.allocate(slot_key_generator_type_map),
            planar_trimesh: self.planar_trimesh.allocate(slot_key_generator_type_map),
        }
    }
}

impl ::morphing_core::stage::Prepare
    for BuiltinPlanarLayer<
        <<::morphing_core::stage::ChannelType<CameraTransform2DPresentation> as ::morphing_core::stage::Archive>::Output as ::morphing_core::stage::Allocate>::Output,
        <<::morphing_core::stage::ChannelType<PlanarTrimeshPresentation> as ::morphing_core::stage::Archive>::Output as ::morphing_core::stage::Allocate>::Output,
    >
{
    type Output = BuiltinPlanarLayer<
        <<<::morphing_core::stage::ChannelType<CameraTransform2DPresentation> as ::morphing_core::stage::Archive>::Output as ::morphing_core::stage::Allocate>::Output as ::morphing_core::stage::Prepare>::Output,
        <<<::morphing_core::stage::ChannelType<PlanarTrimeshPresentation> as ::morphing_core::stage::Archive>::Output as ::morphing_core::stage::Allocate>::Output as ::morphing_core::stage::Prepare>::Output,
    >;

    fn prepare(
        &self,
        time: ::morphing_core::timer::Time,
        storage_type_map: &mut ::morphing_core::storable::StorageTypeMap,
        device: &::wgpu::Device,
        queue: &::wgpu::Queue,
        format: ::wgpu::TextureFormat,
    ) -> Self::Output {
        BuiltinPlanarLayer {
            camera_transform_2d: self
                .camera_transform_2d
                .prepare(time, storage_type_map, device, queue, format),
            planar_trimesh: self
                .planar_trimesh
                .prepare(time, storage_type_map, device, queue, format),
        }
    }
}

pub struct BuiltinPlanarLayerCameraTransfrom2DChannel;

impl ::morphing_core::stage::ChannelIndex<BuiltinPlanarLayer>
    for BuiltinPlanarLayerCameraTransfrom2DChannel
{
    type Channel = ::morphing_core::stage::ChannelType<CameraTransform2DPresentation>;

    fn index_attachment<'t, 'a, W, LI>(
        attachment: &'a ::morphing_core::stage::LayerAttachment<
            't,
            W,
            LI,
            BuiltinPlanarLayer,
            <BuiltinPlanarLayer as ::morphing_core::stage::Layer>::Residue<'t, W, LI>,
        >,
    ) -> &'a ::morphing_core::stage::ChannelAttachment<
        't,
        W,
        LI,
        BuiltinPlanarLayer,
        Self,
        Self::Channel,
        <Self::Channel as ::morphing_core::stage::Channel>::Presentation,
    >
    where
        W: ::morphing_core::stage::World,
        LI: ::morphing_core::stage::LayerIndex<W, Layer = BuiltinPlanarLayer>,
    {
        &attachment.residue.camera_transform_2d
    }
}

pub struct BuiltinPlanarLayerPlanarTrimeshChannel;

impl ::morphing_core::stage::ChannelIndex<BuiltinPlanarLayer>
    for BuiltinPlanarLayerPlanarTrimeshChannel
{
    type Channel = ::morphing_core::stage::ChannelType<PlanarTrimeshPresentation>;

    fn index_attachment<'t, 'a, W, LI>(
        attachment: &'a ::morphing_core::stage::LayerAttachment<
            't,
            W,
            LI,
            BuiltinPlanarLayer,
            <BuiltinPlanarLayer as ::morphing_core::stage::Layer>::Residue<'t, W, LI>,
        >,
    ) -> &'a ::morphing_core::stage::ChannelAttachment<
        't,
        W,
        LI,
        BuiltinPlanarLayer,
        Self,
        Self::Channel,
        <Self::Channel as ::morphing_core::stage::Channel>::Presentation,
    >
    where
        W: ::morphing_core::stage::World,
        LI: ::morphing_core::stage::LayerIndex<W, Layer = BuiltinPlanarLayer>,
    {
        &attachment.residue.planar_trimesh
    }
}

// hand-written

static BUILTIN_PLANAR_PIPELINE: OnceLock<wgpu::RenderPipeline> = OnceLock::new();
fn builtin_planar_pipeline(device: &wgpu::Device) -> &'static wgpu::RenderPipeline {
    BUILTIN_PLANAR_PIPELINE.get_or_init(|| {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../shaders/builtin_planar.wgsl"
            ))),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                TransformShaderTypes::bind_group_layout(device),
                PaintShaderTypes::bind_group_layout(device),
                CameraTransform2DShaderTypes::bind_group_layout(device),
            ],
            push_constant_ranges: &[],
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: Vertex::min_size().get(),
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: Vertex::METADATA.offset(0),
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x2,
                    }],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb, // TODO: check if color channels messed up?
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    })
}

impl ::morphing_core::stage::Render
    for BuiltinPlanarLayer<
        Vec<::morphing_core::timeline::PresentationKey<CameraTransform2DPresentation>>,
        Vec<::morphing_core::timeline::PresentationKey<PlanarTrimeshPresentation>>,
    >
{
    fn render(
        &self,
        storage_type_map: &::morphing_core::storable::StorageTypeMap,
        encoder: &mut ::wgpu::CommandEncoder,
        target: &::wgpu::TextureView,
    ) {
    }
}
