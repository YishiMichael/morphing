use std::borrow::Cow;
use std::sync::OnceLock;

use encase::ShaderType;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillVertex, FillVertexConstructor, StrokeOptions, StrokeVertex,
    StrokeVertexConstructor,
};
use palette::WithAlpha;
use wgpu::util::DeviceExt;

use super::super::components::camera::{Camera, CameraShaderTypes};
use super::super::components::component::Component;
use super::super::components::component::ComponentShaderTypes;
use super::super::components::fill::Fill;
use super::super::components::paint::{Paint, PaintShaderTypes};
use super::super::components::path::Path;
use super::super::components::stroke::Stroke;
use super::super::components::transform::Transform;
use super::super::components::transform::TransformShaderTypes;
use super::super::toplevel::palette::{TEAL, WHITE};
use super::super::toplevel::world::World;
use super::mobject::{Mobject, MobjectBuilder, MobjectRealization};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
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

impl MobjectRealization for Vec<PlanarTrianglesRealization> {
    fn render(&self, render_pass: &mut wgpu::RenderPass) -> anyhow::Result<()> {
        for planar_triangles_realization in self {
            planar_triangles_realization.render(render_pass)?;
        }
        Ok(())
    }
}

static PLANAR_TRNANGLES_PIPELINE: OnceLock<wgpu::RenderPipeline> = OnceLock::new();

pub struct PlanarTrianglesRealization {
    pipeline: &'static wgpu::RenderPipeline,
    transform_bind_group: wgpu::BindGroup,
    paint_bind_group: wgpu::BindGroup,
    camera_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl PlanarTrianglesRealization {
    fn pipeline(device: &wgpu::Device) -> &'static wgpu::RenderPipeline {
        PLANAR_TRNANGLES_PIPELINE.get_or_init(|| {
            let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "../shaders/planar_triangles.wgsl"
                ))),
            });
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    TransformShaderTypes::bind_group_layout(device),
                    PaintShaderTypes::bind_group_layout(device),
                    CameraShaderTypes::bind_group_layout(device),
                ],
                push_constant_ranges: &[],
            });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: None, // TODO: check; Some("vs_main")
                    compilation_options: Default::default(),
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
                    entry_point: None,
                    compilation_options: Default::default(),
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
                cache: None,
            })
        })
    }
}

impl MobjectRealization for PlanarTrianglesRealization {
    fn render(&self, render_pass: &mut wgpu::RenderPass) -> anyhow::Result<()> {
        render_pass.set_pipeline(self.pipeline);
        render_pass.set_bind_group(0, &self.transform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.paint_bind_group, &[]);
        render_pass.set_bind_group(2, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw(0..3, 0..1);
        Ok(())
    }
}

impl Mobject for ShapeMobject {
    type Realization = Vec<PlanarTrianglesRealization>;

    fn realize(&self, device: &wgpu::Device) -> anyhow::Result<Self::Realization> {
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
                let pipeline = PlanarTrianglesRealization::pipeline(device);

                // buffers
                let transform_shader_types = self.transform.to_shader_types();
                let transform_buffers = transform_shader_types.initialize_buffers(device)?;
                let transform_bind_group =
                    TransformShaderTypes::bind_group_from_buffers(device, &transform_buffers);

                let camera_shader_types = Camera::default().to_shader_types();
                let camera_buffers = camera_shader_types.initialize_buffers(device)?;
                let camera_bind_group =
                    CameraShaderTypes::bind_group_from_buffers(device, &camera_buffers);

                let paint_shader_types = paint.to_shader_types();
                let paint_buffers = paint_shader_types.initialize_buffers(device)?;
                let paint_bind_group =
                    PaintShaderTypes::bind_group_from_buffers(device, &paint_buffers);

                // TODO
                let vertex_buffer = {
                    let mut buffer = encase::StorageBuffer::new(Vec::<u8>::new());
                    buffer.write(&vertex_buffers.vertices)?;
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: buffer.as_ref(),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
                };
                let index_buffer = {
                    let mut buffer = encase::StorageBuffer::new(Vec::<u8>::new());
                    buffer.write(&vertex_buffers.indices)?;
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: buffer.as_ref(),
                        usage: wgpu::BufferUsages::INDEX,
                    })
                }; // TODO

                Ok(PlanarTrianglesRealization {
                    pipeline,
                    transform_bind_group,
                    paint_bind_group,
                    camera_bind_group,
                    vertex_buffer,
                    index_buffer,
                })
            })
            .collect()
    }

    // let mut encoder = renderer
    //     .device
    //     .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    // let frame = renderer.surface.get_current_texture().unwrap();
    // let frame_view = frame
    //     .texture
    //     .create_view(&wgpu::TextureViewDescriptor::default());
    // {
    //     let mut frame_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    //         label: None,
    //         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
    //             view: &frame_view,
    //             resolve_target: None,
    //             ops: wgpu::Operations {
    //                 load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
    //                 store: wgpu::StoreOp::Store,
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
    //     frame_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    //     frame_pass.draw(0..3, 0..1);
    // }

    // renderer.queue.submit(Some(encoder.finish()));
    // frame.present();
}

// TODO: port ctors from bezier_rs::Subpath

pub struct Rect(pub nalgebra::Vector2<f64>);

impl MobjectBuilder for Rect {
    type Instantiation = ShapeMobject;

    fn instantiate(self, _world: &World) -> Self::Instantiation {
        ShapeMobject {
            transform: Transform::default(),
            path: Path::from_iter(std::iter::once(bezier_rs::Subpath::new_rect(
                -glam::DVec2::new(self.0.x, self.0.y) / 2.0,
                glam::DVec2::new(self.0.x, self.0.y) / 2.0,
            ))),
            fill: Some(Fill {
                options: FillOptions::default(),
                paint: Paint {
                    color: WHITE.into_format().with_alpha(1.0),
                    gradients: Vec::new(),
                },
            }),
            stroke: Some(Stroke {
                dash_pattern: None,
                options: StrokeOptions::default(),
                paint: Paint {
                    color: TEAL.into_format().with_alpha(1.0),
                    gradients: Vec::new(),
                },
            }),
        }
    }
}
