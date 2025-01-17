use encase::ShaderType;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillVertex, FillVertexConstructor, StrokeOptions, StrokeVertex,
    StrokeVertexConstructor, VertexBuffers,
};
use palette::WithAlpha;
use wgpu::util::DeviceExt;

use crate::components::camera::{Camera, CameraShaderTypes};
use crate::components::paint::{Paint, PaintShaderTypes};
use crate::components::transform::TransformShaderTypes;
use crate::toplevel::palette::{TEAL, WHITE};

use super::super::components::fill::Fill;
use super::super::components::path::Path;
use super::super::components::stroke::Stroke;
use super::super::components::transform::Transform;
use super::super::toplevel::renderer::Renderer;
use super::mobject::{Mobject, MobjectBuilder};

#[derive(Clone)]
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

impl Mobject for ShapeMobject {
    fn render(&self, renderer: &Renderer) {
        // pipeline
        let device = &renderer.device;

        let transform_bind_group_layout = TransformShaderTypes::create_bind_group_layout(&device);
        let paint_bind_group_layout = PaintShaderTypes::create_bind_group_layout(&device);
        let camera_bind_group_layout = CameraShaderTypes::create_bind_group_layout(&device);
        let pipeline = {
            let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "../shaders/shape.wgsl"
                ))),
            });
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    &transform_bind_group_layout,
                    &paint_bind_group_layout,
                    &camera_bind_group_layout,
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
                        format: wgpu::TextureFormat::R32Float,
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
        };

        fn render_half(
            transform_bind_group: &wgpu::BindGroup,
            camera_bind_group: &wgpu::BindGroup,
            paint_bind_group: &wgpu::BindGroup,
            vertex_buffers: VertexBuffers<Vertex, u32>,
            pipeline: &wgpu::RenderPipeline,
            renderer: &Renderer,
        ) {
            let vertex_buffer = {
                let mut buffer = encase::UniformBuffer::new(Vec::<u8>::new());
                buffer.write(&vertex_buffers.vertices).unwrap();
                renderer
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: buffer.as_ref(),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
            };
            let index_buffer = {
                let mut buffer = encase::UniformBuffer::new(Vec::<u8>::new());
                buffer.write(&vertex_buffers.indices).unwrap();
                renderer
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: buffer.as_ref(),
                        usage: wgpu::BufferUsages::INDEX,
                    })
            };

            let mut encoder = renderer
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            let frame = renderer.surface.get_current_texture().unwrap();
            let frame_view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            {
                let mut frame_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &frame_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                frame_pass.set_pipeline(pipeline);
                frame_pass.set_bind_group(0, transform_bind_group, &[]);
                frame_pass.set_bind_group(1, camera_bind_group, &[]);
                frame_pass.set_bind_group(2, paint_bind_group, &[]);
                frame_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                frame_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                frame_pass.draw(0..3, 0..1);
            }

            renderer.queue.submit(Some(encoder.finish()));
            frame.present();
        }

        // buffers
        let transform_shader_types = self.transform.to_shader_types();
        let transform_buffers = transform_shader_types.create_buffers_init(device);
        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &transform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffers.transform_uniform.as_entire_binding(),
            }],
        });

        let camera_shader_types = Camera::default().to_shader_types();
        let camera_buffers = camera_shader_types.create_buffers_init(device);
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffers.camera_uniform.as_entire_binding(),
            }],
        });

        if let Some(fill) = self.fill.as_ref() {
            let paint_shader_types = fill.paint.to_shader_types();
            let paint_buffers = paint_shader_types.create_buffers_init(device);
            let paint_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &paint_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: paint_buffers.paint_uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: paint_buffers.gradients_storage.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: paint_buffers.radial_stops_storage.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: paint_buffers.angular_stops_storage.as_entire_binding(),
                    },
                ],
            });

            let lyon_path = self.path.to_lyon_path();
            let mut vertex_buffers: lyon::tessellation::VertexBuffers<Vertex, u32> =
                lyon::tessellation::VertexBuffers::new();
            let mut vertex_builder = BuffersBuilder::new(&mut vertex_buffers, VertexConstructor);
            let mut tessellator = lyon::tessellation::FillTessellator::new();
            assert!(tessellator
                .tessellate(lyon_path.iter(), &fill.options, &mut vertex_builder)
                .is_ok());

            render_half(
                &transform_bind_group,
                &camera_bind_group,
                &paint_bind_group,
                vertex_buffers,
                &pipeline,
                renderer,
            );
        }

        if let Some(stroke) = self.stroke.as_ref() {
            let paint_shader_types = stroke.paint.to_shader_types();
            let paint_buffers = paint_shader_types.create_buffers_init(device);
            let paint_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &paint_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: paint_buffers.paint_uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: paint_buffers.gradients_storage.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: paint_buffers.radial_stops_storage.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: paint_buffers.angular_stops_storage.as_entire_binding(),
                    },
                ],
            });

            let lyon_path = if let Some(dash_pattern) = stroke.dash_pattern.as_ref() {
                self.path.dash(dash_pattern).to_lyon_path()
            } else {
                self.path.to_lyon_path()
            };
            let mut vertex_buffers: lyon::tessellation::VertexBuffers<Vertex, u32> =
                lyon::tessellation::VertexBuffers::new();
            let mut vertex_builder = BuffersBuilder::new(&mut vertex_buffers, VertexConstructor);
            let mut tessellator = lyon::tessellation::StrokeTessellator::new();
            assert!(tessellator
                .tessellate(lyon_path.iter(), &stroke.options, &mut vertex_builder)
                .is_ok());

            render_half(
                &transform_bind_group,
                &camera_bind_group,
                &paint_bind_group,
                vertex_buffers,
                &pipeline,
                renderer,
            );
        }
    }
}

// TODO: port ctors from bezier_rs::Subpath

pub struct Rect {
    pub min: nalgebra::Vector2<f64>,
    pub max: nalgebra::Vector2<f64>,
}

impl MobjectBuilder for Rect {
    type Instantiation = ShapeMobject;

    fn instantiate(self, _world: &crate::toplevel::world::World) -> Self::Instantiation {
        ShapeMobject {
            transform: Transform::default(),
            path: Path::from_iter(std::iter::once(bezier_rs::Subpath::new_rect(
                glam::DVec2::new(self.min.x, self.min.y),
                glam::DVec2::new(self.max.x, self.max.y),
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
