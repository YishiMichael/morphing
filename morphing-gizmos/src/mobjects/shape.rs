use std::borrow::Cow;
use std::sync::OnceLock;

use encase::ShaderType;
use iced::widget::shader::wgpu::util::DeviceExt;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillVertex, FillVertexConstructor, StrokeOptions, StrokeVertex,
    StrokeVertexConstructor,
};
use morphing_core::config::Config;
use morphing_core::traits::Mobject;
use morphing_core::traits::MobjectBuilder;
use morphing_core::traits::MobjectPresentation;

use super::super::components::camera::{Camera, CameraShaderTypes};
use super::super::components::color::Palette;
use super::super::components::component::Component;
use super::super::components::component::ComponentShaderTypes;
use super::super::components::fill::Fill;
use super::super::components::paint::{Paint, PaintShaderTypes};
use super::super::components::path::Path;
use super::super::components::stroke::Stroke;
use super::super::components::transform::Transform;
use super::super::components::transform::TransformShaderTypes;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ShapeMobject {
    pub(crate) transform: Transform,
    pub(crate) path: Path,
    pub(crate) fill: Option<Fill>,
    pub(crate) stroke: Option<Stroke>,
}

#[derive(ShaderType)]
struct Vertex {
    position: nalgebra::Vector2<f32>,
}

struct VertexConstructor;

impl FillVertexConstructor<Vertex> for VertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> Vertex {
        let (x, y) = vertex.position().into();
        Vertex {
            position: nalgebra::Vector2::new(x, y),
        }
    }
}

impl StrokeVertexConstructor<Vertex> for VertexConstructor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> Vertex {
        let (x, y) = vertex.position().into();
        Vertex {
            position: nalgebra::Vector2::new(x, y),
        }
    }
}

static PLANAR_TRNANGLES_PIPELINE: OnceLock<iced::widget::shader::wgpu::RenderPipeline> =
    OnceLock::new();

pub struct PlanarTrianglesPresentation {
    pipeline: &'static iced::widget::shader::wgpu::RenderPipeline,
    transform_bind_group: iced::widget::shader::wgpu::BindGroup,
    paint_bind_group: iced::widget::shader::wgpu::BindGroup,
    camera_bind_group: iced::widget::shader::wgpu::BindGroup,
    vertex_buffer: iced::widget::shader::wgpu::Buffer,
    index_buffer: iced::widget::shader::wgpu::Buffer,
}

impl PlanarTrianglesPresentation {
    fn pipeline(
        device: &iced::widget::shader::wgpu::Device,
    ) -> &'static iced::widget::shader::wgpu::RenderPipeline {
        PLANAR_TRNANGLES_PIPELINE.get_or_init(|| {
            let shader_module =
                device.create_shader_module(iced::widget::shader::wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: iced::widget::shader::wgpu::ShaderSource::Wgsl(Cow::Borrowed(
                        include_str!("../shaders/planar_triangles.wgsl"),
                    )),
                });
            let pipeline_layout = device.create_pipeline_layout(
                &iced::widget::shader::wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[
                        TransformShaderTypes::bind_group_layout(device),
                        PaintShaderTypes::bind_group_layout(device),
                        CameraShaderTypes::bind_group_layout(device),
                    ],
                    push_constant_ranges: &[],
                },
            );
            device.create_render_pipeline(&iced::widget::shader::wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: iced::widget::shader::wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &[iced::widget::shader::wgpu::VertexBufferLayout {
                        array_stride: Vertex::min_size().get(),
                        step_mode: iced::widget::shader::wgpu::VertexStepMode::Vertex,
                        attributes: &[iced::widget::shader::wgpu::VertexAttribute {
                            offset: Vertex::METADATA.offset(0),
                            shader_location: 0,
                            format: iced::widget::shader::wgpu::VertexFormat::Float32x2,
                        }],
                    }],
                },
                fragment: Some(iced::widget::shader::wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: "fs_main",
                    targets: &[Some(iced::widget::shader::wgpu::ColorTargetState {
                        format: iced::widget::shader::wgpu::TextureFormat::Bgra8UnormSrgb, // TODO: check if color channels messed up?
                        blend: Some(iced::widget::shader::wgpu::BlendState {
                            color: iced::widget::shader::wgpu::BlendComponent {
                                src_factor: iced::widget::shader::wgpu::BlendFactor::One,
                                dst_factor: iced::widget::shader::wgpu::BlendFactor::One,
                                operation: iced::widget::shader::wgpu::BlendOperation::Add,
                            },
                            alpha: iced::widget::shader::wgpu::BlendComponent {
                                src_factor: iced::widget::shader::wgpu::BlendFactor::One,
                                dst_factor: iced::widget::shader::wgpu::BlendFactor::One,
                                operation: iced::widget::shader::wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: iced::widget::shader::wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: iced::widget::shader::wgpu::PrimitiveState {
                    topology: iced::widget::shader::wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: iced::widget::shader::wgpu::FrontFace::Ccw,
                    cull_mode: Some(iced::widget::shader::wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                    // or Features::POLYGON_MODE_POINT
                    polygon_mode: iced::widget::shader::wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None,
                multisample: iced::widget::shader::wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            })
        })
    }
}

impl MobjectPresentation for PlanarTrianglesPresentation {
    fn draw<'rp>(&'rp self, render_pass: &mut iced::widget::shader::wgpu::RenderPass<'rp>) {
        render_pass.set_pipeline(self.pipeline);
        render_pass.set_bind_group(0, &self.transform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.paint_bind_group, &[]);
        render_pass.set_bind_group(2, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            self.index_buffer.slice(..),
            iced::widget::shader::wgpu::IndexFormat::Uint32,
        );
        render_pass.draw(0..3, 0..1);
    }
}

pub struct VecPlanarTrianglesPresentation(Vec<PlanarTrianglesPresentation>);

impl FromIterator<PlanarTrianglesPresentation> for VecPlanarTrianglesPresentation {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = PlanarTrianglesPresentation>,
    {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for VecPlanarTrianglesPresentation {
    type Item = PlanarTrianglesPresentation;
    type IntoIter = <Vec<PlanarTrianglesPresentation> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl MobjectPresentation for VecPlanarTrianglesPresentation {
    fn draw<'rp>(&'rp self, render_pass: &mut iced::widget::shader::wgpu::RenderPass<'rp>) {
        for planar_triangles_presentation in &self.0 {
            planar_triangles_presentation.draw(render_pass);
        }
    }
}

impl Mobject for ShapeMobject {
    type MobjectPresentation = VecPlanarTrianglesPresentation;

    fn presentation(
        &self,
        device: &iced::widget::shader::wgpu::Device,
    ) -> Self::MobjectPresentation {
        std::iter::empty()
            .chain(self.fill.iter().map(|fill| {
                let lyon_path = self.path.to_lyon_path();
                let mut vertex_buffers: lyon::tessellation::VertexBuffers<Vertex, u32> =
                    lyon::tessellation::VertexBuffers::new();
                let mut vertex_builder =
                    BuffersBuilder::new(&mut vertex_buffers, VertexConstructor);
                let mut tessellator = lyon::tessellation::FillTessellator::new();
                assert!(tessellator
                    .tessellate(lyon_path.iter(), &fill.options, &mut vertex_builder)
                    .is_ok());
                (vertex_buffers, &fill.paint)
            }))
            .chain(self.stroke.iter().map(|stroke| {
                let lyon_path = if let Some(dash_pattern) = stroke.dash_pattern.as_ref() {
                    self.path.dash(dash_pattern).to_lyon_path()
                } else {
                    self.path.to_lyon_path()
                };
                let mut vertex_buffers: lyon::tessellation::VertexBuffers<Vertex, u32> =
                    lyon::tessellation::VertexBuffers::new();
                let mut vertex_builder =
                    BuffersBuilder::new(&mut vertex_buffers, VertexConstructor);
                let mut tessellator = lyon::tessellation::StrokeTessellator::new();
                assert!(tessellator
                    .tessellate(lyon_path.iter(), &stroke.options, &mut vertex_builder)
                    .is_ok());
                (vertex_buffers, &stroke.paint)
            }))
            .map(|(vertex_buffers, paint)| {
                let pipeline = PlanarTrianglesPresentation::pipeline(device);

                // buffers
                let transform_shader_types = self.transform.to_shader_types();
                let transform_buffers = transform_shader_types.initialize_buffers(device);
                let transform_bind_group =
                    TransformShaderTypes::bind_group_from_buffers(device, &transform_buffers);

                let camera_shader_types = Camera::default().to_shader_types();
                let camera_buffers = camera_shader_types.initialize_buffers(device);
                let camera_bind_group =
                    CameraShaderTypes::bind_group_from_buffers(device, &camera_buffers);

                let paint_shader_types = paint.to_shader_types();
                let paint_buffers = paint_shader_types.initialize_buffers(device);
                let paint_bind_group =
                    PaintShaderTypes::bind_group_from_buffers(device, &paint_buffers);

                // TODO
                let vertex_buffer = {
                    let mut buffer = encase::StorageBuffer::new(Vec::<u8>::new());
                    buffer.write(&vertex_buffers.vertices).unwrap();
                    device.create_buffer_init(
                        &iced::widget::shader::wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents: buffer.as_ref(),
                            usage: iced::widget::shader::wgpu::BufferUsages::VERTEX,
                        },
                    )
                };
                let index_buffer = {
                    let mut buffer = encase::StorageBuffer::new(Vec::<u8>::new());
                    buffer.write(&vertex_buffers.indices).unwrap();
                    device.create_buffer_init(
                        &iced::widget::shader::wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents: buffer.as_ref(),
                            usage: iced::widget::shader::wgpu::BufferUsages::INDEX,
                        },
                    )
                }; // TODO

                PlanarTrianglesPresentation {
                    pipeline,
                    transform_bind_group,
                    paint_bind_group,
                    camera_bind_group,
                    vertex_buffer,
                    index_buffer,
                }
            })
            .collect()
    }

    // let mut encoder = renderer
    //     .device
    //     .create_command_encoder(&iced::widget::shader::wgpu::CommandEncoderDescriptor { label: None });
    // let frame = renderer.surface.get_current_texture().unwrap();
    // let frame_view = frame
    //     .texture
    //     .create_view(&iced::widget::shader::wgpu::TextureViewDescriptor::default());
    // {
    //     let mut frame_pass = encoder.begin_render_pass(&iced::widget::shader::wgpu::RenderPassDescriptor {
    //         label: None,
    //         color_attachments: &[Some(iced::widget::shader::wgpu::RenderPassColorAttachment {
    //             view: &frame_view,
    //             resolve_target: None,
    //             ops: iced::widget::shader::wgpu::Operations {
    //                 load: iced::widget::shader::wgpu::LoadOp::Clear(iced::widget::shader::wgpu::Color::BLACK),
    //                 store: iced::widget::shader::wgpu::StoreOp::Store,
    //             },
    //         })],
    //         depth_stencil_attachment: None,
    //         timestamp_writes: None,
    //         occlusion_query_set: None,
    //     });
    //     frame_pass.set_pipeline(pipeline);
    //     frame_pass.set_bind_group(0, transform_bind_group, &[]);
    //     frame_pass.set_bind_group(1, paint_bind_group, &[]);
    //     frame_pass.set_bind_group(2, camera_bind_group, &[]);
    //     frame_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    //     frame_pass.set_index_buffer(index_buffer.slice(..), iced::widget::shader::wgpu::IndexFormat::Uint32);
    //     frame_pass.draw(0..3, 0..1);
    // }

    // renderer.queue.submit(Some(encoder.finish()));
    // frame.present();
}

// TODO: port ctors from bezier_rs::Subpath

pub struct Rect(pub nalgebra::Vector2<f64>);

impl MobjectBuilder for Rect {
    type Instantiation = ShapeMobject;

    fn instantiate(self, _config: &Config) -> Self::Instantiation {
        ShapeMobject {
            transform: Transform::default(),
            path: Path::from_iter(std::iter::once(bezier_rs::Subpath::new_rect(
                -glam::DVec2::new(self.0.x, self.0.y) / 2.0,
                glam::DVec2::new(self.0.x, self.0.y) / 2.0,
            ))),
            fill: Some(Fill {
                options: FillOptions::default(),
                paint: Paint {
                    color: Palette::White.into(),
                    gradients: Vec::new(),
                },
            }),
            stroke: Some(Stroke {
                dash_pattern: None,
                options: StrokeOptions::default(),
                paint: Paint {
                    color: Palette::Teal.into(),
                    gradients: Vec::new(),
                },
            }),
        }
    }
}
